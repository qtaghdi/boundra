# MVP Implementation Status

## Goal

Build a usable core tooling MVP centered on domain scaffolding, boundary checks, graph output, and contract-oriented generation.

## Status

The core tooling MVP is complete.

Completed:

- Rust workspace and crate baseline.
- `check-boundaries` for BR-001 through BR-004.
- `create-domain <name>` scaffold workflow.
- `graph-domains` manifest-based dependency graph output.
- `generate route|query|mutation <domain>/<name>`.
- `boundra.config.json` loading for paths, domain defaults, scan extensions, and ignored paths.
- `domains/<domain>/domain.json` loading and validation.
- `publicApi` and `dependsOn` validation.
- JSON output for CI and agent integrations.
- GitHub Actions example.
- Multiline static import/export and dynamic `import(...)` scanning.
- Basic `tsconfig.json` path alias resolution.
- CLI internals split into command modules.
- `packages/runtime` TypeScript helper surface.
- Generated shared contracts registered into `domain.json` `publicApi.shared`.

## Definition of Done

Done:

- `check-boundaries` detects all four boundary rules.
- Boundary violations return exit code `1`.
- Diagnostics include rule, file, line, import, message, and suggestion.
- CLI commands are covered by fixture tests.
- README and docs index provide an onboarding path.
- `cargo test` passes locally.

## Originally Deferred From MVP

The following are intentionally deferred:

- npm package or binary distribution.
- public release workflow.
- full TypeScript parser or SWC backend.
- Starlark/Lua extension runtime.

Schema-backed codegen and application dogfooding were completed in the
post-MVP framework slice.

## Post-MVP Validation

The first committed dogfood slice is complete:

- `order` and `billing` were created through the Boundra CLI
- route/query/mutation artifacts are consumed by `examples/order-billing`
- generated contracts pass strict TypeScript compilation
- `check-boundaries` and `graph-domains` run in the aggregate
  `pnpm verify-example` gate
- observed friction is recorded in `docs/dogfooding-notes.md`

Dogfooding now executes schema validation, transport calls, and route handlers.
Domain dependencies are managed through `add-dependency`, so the first dogfood
phase is complete without manual manifest maintenance.
