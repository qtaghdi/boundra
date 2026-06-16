# CLI Spec v1

## 1. Commands

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
- Existing `publicApi.shared` entries are preserved and duplicate entries are not added.

## 2. CLI UX Principles

- Boundary diagnostics include rule code, file, line, import, message, and suggestion.
- Successful scaffolding output includes one summary line plus created paths.
- JSON output keeps a stable schema for CI and AI parsing.

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
