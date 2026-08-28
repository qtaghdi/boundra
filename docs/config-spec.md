# Boundra Config Spec

## 1. File Location

Root file:

```txt
boundra.config.json
```

The file is optional. When it is missing, Boundra uses built-in defaults.

## 2. Purpose

`boundra.config.json` defines the project model used by the CLI:

- workspace and package roots
- domain root and manifest file name
- default public API paths for new domains
- scanner extensions and ignored paths for boundary checks
- dependency capability classification and shared-layer policy

## 3. Supported Fields

### `project`

```json
{
  "project": {
    "workspaceRoot": "."
  }
}
```

Supported:

- `workspaceRoot`: relative workspace root path

### `paths`

```json
{
  "paths": {
    "apps": "apps",
    "domains": "domains",
    "packages": "packages",
    "crates": "crates"
  }
}
```

Supported:

- `apps`
- `domains`
- `packages`
- `crates`

All paths must be relative.

### `domain`

```json
{
  "domain": {
    "manifestFile": "domain.json",
    "publicApi": {
      "client": ["./client/public.ts"],
      "server": ["./server/public.ts"],
      "shared": ["./shared/public.ts"]
    }
  }
}
```

Supported:

- `manifestFile`: file name only, not a path
- `publicApi.client`
- `publicApi.server`
- `publicApi.shared`

These defaults are used by `create-domain`.

### `checkBoundaries`

```json
{
  "checkBoundaries": {
    "includeExtensions": ["ts", "tsx", "js", "jsx", "svelte"],
    "ignore": [
      "**/node_modules/**",
      "**/.next/**",
      "**/.turbo/**",
      "**/.claude/worktrees/**",
      "**/dist/**",
      "**/build/**",
      "**/coverage/**",
      "**/target/**"
    ],
    "capabilities": {
      "external": {
        "react": ["ui"],
        "@prisma/client": ["database"],
        "node:*": ["runtime"]
      },
      "packages": {
        "ui": ["ui"],
        "db": ["database"],
        "infra": ["runtime"]
      },
      "apps": ["runtime"]
    },
    "policy": {
      "shared": {
        "denyCapabilities": ["ui", "database", "runtime"]
      }
    }
  }
}
```

Supported:

- `includeExtensions`: file extensions scanned by the parser
- `ignore`: simple workspace-relative ignore patterns
- `capabilities.external`: external import matcher to capability list
- `capabilities.packages`: direct child under `paths.packages` to capability list
- `capabilities.apps`: capabilities assigned to imports that resolve into `paths.apps`
- `policy.shared.denyCapabilities`: capabilities that trigger BR-003 from `shared`

External matchers match an exact package and its subpaths. A single trailing
`*` is supported for prefix families such as `node:*`.

Configured `external` and `packages` entries overlay the built-in defaults.
Assigning an empty list disables that source's default classification. `apps`
and `policy.shared.denyCapabilities` replace their respective defaults.

## 4. Defaults

When no config file is present, Boundra behaves as if the following config exists:

```json
{
  "project": {
    "workspaceRoot": "."
  },
  "paths": {
    "apps": "apps",
    "domains": "domains",
    "packages": "packages",
    "crates": "crates"
  },
  "domain": {
    "manifestFile": "domain.json",
    "publicApi": {
      "client": ["./client/public.ts"],
      "server": ["./server/public.ts"],
      "shared": ["./shared/public.ts"]
    }
  },
  "checkBoundaries": {
    "includeExtensions": ["ts", "tsx", "js", "jsx", "svelte"],
    "ignore": [
      "**/node_modules/**",
      "**/.next/**",
      "**/.turbo/**",
      "**/.claude/worktrees/**",
      "**/dist/**",
      "**/build/**",
      "**/coverage/**",
      "**/target/**"
    ],
    "capabilities": {
      "external": {
        "react": ["ui"],
        "react-dom": ["ui"],
        "next": ["ui", "runtime"],
        "@prisma/client": ["database"],
        "fs": ["runtime"],
        "path": ["runtime"],
        "crypto": ["runtime"],
        "child_process": ["runtime"],
        "stream": ["runtime"],
        "http": ["runtime"],
        "https": ["runtime"],
        "os": ["runtime"],
        "process": ["runtime"],
        "node:*": ["runtime"]
      },
      "packages": {
        "ui": ["ui"],
        "db": ["database"],
        "infra": ["runtime"]
      },
      "apps": ["runtime"]
    },
    "policy": {
      "shared": {
        "denyCapabilities": ["ui", "database", "runtime"]
      }
    }
  }
}
```

## 5. Validation Rules

- configured paths must be relative
- `paths.domains` must exist for project model loading
- `domain.manifestFile` must be a file name
- public API paths must be relative
- public API paths must not expose `internal`
- `checkBoundaries.includeExtensions` must not be empty
- capability names must not be empty
- external capability wildcards must be a single trailing `*`
- workspace capability package keys must be direct child names under `paths.packages`

## 6. TypeScript Path Aliases

When a root `tsconfig.json` exists, Boundra reads `compilerOptions.paths` for
boundary resolution. Relative `extends` chains are resolved from parent to
child, and child aliases override aliases with the same pattern. Alias targets
respect the `baseUrl` of the config that declares them.

Each file is parsed as JSONC, including comments and trailing commas, to match
normal TypeScript configuration syntax. Package-based `extends` values are
ignored until Boundra defines package resolution semantics.

Only `compilerOptions.paths` affects the Boundra project model. Other TypeScript
compiler options remain owned by the TypeScript toolchain.

## 7. Not Yet Supported

The following fields are planned or possible later, but are not implemented in the current CLI:

- `version`
- `naming`
- `rules`
- `codegen`
- `graph`
- custom exit codes
- custom diagnostic messages
