# @boundra/runtime

Schema-backed contract primitives used by Boundra-generated TypeScript code.

## Install

```bash
pnpm add @boundra/runtime zod
```

Generated contracts use Zod for schema authoring while the runtime depends only
on a structural `parse(unknown)` contract.

## Public API

- `defineRoute`, `defineQuery`, `defineMutation`
- `createBoundraClient`
- `implementRoute`, `implementQuery`, `implementMutation`
- `executeContract`
- `BoundraRuntimeError`

See the repository documentation for contract generation and host integration.
