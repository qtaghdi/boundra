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
    "includeExtensions": ["ts", "tsx", "js", "jsx"],
    "ignore": [
      "**/node_modules/**",
      "**/dist/**",
      "**/build/**",
      "**/coverage/**",
      "**/target/**"
    ]
  }
}
```

Supported:

- `includeExtensions`: file extensions scanned by the parser
- `ignore`: simple workspace-relative ignore patterns

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
    "includeExtensions": ["ts", "tsx", "js", "jsx"],
    "ignore": [
      "**/node_modules/**",
      "**/dist/**",
      "**/build/**",
      "**/coverage/**",
      "**/target/**"
    ]
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

## 6. Not Yet Supported

The following fields are planned or possible later, but are not implemented in the current CLI:

- `version`
- `naming`
- `rules`
- `codegen`
- `graph`
- custom exit codes
- custom diagnostic messages
