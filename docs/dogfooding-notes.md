# Dogfooding Notes

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

- the TypeScript compiler resolves generated contracts and `@boundra/runtime`
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
- `@boundra/runtime` validates client transport and server handler boundaries
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
- added `@domains/*` and `@boundra/runtime` path aliases in a temporary root `tsconfig.json`
- set `billing` to depend on `order`
- ran:
  - `cargo test`
  - `check-boundaries --format json`
  - `graph-domains --format json`
  - `graph-domains --format mermaid`

### Result

The first dogfooding pass looked healthy:

- generated contracts imported `@boundra/runtime` helper types correctly
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
