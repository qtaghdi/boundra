# CLI Spec v1

## 1. Commands

Global information flags:

- `--help`, `-h`
- `--version`, `-V`

### 1.0 `init`

Initialize a Boundra workspace without overwriting application files.

Generated targets:
- `boundra.config.json`
- configured `apps/` and `domains/` directories

Options:
- `--root <path>` (default: `.`)
- `--name <kebab-case>` (default: root directory name)

Behavior:
- an existing `boundra.config.json` fails with `PROJECT-003`
- existing directories are preserved
- generated config is immediately accepted by `check-boundaries`

### 1.1 `create-domain <name>`

Create a domain scaffold.

Input rules:
- `<name>` must be `kebab-case`.

Generated targets:
- `domains/<name>/client/`
- `domains/<name>/server/`
- `domains/<name>/shared/`
- `domains/<name>/mcp/`
- `domains/<name>/tests/`
- `domains/<name>/domain.json`

Options:
- `--root <path>` (default: `.`)

### 1.2 `check-boundaries`

Analyze imports and detect boundary rule violations.

Options:
- `--root <path>` (default: `.`)
- `--format text|json` (default: `text`)

### 1.3 `graph-domains`

Generate a domain dependency graph from domain manifests.

Options:
- `--root <path>` (default: `.`)
- `--format mermaid|json|dot` (default: `mermaid`)
- `--output <path>`

### 1.4 `generate route <domain>/<name>`
### 1.5 `generate query <domain>/<name>`
### 1.6 `generate mutation <domain>/<name>`

Generate contract-centered template files.

Input rules:
- `<domain>` and `<name>` must be `kebab-case`.
- `<domain>` must already exist.

Options:
- `--root <path>` (default: `.`)

Side effects:
- Generated shared contract files are appended to `domains/<domain>/domain.json` under `publicApi.shared`.
- Generated shared contracts are exported from `domains/<domain>/shared/public.ts`.
- Existing `publicApi.shared` entries are preserved and duplicate entries are not added.
- Generated contracts contain input/result schemas and inferred types.
- Generated route files bind a validated server implementation.
- Generated query/mutation files expose framework-neutral async client functions.

### 1.7 `add-dependency <domain>/<dependency>`

Declare that one domain depends on another domain.

Input rules:
- both names must be `kebab-case`
- both domains must already exist
- a domain cannot depend on itself

Options:
- `--root <path>` (default: `.`)

Side effects:
- append `<dependency>` to `<domain>/domain.json` `dependsOn`
- preserve existing dependencies
- do not add duplicates

## 2. CLI UX Principles

- Boundary diagnostics include rule code, file, line, import, message, and suggestion.
- Successful scaffolding output includes one summary line plus created paths.
- JSON output keeps a stable schema for CI and AI parsing.
- CLI failures include a stable code, context, and actionable suggestion.
- When JSON is requested, project/config failures are emitted as JSON.

## 3. Machine-Readable Output (JSON)

```json
{
  "status": "failed",
  "violations": [
    {
      "rule": "BR-001",
      "file": "domains/order/client/use-order.ts",
      "line": 12,
      "import": "domains/order/server/order-service",
      "message": "client layer cannot import server layer",
      "suggestion": "move shared contract to shared layer or call through an API boundary"
    }
  ],
  "meta": {
    "command": "check-boundaries",
    "violation_count": 1
  }
}
```
