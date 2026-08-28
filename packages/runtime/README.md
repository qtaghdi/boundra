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
- `createHttpTransport`
- `BoundraHttpError` with bounded structured non-2xx response details
- per-call `BoundraCallOptions` with `AbortSignal` propagation
- `implementRoute`, `implementQuery`, `implementMutation`
- `executeContract`
- `BoundraRuntimeError`
- `defineConfig` and `BoundraConfig` from `boundra/config`
- `createBoundraErrorView` from `boundra/ui`

Configuration-aware TypeScript tooling can use the public config contract:

```ts
import { defineConfig } from "boundra/config";

const config = defineConfig({
  paths: {
    domains: "domains",
  },
});
```

The native CLI still discovers `boundra.config.json`. The `boundra/config`
subpath provides authoring types and helpers without requiring the Rust CLI to
execute JavaScript or TypeScript configuration files. See
`docs/config-authoring.md` in the repository for the supported surface.

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

Query and mutation calls accept optional per-call cancellation:

```ts
const controller = new AbortController();
await client.query(getTaskQuery, input, { signal: controller.signal });
```

The signal reaches custom transports and `createHttpTransport`. If the request
is intentionally aborted, the original cancellation error is preserved.

See the repository documentation for CLI installation, contract generation,
and host integration. Browser error view examples for Vite, Next.js, and
handled errors are documented in `docs/error-view.md`.
