# Boundra

The npm package for Boundra's schema-backed TypeScript runtime. The native CLI
is distributed separately through GitHub Releases during the public preview.

## Install

```bash
pnpm add boundra zod
```

Generated contracts use Zod for schema authoring while the runtime depends only
on a structural `parse(unknown)` contract.

## Public API

- `defineRoute`, `defineQuery`, `defineMutation`
- `createBoundraClient`
- `implementRoute`, `implementQuery`, `implementMutation`
- `executeContract`
- `BoundraRuntimeError`

See the repository documentation for CLI installation, contract generation,
and host integration.
