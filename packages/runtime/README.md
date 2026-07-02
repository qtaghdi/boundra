# Boundra

The npm package for Boundra's schema-backed TypeScript runtime. The native CLI
is distributed separately through GitHub Releases during the public preview.

## Install

```bash
pnpm add boundra zod
```

Initialize and run the native CLI through the same npm package:

```bash
pnpm exec boundra init
pnpm exec boundra check-boundaries
```

The first CLI invocation downloads and verifies the matching native release.
Set `BOUNDRA_CLI_PATH` to an existing binary in offline or controlled environments.

Generated contracts use Zod for schema authoring while the runtime depends only
on a structural `parse(unknown)` contract.

## Public API

- `defineRoute`, `defineQuery`, `defineMutation`
- `createBoundraClient`
- `implementRoute`, `implementQuery`, `implementMutation`
- `executeContract`
- `BoundraRuntimeError`

Validation failures include provider-neutral field issues:

```ts
try {
  await client.mutation(createTaskMutation, input);
} catch (error) {
  if (error instanceof BoundraRuntimeError) {
    console.log(error.code, error.issues[0]?.path, error.issues[0]?.message);
  }
}
```

`error.toJSON()` returns a safe diagnostic shape without the original input or
internal `cause`, suitable for development overlays and application error UIs.

See the repository documentation for CLI installation, contract generation,
and host integration.
