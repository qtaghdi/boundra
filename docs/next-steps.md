# Next Steps

## Current Baseline

Boundra now has a usable MVP foundation:

- `check-boundaries` detects BR-001 through BR-004.
- `create-domain <name>` scaffolds domain folders, `domain.json`, and public API stubs.
- `boundra.config.json` is loaded for core project paths, domain defaults, scanner extensions, and ignore paths.
- `domains/<domain>/domain.json` is loaded and validated for name, public API files, and declared dependencies.
- BR-004 uses manifest-declared `publicApi` entries when available.
- CLI fixture tests cover boundary output, project roots, scaffolding, manifest validation, and ignored paths.
- Config and manifest JSON parsing uses `serde` and `serde_json`.
- `check-boundaries --format json` is serialized from typed output structs.
- Import scanning supports multiline static imports/exports and dynamic `import(...)`.
- `tsconfig.json` path aliases are resolved during boundary checks.
- `graph-domains` outputs domain dependency graphs as Mermaid, DOT, or JSON.
- `generate route|query|mutation <domain>/<name>` scaffolds contract-centered stubs.
- CLI command handlers are split into focused modules for parsing, output, utilities, and command execution.
- The next framework surface is defined as Rust engine plus TypeScript runtime helpers, with Starlark deferred until dogfooding proves the need.
- `packages/runtime` provides the first pure TypeScript helper types used by generated contracts.

## Completed: Replace Ad Hoc JSON Parsing

The config and manifest loaders now use typed `serde` parsing instead of ad hoc string extraction.

Completed:

- Added `serde` and `serde_json` to `boundra-core`.
- Parse `boundra.config.json` into typed structs with defaults.
- Parse `domain.json` into typed structs with defaults.
- Return invalid JSON errors with file path context.
- Existing E2E tests pass.

## Completed: Stabilize Machine-Readable Output

JSON output is now produced from typed output structs instead of manual string printing.

Completed:

- Added stable output structs for `check-boundaries`.
- Serialize JSON with `serde_json`.
- Include `status`, `violations`, and `meta`.
- Added tests that parse CLI JSON output as JSON.

## Completed: Improve Import Parsing

The lightweight parser now handles the most common real-world import shapes without adopting SWC yet.

Completed:

- Support multiline `import ... from` statements.
- Support multiline `export ... from` statements.
- Support dynamic `import(...)`, including multiline calls.
- Keep a lightweight parser for the current milestone.

## Completed: Add CI Integration

The CLI should be easy to run in a repository before Boundra has packaging.

Completed:

- Add a documented `cargo run -p boundra-cli -- check-boundaries --root .` workflow.
- Add a GitHub Actions example.
- Document expected exit codes.
- Add a small CI-oriented JSON output example.

## Completed: Add Domain Graph Output

Domain dependencies can now be inspected from manifests.

Completed:

- Added `graph-domains`.
- Support `--format mermaid|json|dot`.
- Support `--output <path>`.
- JSON output includes domains, edges, and metadata.

## Completed: Resolve TypeScript Path Aliases

Boundary checks now resolve simple `tsconfig.json` `compilerOptions.paths` aliases.

Completed:

- Parse alias mappings from root `tsconfig.json`.
- Resolve aliases such as `@domains/*` to `domains/*`.
- Preserve existing relative and `domains/` import behavior.
- Added E2E coverage for alias-resolved violations.

## Completed: Start Code Generation

Once config, manifest, and boundary validation are stable, Boundra can move from analyzer to framework workflow.

Completed:

- Implement `generate route <domain>/<name>`.
- Implement `generate query <domain>/<name>`.
- Implement `generate mutation <domain>/<name>`.
- Generate shared contract stubs and server/client placeholders.
- Enforce kebab-case for generated resources.
- Refuse generation when the target domain does not exist.

## Completed: Modularize CLI Internals

The CLI is now split by responsibility so new commands can be added without growing one large file.

Completed:

- Keep `cli.rs` as a thin command router.
- Move argument parsing to `parsing.rs`.
- Move boundary output formatting to `output.rs`.
- Move shared naming/path helpers to `util.rs`.
- Move command handlers into `commands/`.

## Completed: Define Framework Surface Direction

Boundra's next layer is now documented as a TypeScript-facing framework surface rather than an immediate plugin language.

Completed:

- Keep Rust responsible for deterministic tooling.
- Make TypeScript the application-facing runtime/helper layer.
- Defer Starlark/Lua until real project dogfooding exposes customization needs.
- Prefer Starlark over Lua for future core policy/codegen hooks.
- Define `packages/runtime` as the next implementation slice.

## Completed: Add TypeScript Runtime Surface

Generated contracts now target a small TypeScript runtime package instead of standalone placeholder types only.

Completed:

- Add `packages/runtime` as a private TypeScript package.
- Define `BoundraRoute`, `BoundraQuery`, and `BoundraMutation` helper types.
- Update generated route/query/mutation contracts to import runtime helper types.
- Add CLI fixture assertions for runtime-backed generated contracts.

## Next Priority

- Codegen templates backed by schema definitions instead of placeholder contracts.
- Public API update assistance for generated files.
- Help/usage output tests for the modularized CLI.
- More complete parser backend, likely SWC, for comments and TypeScript syntax edge cases.
