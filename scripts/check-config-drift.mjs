import fs from "node:fs";
import path from "node:path";
import process from "node:process";

import ts from "typescript";

const root = process.cwd();
const rustPath = path.join(root, "crates/core/src/lib.rs");
const typescriptPath = path.join(root, "packages/config/src/index.ts");

const rustSource = fs.readFileSync(rustPath, "utf8");
const typescriptSourceText = fs.readFileSync(typescriptPath, "utf8");
const typescriptSource = ts.createSourceFile(
  typescriptPath,
  typescriptSourceText,
  ts.ScriptTarget.Latest,
  true,
  ts.ScriptKind.TS,
);

const rustConfigStructs = new Set([
  "RawBoundraConfig",
  "RawProjectConfig",
  "RawProjectPaths",
  "RawDomainDefaults",
  "RawPublicApi",
  "RawCheckBoundariesConfig",
  "RawCapabilityConfig",
  "RawBoundaryPolicyConfig",
  "RawLayerCapabilityPolicy",
]);

function snakeToCamel(value) {
  return value.replace(/_([a-z])/g, (_, letter) => letter.toUpperCase());
}

function rustStructBody(structName) {
  const marker = `struct ${structName} {`;
  const markerIndex = rustSource.indexOf(marker);
  if (markerIndex === -1) {
    throw new Error(`Rust config struct not found: ${structName}`);
  }

  const bodyStart = markerIndex + marker.length;
  let depth = 1;
  for (let index = bodyStart; index < rustSource.length; index += 1) {
    if (rustSource[index] === "{") depth += 1;
    if (rustSource[index] === "}") depth -= 1;
    if (depth === 0) return rustSource.slice(bodyStart, index);
  }

  throw new Error(`Rust config struct is not closed: ${structName}`);
}

function classifyRustType(typeText) {
  const normalized = typeText.replace(/\s+/g, "");

  if (rustConfigStructs.has(normalized)) {
    return { kind: "object", reference: normalized };
  }
  if (normalized === "String") return { kind: "string" };
  if (normalized === "Vec<String>") return { kind: "string[]" };
  if (normalized === "BTreeMap<String,Vec<String>>") {
    return { kind: "record<string,string[]>" };
  }

  throw new Error(`Unsupported Rust config field type: ${typeText}`);
}

function rustFields(structName) {
  const body = rustStructBody(structName);
  const fields = [];

  for (const rawLine of body.split("\n")) {
    const line = rawLine.trim();
    if (!line) continue;

    const match = line.match(/^([A-Za-z_][A-Za-z0-9_]*)\s*:\s*Option<(.+)>,$/);
    if (!match) {
      throw new Error(
        `Rust config field must remain optional and use Option<T>: ${structName}.${line}`,
      );
    }

    fields.push({
      name: snakeToCamel(match[1]),
      type: classifyRustType(match[2]),
    });
  }

  return fields;
}

function collectRustShape(structName, prefix = "", output = new Map()) {
  for (const field of rustFields(structName)) {
    const fieldPath = prefix ? `${prefix}.${field.name}` : field.name;
    output.set(fieldPath, field.type.kind);

    if (field.type.kind === "object") {
      collectRustShape(field.type.reference, fieldPath, output);
    }
  }

  return output;
}

const interfaces = new Map();
const aliases = new Map();
for (const statement of typescriptSource.statements) {
  if (ts.isInterfaceDeclaration(statement)) {
    interfaces.set(statement.name.text, statement);
  } else if (ts.isTypeAliasDeclaration(statement)) {
    aliases.set(statement.name.text, statement);
  }
}

function typeReferenceName(node) {
  return node.typeName.getText(typescriptSource);
}

function classifyTypescriptType(node) {
  if (ts.isTypeOperatorNode(node) && node.operator === ts.SyntaxKind.ReadonlyKeyword) {
    return classifyTypescriptType(node.type);
  }

  if (node.kind === ts.SyntaxKind.StringKeyword) return { kind: "string" };

  if (ts.isArrayTypeNode(node)) {
    const element = classifyTypescriptType(node.elementType);
    if (element.kind === "string") return { kind: "string[]" };
    throw new Error(`Unsupported TypeScript config array type: ${node.getText(typescriptSource)}`);
  }

  if (ts.isTypeReferenceNode(node)) {
    const name = typeReferenceName(node);

    if (name === "Readonly" || name === "ReadonlyArray" || name === "Array") {
      const [inner] = node.typeArguments ?? [];
      if (!inner) throw new Error(`Missing type argument for ${name}`);
      if (name === "Readonly") return classifyTypescriptType(inner);

      const element = classifyTypescriptType(inner);
      if (element.kind === "string") return { kind: "string[]" };
      throw new Error(`Unsupported TypeScript config array type: ${node.getText(typescriptSource)}`);
    }

    if (name === "Record") {
      const [keyType, valueType] = node.typeArguments ?? [];
      if (!keyType || !valueType) throw new Error("Record config type needs two arguments");
      const key = classifyTypescriptType(keyType);
      const value = classifyTypescriptType(valueType);
      if (key.kind === "string" && value.kind === "string[]") {
        return { kind: "record<string,string[]>" };
      }
      throw new Error(`Unsupported TypeScript config record type: ${node.getText(typescriptSource)}`);
    }

    if (interfaces.has(name)) return { kind: "object", reference: name };
    if (aliases.has(name)) return classifyTypescriptType(aliases.get(name).type);
  }

  throw new Error(`Unsupported TypeScript config field type: ${node.getText(typescriptSource)}`);
}

function propertyName(member, interfaceName) {
  if (!member.name) throw new Error(`Unnamed property in ${interfaceName}`);
  if (ts.isIdentifier(member.name) || ts.isStringLiteral(member.name)) return member.name.text;
  throw new Error(`Unsupported property name in ${interfaceName}: ${member.name.getText(typescriptSource)}`);
}

function collectTypescriptShape(interfaceName, prefix = "", output = new Map()) {
  const declaration = interfaces.get(interfaceName);
  if (!declaration) throw new Error(`TypeScript config interface not found: ${interfaceName}`);

  for (const member of declaration.members) {
    if (!ts.isPropertySignature(member)) continue;
    if (!member.questionToken) {
      throw new Error(
        `TypeScript config field must remain optional: ${interfaceName}.${propertyName(member, interfaceName)}`,
      );
    }
    if (!member.type) {
      throw new Error(`TypeScript config field has no type: ${interfaceName}.${propertyName(member, interfaceName)}`);
    }

    const name = propertyName(member, interfaceName);
    const fieldPath = prefix ? `${prefix}.${name}` : name;
    const type = classifyTypescriptType(member.type);
    output.set(fieldPath, type.kind);

    if (type.kind === "object") {
      collectTypescriptShape(type.reference, fieldPath, output);
    }
  }

  return output;
}

const rustShape = collectRustShape("RawBoundraConfig");
const typescriptShape = collectTypescriptShape("BoundraConfig");
const allPaths = [...new Set([...rustShape.keys(), ...typescriptShape.keys()])].sort();
const differences = [];

for (const fieldPath of allPaths) {
  const rustKind = rustShape.get(fieldPath);
  const typescriptKind = typescriptShape.get(fieldPath);
  if (rustKind !== typescriptKind) {
    differences.push(
      `${fieldPath}: rust=${rustKind ?? "<missing>"}, typescript=${typescriptKind ?? "<missing>"}`,
    );
  }
}

if (differences.length > 0) {
  console.error("config drift detected between Rust and TypeScript:\n");
  for (const difference of differences) console.error(`- ${difference}`);
  process.exitCode = 1;
} else {
  console.log(`config drift: OK (${rustShape.size} fields)`);
}
