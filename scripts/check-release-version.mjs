import { readFile } from "node:fs/promises";
import { resolve } from "node:path";

const repositoryRoot = resolve(import.meta.dirname, "..");
const requested = (process.argv[2] ?? process.env.GITHUB_REF_NAME ?? "")
  .replace(/^v/, "")
  .trim();

if (!requested) {
  throw new Error("release version is required, for example: v0.1.1");
}

const rootPackage = JSON.parse(
  await readFile(resolve(repositoryRoot, "package.json"), "utf8"),
);
const runtimePackage = JSON.parse(
  await readFile(
    resolve(repositoryRoot, "packages/runtime/package.json"),
    "utf8",
  ),
);
const cargoManifest = await readFile(
  resolve(repositoryRoot, "Cargo.toml"),
  "utf8",
);
const cargoVersion = cargoManifest.match(
  /\[workspace\.package\][\s\S]*?^version\s*=\s*"([^"]+)"/m,
)?.[1];

const versions = {
  "root package": rootPackage.version,
  "runtime package": runtimePackage.version,
  "Cargo workspace": cargoVersion,
};

for (const [source, version] of Object.entries(versions)) {
  if (version !== requested) {
    throw new Error(`${source} version ${version ?? "<missing>"} != ${requested}`);
  }
}

console.log(`release-version: OK (${requested})`);
