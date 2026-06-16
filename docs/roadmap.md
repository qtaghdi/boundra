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

Remaining:

- schema-backed codegen templates
- stronger generated contract model
- help/usage output tests

## Phase 3: Dogfooding

Status: next.

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

## Phase 4: Core Stabilization

Goals:

- introduce a stronger parser backend if needed
- improve diagnostics and suggestions
- improve performance on larger workspaces
- stabilize JSON output schemas
- harden manifest/config schema compatibility

## Phase 5: Extension Design

Goals:

- evaluate Starlark policy/codegen hooks after dogfooding
- keep Lua as an optional local automation candidate
- avoid extension runtime before real customization pressure exists

## Phase 6: External Release

Goals:

- decide npm/binary packaging strategy
- write public quickstart
- provide examples after dogfooding
- publish only after CLI commands and generated layouts stabilize
