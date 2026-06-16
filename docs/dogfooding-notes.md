# Dogfooding Notes

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
