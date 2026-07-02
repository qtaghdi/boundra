# Framework Surface

## 1. Purpose

Boundra should not stop at being a boundary checker. The next layer is a small TypeScript-facing framework surface that lets real applications use Boundra-generated contracts without replacing Next.js, bundlers, ORMs, UI frameworks, or test runners.

This document defines the language split for the next phase.

## 2. Language Responsibilities

### Rust Engine

Rust remains responsible for deterministic tooling:

- project and domain manifest loading
- import scanning and boundary analysis
- rule diagnostics
- dependency graph generation
- CLI orchestration
- code generation file writes

Rust should stay strict, fast, and boring.

### TypeScript Surface

TypeScript should own the application-facing API:

- generated contract types
- route/query/mutation adapters
- client/server helper functions
- framework integration helpers for apps
- developer-facing package entrypoints

TypeScript is the right layer for anything application code imports directly.

### Extension DSL

An extension DSL may be introduced later, but it should not come before real dogfooding.

Preferred order:

1. TypeScript framework surface
2. schema-backed codegen templates
3. dogfood project feedback
4. Starlark policy/codegen hooks
5. optional Lua local automation hooks

## 3. Starlark vs Lua Decision

Boundra should prefer Starlark over Lua for core extensibility.

Reasons:

- Starlark is deterministic and policy-oriented.
- It fits rule/config/codegen customization better than unrestricted scripting.
- It aligns with Boundra's "safe constraints over freedom" principle.
- It avoids turning project policy into arbitrary local scripts too early.

Lua remains a possible later option for local automation or AI/MCP scripting, but not for core policy.

## 4. First TypeScript Package

The first TypeScript-facing package should be intentionally small.

Candidate package:

```txt
packages/
  runtime/
    src/
      types.ts
      index.ts
```

Initial responsibilities:

- expose schema-backed helper types for generated contracts
- define route/query/mutation contract and implementation shapes
- validate contract input and result values
- provide a framework-neutral client/transport boundary
- avoid framework-specific runtime behavior
- avoid depending on React, Next.js, database clients, or server runtimes

Non-goals:

- no HTTP server
- no router replacement
- no built-in fetch transport yet
- no React hooks runtime yet
- no ORM integration

## 5. Generated Code Direction

Generated files use runtime-backed schema shapes.

Example direction:

```ts
import { defineRoute, type InferSchema } from "boundra";
import { z } from "zod";

export const createInvoiceInputSchema = z.object({});
export const createInvoiceResultSchema = z.object({});

export type CreateInvoiceInput = InferSchema<typeof createInvoiceInputSchema>;
export type CreateInvoiceResult = InferSchema<typeof createInvoiceResultSchema>;

export const createInvoiceRoute = defineRoute({
  name: "create-invoice",
  input: createInvoiceInputSchema,
  result: createInvoiceResultSchema,
});
```

Zod authors schemas, while the runtime depends only on a structural `parse`
contract. See `docs/contract-schema-spec.md` and ADR 0002.

## 6. Dogfooding Rule

Do not add Starlark, Lua, or WASM plugin runtime until Boundra has been used in at least one real internal project flow.

The first validation target is complete:

- create at least two domains
- generate at least one route, query, and mutation
- wire generated contracts into an app package or app folder
- run `check-boundaries` and `graph-domains` during normal development
- record missing APIs before adding extension scripting

The committed `examples/order-billing` flow now covers these points and is
verified by `pnpm verify-example`. It exposed placeholder contract types, manual dependency
declarations, and missing public export generation as the next design inputs.

Those inputs are resolved by ADR 0002, schema-backed generated contracts, the
framework-neutral client/server runtime, `add-dependency`, and generated shared
public exports.

## 7. First Implementation Slice

Status: complete.

The first code slice was:

1. add `packages/runtime`
2. define pure TypeScript helper types
3. update generated route/query/mutation templates to import those helper types
4. add a small fixture test that verifies generated files use the runtime surface

This keeps the next step practical without prematurely adding a plugin language.
