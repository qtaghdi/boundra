# Dogfooding Notes

## 2026-07-10: Real Repository Parser and Performance Audit

### Scope

Boundra was run against a clean Git snapshot of a local four-app Next.js
monorepo containing 773 tracked TypeScript and JavaScript files. Import
extraction was compared by file and module path with the repository's
TypeScript 6 `preProcessFile` output.

The same release CLI was also measured against the real snapshot and the
existing synthetic 1,000-file and 10,000-file fixtures.

### Findings

- TypeScript reported 2,605 raw entries, including four duplicate module
  augmentation references. Boundra reported 2,601 raw imports after fixes.
- After de-duplicating by source file and module path, both scanners produced
  the same 2,559 entries: zero missing imports and zero extra imports.
- Four dynamic imports with interpolated template paths were initially treated
  as static paths. Boundra now ignores interpolated templates while retaining
  static template literal imports.
- A normal commented `tsconfig.json` initially failed strict JSON parsing.
  Boundra now parses TypeScript configuration as JSONC, including comments and
  trailing commas.
- The populated working tree initially produced 11,993 imports because Next,
  Turbo, and agent worktree output was scanned. Default ignores for `.next`,
  `.turbo`, and `.claude/worktrees` reduced that to 2,605; the four entries
  beyond the clean snapshot came from generated `next-env.d.ts` files.

### Performance

Measured on macOS arm64 with Node.js 24.14.0 and the release CLI:

- real 773-file snapshot: 30-40 ms, maximum RSS 3.98 MB
- synthetic 1,000 files: 22.49 ms cold, 19.62 ms warm median, 2.59 MB maximum RSS
- synthetic 10,000 files: 178.12 ms cold, 244.19 ms warm median, 5.97 MB maximum RSS

All runs completed with zero boundary violations in their valid fixtures.

### Decision

Keep the lightweight parser for the current stabilization phase. The real
repository exposed narrow compatibility issues that were fixed without an AST
backend, while normalized import coverage matched TypeScript exactly and the
10,000-file fixture remained below 250 ms warm median.

Reconsider SWC or another full parser only when a broader real-repository corpus
shows syntax-related false negatives, or when new analysis requires AST
semantics that the import scanner cannot represent safely.

## 2026-07-01: Promote the Dogfood Slice to a Public Example

The completed dogfood slice now has stable contracts, runtime behavior, and CI
coverage. It is no longer only an internal fixture, so it lives at
`examples/order-billing` as a self-contained Boundra project.

The repository reserves `apps/` for real deployable products such as a web,
admin, or API application. Example verification still exercises the same
route/query/mutation flow, boundary rules, and domain graph through the example
project root.

## 2026-07-01: Repeatable TypeScript Dogfood Slice

### Goal

Turn the temporary sandbox validation into a committed, repeatable TypeScript
consumer flow. This slice must validate generated contracts with the TypeScript
compiler, not only with Rust fixture assertions.

### Scope

- add the minimum pnpm workspace and TypeScript configuration
- create `order` and `billing` through `create-domain`
- generate one route, query, and mutation through the Boundra CLI
- consume generated public contracts from `examples/order-billing`
- declare `billing -> order` in the billing manifest
- keep the app framework-neutral; do not add Next.js, React, an ORM, or a new
  runtime abstraction

### Acceptance Criteria

The slice is complete when all of the following commands pass against committed
files:

```bash
pnpm typecheck
cargo test --workspace
cargo run -p boundra-cli -- check-boundaries --root examples/order-billing --format json
cargo run -p boundra-cli -- graph-domains --root examples/order-billing --format json
```

The graph output must contain `billing -> order`, generated contracts must be
registered in each domain manifest, and the example must import contracts
through their declared public API paths.

### Learning Target

Record concrete friction around generated placeholder types, runtime helpers,
manifest edits, and public API imports. Use those observations as input to the
schema-backed codegen specification instead of choosing a schema model in
advance.

### Result

The committed slice passes all acceptance commands:

- the TypeScript compiler resolves generated contracts and `boundra`
- `examples/order-billing` consumes the three manifest-declared shared contracts
- a billing server workflow imports an order contract through its declared
  public API without a BR-004 violation
- the graph contains the `billing -> order` edge
- `pnpm verify-example` repeats the complete local and CI validation path

### Observed Friction

- `dependsOn` still requires a manual manifest edit; there is no CLI workflow
  for declaring a domain dependency
- generated input and result types are `Record<string, never>`, so they prove
  wiring but cannot model a useful feature yet
- generated query and mutation hooks are compile-safe placeholders rather than
  executable client adapters
- consumers import manifest-listed contract files directly because generation
  does not maintain a shared public barrel

### Next Decision

Define schema-backed codegen from these constraints. The specification must
choose the contract source of truth, generated-file ownership, and public export
strategy before implementation. Separately specify how domain dependencies are
added without manual manifest editing.

### Resolution

The follow-up framework slice resolved the four observed gaps:

- contracts now use Zod input/result schemas with inferred TypeScript types
- `boundra` validates client transport and server handler boundaries
- generated query/mutation files are executable framework-neutral client
  adapters
- generation maintains `shared/public.ts` exports
- `add-dependency <domain>/<dependency>` updates `dependsOn` idempotently
- CLI and runtime failures now expose stable codes and recovery suggestions

The dogfood command executes valid route/query/mutation flows and verifies an
invalid input produces `RUNTIME-001`.

## 2026-06-16

### What Was Tested

Created a temporary sandbox flow to verify Boundra as a framework workflow before committing any example app:

- created `order` and `billing` domains with `create-domain`
- generated:
  - `billing/create-invoice` route
  - `order/get-order` query
  - `order/submit-order` mutation
- consumed generated contracts from an app-like `apps/sandbox` location
- added `@domains/*` and `boundra` path aliases in a temporary root `tsconfig.json`
- set `billing` to depend on `order`
- ran:
  - `cargo test`
  - `check-boundaries --format json`
  - `graph-domains --format json`
  - `graph-domains --format mermaid`

### Result

The first dogfooding pass looked healthy:

- generated contracts imported `boundra` helper types correctly
- generated shared contracts were registered in `domain.json` `publicApi.shared`
- sandbox app code could import generated contracts through `@domains/*`
- `check-boundaries` passed with no violations
- `graph-domains` showed `billing -> order`

### Decision

Do not commit the temporary sandbox app or sample domains yet.

Reason:

- they are validation artifacts, not official examples
- the TypeScript app/tooling setup is not established yet
- committing them now would blur the line between dogfood, fixture, and product example

Keep the learning, discard the temporary files unless they become a deliberate fixture or example later.

### Why Some Crates Are Empty

The following crates are intentionally present as placeholders:

- `crates/analyzer`
- `crates/codegen`
- `crates/graph`

They are empty because the current MVP keeps orchestration simple:

- graph output currently lives inside `boundra-cli`
- code generation currently lives inside `boundra-cli`
- analyzer responsibilities are currently split between `parser`, `rules`, and `core`

This avoids premature crate boundaries while the APIs are still changing.

Move code into these crates only when:

- the logic becomes large enough to reuse outside the CLI
- CLI code becomes hard to maintain
- there is a stable public Rust API to expose
- dogfooding proves the boundary is worth the extra crate complexity

Until then, the empty crates are roadmap markers, not missing implementation.
