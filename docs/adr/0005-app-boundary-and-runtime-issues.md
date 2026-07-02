# ADR 0005: App Public Boundaries and Structured Runtime Issues

- Status: Accepted
- Date: 2026-07-02

## Context

The first external CRUD consumer proved that domain-to-domain boundaries and
schema-backed runtime contracts work, but it exposed two gaps.

First, BR-004 only evaluates imports whose source is another domain. An app can
therefore bypass a domain's manifest and import server or internal files
directly. This makes the composition root a route around the architecture.

Second, `BoundraRuntimeError` keeps a schema provider error only as `cause` and
embeds its string representation in `message`. Applications and future dev
overlays cannot reliably render a field path and actionable reason without
depending on Zod internals.

## Decision

- Introduce BR-005: app code may import a domain only through a path declared
  in that domain's `publicApi` manifest.
- Resolve the configured apps directory in the boundary context instead of
  assuming it is always named `apps`.
- Preserve BR-004 for domain-to-domain access so rule identities remain stable.
- Add provider-neutral validation issues to `BoundraRuntimeError`.
- Normalize only `code`, `path`, and `message` from structurally compatible
  schema errors.
- Add a safe `toJSON()` representation that excludes the original input and
  internal cause.
- Keep production error rendering application-owned. Future Vite and Next
  development adapters consume the same structured error shape.

## Consequences

Positive:

- applications can no longer couple themselves to domain internals
- runtime errors can power friendly field errors and development overlays
- adapters do not need a direct Zod dependency
- serialized diagnostics avoid leaking raw input by default

Negative:

- existing applications that import domain internals will fail BR-005
- schema providers without a structured `issues` array expose an empty list
- the rule engine context now includes the configured apps directory

## Follow-up

- provide a development-only Vite overlay using the structured error protocol
- add a Next adapter after the Vite integration establishes the contract
- add `boundra init` so public API aliases and checks are configured by default
