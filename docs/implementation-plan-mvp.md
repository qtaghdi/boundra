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

## Not In MVP

The following are intentionally deferred:

- npm package or binary distribution.
- public release workflow.
- full TypeScript parser or SWC backend.
- schema-backed codegen.
- real application dogfooding.
- Starlark/Lua extension runtime.

## Next Validation

Before release work, Boundra should be used in a real internal project flow:

- create at least two domains
- generate route/query/mutation artifacts
- wire generated contracts into an app or package
- run `check-boundaries` and `graph-domains` during normal development
- record friction before adding extension scripting
