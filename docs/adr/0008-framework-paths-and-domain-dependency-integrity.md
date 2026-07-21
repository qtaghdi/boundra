# ADR 0008: Framework Paths and Domain Dependency Integrity

- Status: Accepted
- Date: 2026-07-21

## Context

Boundra exposes `paths.apps`, `paths.domains`, and `paths.packages`, but the
boundary engine still assumes that domains live under `domains/` and that
workspace runtime packages live under `packages/`. Projects using layouts such
as SvelteKit's `src/lib/domains` therefore load manifests successfully while
some import rules silently miss the same files.

The scanner accepts configurable extensions, but its default set and the Vite
hot-update filter exclude `.svelte`. TypeScript aliases are also read only from
the root `tsconfig.json`, even though generated framework configs commonly
place aliases in an extended config.

Finally, `domain.json` records `dependsOn`, but cross-domain imports do not
enforce that declaration and dependency cycles are accepted. The manifest
graph can therefore disagree with the code graph and cannot serve as a reliable
architecture contract.

## Decision

- Resolve domain, app, and workspace package paths from `boundra.config.json`
  throughout the rule engine.
- Include `.svelte` in the default scanner extensions and Vite hot-update
  checks.
- Resolve relative `tsconfig.json` `extends` chains, including JSONC, `baseUrl`,
  and child alias overrides.
- Introduce BR-006: a domain may import another domain's declared public API
  only when the target domain is listed in the source manifest's `dependsOn`.
- Reject cyclic `dependsOn` graphs while loading the project model.
- Prevent `add-dependency` from writing an edge that would create a cycle.

## Consequences

Positive:

- configured framework layouts are checked consistently
- Svelte source changes participate in CLI checks and Vite diagnostics
- generated and extended TypeScript aliases resolve without duplicated config
- the manifest graph accurately describes allowed cross-domain coupling
- dependency cycles fail before they can become architectural deadlocks

Negative:

- projects with undeclared cross-domain public imports start reporting BR-006
- projects with existing dependency cycles must repair their manifests before
  Boundra commands can load the project
- package-based `extends` values remain owned by the TypeScript toolchain until
  Boundra has a package-resolution contract

## Compatibility

The changes are additive for valid projects. The existing BR-001 through
BR-005 identities and diagnostic shapes remain stable. BR-006 adds a new
violation, and cycle validation uses the existing project error exit code `2`.
