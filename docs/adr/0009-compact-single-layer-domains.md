# ADR 0009: Compact Layout for Single-Layer Domains

- Status: Accepted
- Date: 2026-07-21

## Context

The canonical Boundra layout gives every domain explicit `client`, `server`,
and `shared` directories. That structure is valuable when a domain crosses
runtime boundaries, but it adds nesting for small domains that expose exactly
one layer. Applications then end up with paths such as
`comparison/client/public.ts` even though the domain has no server or shared
surface.

Moving those files to the domain root without framework support is unsafe.
The rule engine currently classifies only named layer directories, so a root
file would become `Unknown` and lose client/server/shared enforcement.

## Decision

- A domain may use a compact root layout when exactly one of
  `publicApi.client`, `publicApi.server`, or `publicApi.shared` is non-empty.
- Direct files under that domain root inherit the single declared public API
  layer for boundary analysis.
- Explicit `client`, `server`, `shared`, `mcp`, and `tests` directories always
  keep their existing meaning.
- Nested root directories do not inherit the compact layer. This keeps the
  compact form intentionally small and prevents ambiguous hidden structures.
- Domains with more than one public API layer continue to use explicit layer
  directories; their root files remain `Unknown`.
- The manifest remains the source of truth. A compact public entrypoint must be
  declared explicitly, for example `publicApi.client: ["./public.ts"]`.

## Consequences

Positive:

- single-layer domains can remove redundant directory nesting
- BR-001, BR-002, and BR-003 remain active for compact root files
- application and cross-domain imports still go through manifest-declared
  public APIs
- multi-runtime domains retain the more explicit canonical structure

Negative:

- compact domains must move tests into `tests/` if they should not inherit the
  production layer
- adding a second runtime layer requires migrating root implementation files
  into explicit layer directories
- code generation continues to target the canonical shared layout

## Migration

1. Confirm that only one public API category is non-empty.
2. Move direct implementation and public files to the domain root.
3. Move tests to `tests/`.
4. Update the manifest public path to `./public.ts`.
5. Update consumers to import the compact public entrypoint.
6. Run `boundra check-boundaries`.
