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

## Completed: Replace Ad Hoc JSON Parsing

The config and manifest loaders now use typed `serde` parsing instead of ad hoc string extraction.

Completed:

- Added `serde` and `serde_json` to `boundra-core`.
- Parse `boundra.config.json` into typed structs with defaults.
- Parse `domain.json` into typed structs with defaults.
- Return invalid JSON errors with file path context.
- Existing E2E tests pass.

## Priority 1: Stabilize Machine-Readable Output

JSON output is still assembled by string printing. It should be produced from typed output structs.

Definition of done:

- Add stable output structs for `check-boundaries`.
- Serialize JSON with `serde_json`.
- Include `status`, `violations`, and future-safe metadata fields.
- Add tests that parse CLI JSON output as JSON.

## Priority 2: Improve Import Parsing

The parser is still line-based. It is enough for MVP fixtures, but not enough for real TypeScript projects.

Definition of done:

- Support multiline `import ... from` statements.
- Support `export ... from` statements reliably.
- Support dynamic `import(...)`.
- Decide whether to adopt SWC or keep a lightweight parser for the next milestone.

## Priority 3: Add CI Integration

The CLI should be easy to run in a repository before Boundra has packaging.

Definition of done:

- Add a documented `cargo run -p boundra-cli -- check-boundaries --root .` workflow.
- Add a GitHub Actions example.
- Document expected exit codes.
- Add a small CI-oriented JSON output example.

## Priority 4: Start Code Generation

Once config, manifest, and boundary validation are stable, Boundra can move from analyzer to framework workflow.

Definition of done:

- Implement `generate route <domain>/<name>`.
- Generate shared contract stub and server/client placeholders.
- Enforce kebab-case for generated resources.
- Refuse generation when the target domain does not exist.

## Parking Lot

- Path aliases from `tsconfig.json`.
- Package manager integration.
- npm binary packaging.
- Domain dependency graph output.
- `graph-domains --format mermaid|json|dot`.
