# Boundra Public Quickstart

## Prerequisites

- Node.js 20 or newer for TypeScript applications
- the `boundra` CLI on `PATH`
- pnpm, npm, or another package manager

## Install the Runtime

```bash
pnpm add @boundra/runtime zod
```

## Create a Project Flow

```bash
boundra create-domain order
boundra create-domain billing
boundra add-dependency billing/order
boundra generate query order/get-order
boundra generate mutation order/submit-order
boundra generate route billing/create-invoice
boundra check-boundaries --format json
boundra graph-domains --format mermaid
```

Generated contracts start with safe empty Zod objects. Replace their fields
with domain input/result schemas, then implement generated client or server
adapters. Boundra never overwrites an existing generated file.

## CI Gate

```bash
boundra check-boundaries --root . --format json
```

See `docs/contract-schema-spec.md` for contract ownership and
`docs/cli-install.md` for native CLI installation.
