# Config Authoring

Boundra exposes the currently supported configuration contract through the
`boundra/config` npm subpath.

```ts
import { defineConfig, type BoundraConfig } from "boundra/config";

const config = defineConfig({
  paths: {
    apps: "apps",
    domains: "domains",
    packages: "packages",
    crates: "crates",
  },
  checkBoundaries: {
    capabilities: {
      external: {
        react: ["ui"],
        "@prisma/client": ["database"],
        "node:*": ["runtime"],
      },
      packages: {
        ui: ["ui"],
        db: ["database"],
        infra: ["runtime"],
      },
      apps: ["runtime"],
    },
    policy: {
      shared: {
        denyCapabilities: ["ui", "database", "runtime"],
      },
    },
  },
});

const typedConfig: BoundraConfig = config;
```

`defineConfig()` is intentionally an identity helper. It preserves literal
inference and checks the object against the public TypeScript contract, while
the native CLI remains responsible for filesystem-aware validation such as
relative paths, domain-root existence, public API paths, and capability matcher
rules.

## Canonical Config File

The native CLI continues to discover only:

```txt
boundra.config.json
```

`boundra.config.ts` is not a supported project entry point in this slice. Boundra
does not make the Rust CLI execute JavaScript or TypeScript merely to load
configuration, because that would make native CLI behavior depend on an
additional runtime.

The `boundra/config` surface is intended for framework adapters, editor tooling,
config generators, tests, and other TypeScript code that needs to construct or
share a Boundra configuration value without duplicating the schema.

## Supported TypeScript Fields

The public type surface mirrors the fields currently consumed by the Rust
project model:

- `project.workspaceRoot`
- `paths.apps`
- `paths.domains`
- `paths.packages`
- `paths.crates`
- `domain.manifestFile`
- `domain.publicApi.client`
- `domain.publicApi.server`
- `domain.publicApi.shared`
- `checkBoundaries.includeExtensions`
- `checkBoundaries.ignore`
- `checkBoundaries.capabilities.external`
- `checkBoundaries.capabilities.packages`
- `checkBoundaries.capabilities.apps`
- `checkBoundaries.policy.shared.denyCapabilities`

Planned config fields are deliberately excluded from `BoundraConfig` until the
CLI consumes them. This prevents TypeScript from accepting configuration that
Boundra would silently ignore.

See `docs/config-spec.md` for defaults and native validation rules.
