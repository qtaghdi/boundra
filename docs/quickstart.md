# Boundra Public Quickstart

## Prerequisites

- Node.js 20 or newer for TypeScript applications
- pnpm, npm, or another package manager

## Install the Runtime

```bash
pnpm add boundra zod
```

The npm package includes the native CLI launcher. Initialize the workspace:

```bash
pnpm exec boundra init --name my-app
```

## Create a Project Flow

```bash
pnpm exec boundra create-domain order
pnpm exec boundra create-domain billing
pnpm exec boundra add-dependency billing/order
pnpm exec boundra generate query order/get-order
pnpm exec boundra generate mutation order/submit-order
pnpm exec boundra generate route billing/create-invoice
pnpm exec boundra check-boundaries --format json
pnpm exec boundra graph-domains --format mermaid
```

Generated contracts start with safe empty Zod objects. Replace their fields
with domain input/result schemas, then implement generated client or server
adapters. Boundra never overwrites an existing generated file.

## CI Gate

```bash
pnpm exec boundra check-boundaries --root . --format json
```

See `docs/contract-schema-spec.md` for contract ownership and
`docs/cli-install.md` for native CLI installation.
