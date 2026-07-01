# Domain Manifest Spec

## 1. File Location

```txt
domains/<domain>/domain.json
```

## 2. Purpose

`domain.json` is the source of truth for a domain's public API and declared domain dependencies.

It is used by:

- `check-boundaries`
- `graph-domains`
- `generate route|query|mutation`
- `add-dependency`
- BR-004 public API validation

## 3. Schema Shape

```json
{
  "$schema": "https://boundra.dev/schemas/domain-manifest.v1.json",
  "name": "billing",
  "version": "0.1.0",
  "publicApi": {
    "client": ["./client/public.ts"],
    "server": ["./server/public.ts"],
    "shared": [
      "./shared/public.ts",
      "./shared/contracts/create-invoice.ts"
    ]
  },
  "dependsOn": ["order"],
  "policies": {
    "allowCrossDomainServerImport": false,
    "allowMcpWrite": false
  }
}
```

## 4. Currently Loaded Fields

The current CLI loads and validates:

- `name`
- `publicApi.client`
- `publicApi.server`
- `publicApi.shared`
- `dependsOn`

Other fields may be present and are preserved when possible, but they are not enforced yet.

## 5. Validation Rules

- `name` must match the domain folder name.
- `dependsOn` entries must reference existing domains.
- public API paths must be relative.
- public API paths must point to existing files.
- public API paths must not expose `internal` paths.

## 6. Code Generation Behavior

`generate route|query|mutation <domain>/<name>` creates a shared contract file:

```txt
domains/<domain>/shared/contracts/<name>.ts
```

After generation, the CLI appends the contract to:

```json
{
  "publicApi": {
    "shared": ["./shared/contracts/<name>.ts"]
  }
}
```

Existing `publicApi.shared` entries are preserved. Duplicate entries are not added.

Generation also appends an idempotent export to `shared/public.ts`.

## 7. Dependency Update Behavior

`add-dependency <domain>/<dependency>` appends the dependency to `dependsOn`.
Both domains must exist, self-dependencies are rejected, and repeated calls do
not add duplicates.

## 8. Versioning

Initial schema version:

```txt
v1
```

Schema changes should be additive first. Breaking changes require an ADR.
