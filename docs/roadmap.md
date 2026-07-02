# Roadmap

## Phase 0: Foundation Design

Status: complete.

Completed:

- project charter
- architecture baseline
- boundary rules
- CLI specification
- config and manifest specs
- naming convention
- initial ADR

## Phase 1: Core Tooling MVP

Status: complete.

Completed:

- `create-domain`
- `check-boundaries`
- BR-001 through BR-004
- JSON diagnostics and exit codes
- config and manifest loading
- CI integration example
- fixture test coverage

## Phase 2: Framework Workflow MVP

Status: complete for the first slice.

Completed:

- `graph-domains`
- `generate route|query|mutation`
- generated contract registration in `domain.json`
- TypeScript runtime helper surface in `packages/runtime`
- CLI internals split into modules

Completed in the framework-quality follow-up:

- schema-backed codegen templates
- executable contract runtime and client/server adapters
- generated shared public exports
- help/usage output tests

## Phase 3: Dogfooding

Status: complete for the first internal flow.

Completed:

- promoted the framework-neutral TypeScript consumer to `examples/order-billing`
- created `order` and `billing` domains through the Boundra CLI
- type-checked generated route/query/mutation contracts against
  `boundra`
- exercised a declared cross-domain public API import
- added one local and CI verification command with `pnpm verify-example`
- executed runtime input/result validation, client transport, and a server route
- removed manual dependency editing with `add-dependency`
- stabilized CLI error codes, context, suggestions, and JSON failure output

Goals:

- use Boundra in a real internal app flow
- create multiple domains
- wire generated contracts into an app or package
- observe missing runtime APIs
- refine generated file structure before packaging

Done when:

- normal development can use Boundra without manual manifest editing
- generated contracts are useful enough to keep
- repeated friction is captured as concrete issues or docs updates

Resolved friction is documented in `docs/dogfooding-notes.md` and ADR 0002.

## Phase 4: Core Stabilization

Goals:

- introduce a stronger parser backend if needed
- improve diagnostics and suggestions
- improve performance on larger workspaces
- stabilize JSON output schemas
- harden manifest/config schema compatibility

Current priority:

- validate parser edge cases from real repositories before choosing SWC
- improve performance measurements on larger workspaces
- prepare packaging only after the current generated API settles

## Phase 5: Extension Design

Goals:

- evaluate Starlark policy/codegen hooks after dogfooding
- keep Lua as an optional local automation candidate
- avoid extension runtime before real customization pressure exists

## Phase 6: External Release

Goals:

- package `boundra` as compiled ESM and declarations
- publish native CLI archives with checksums
- write public quickstart
- provide examples after dogfooding
- publish only after CLI commands and generated layouts stabilize

Packaging direction is defined by ADR 0003 and `docs/packaging-spec.md`.
