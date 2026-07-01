# CI Integration

## GitHub Actions

Use the committed dogfood flow as the required CI gate. It type-checks the
TypeScript surface, executes runtime and dogfood flows, runs Rust tests, checks
boundaries, and renders the domain graph:

```yaml
name: Boundra

on:
  pull_request:
  push:
    branches:
      - main

jobs:
  verify-dogfood:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: actions/setup-node@v4
        with:
          node-version: 24
      - run: corepack enable
      - run: pnpm install --frozen-lockfile
      - run: pnpm verify-dogfood
```

## Local Verification

Run the same aggregate gate locally:

```bash
pnpm verify-dogfood
```

The individual boundary command remains suitable for repositories that only
adopt Boundra's analyzer.

## Exit Codes

- `0`: no boundary violations
- `1`: boundary violations found
- `2`: config, manifest, or CLI usage error
- `3`: runtime scan or file system error

## JSON Output

CI and agent integrations should prefer JSON output:

```bash
cargo run -p boundra-cli -- check-boundaries --root . --format json
```

Example:

```json
{
  "status": "failed",
  "violations": [
    {
      "rule": "BR-001",
      "file": "domains/order/client/use-order.ts",
      "line": 3,
      "import": "../server/order-service",
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
