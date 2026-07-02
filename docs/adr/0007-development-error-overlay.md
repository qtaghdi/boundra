# ADR 0007: Development-Only Error Overlay

- Status: Accepted
- Date: 2026-07-02

## Context

Boundra runtime and CLI diagnostics are structured, but application developers
must build their own view to see them in a browser. Boundra should provide
framework-grade development feedback without becoming a UI framework or
controlling production error pages.

## Decision

- Export a Vite plugin from `boundra/vite`.
- Inject a development-only overlay client through a virtual module.
- Capture unhandled `BoundraRuntimeError` values and display their normalized
  issues.
- Run `check-boundaries --format json` at dev-server start and on source updates,
  then send violations through Vite's custom HMR event channel.
- Clear the overlay when the relevant runtime or boundary error is resolved.
- Do not include or activate the overlay in production builds.
- Keep the transport protocol small enough for a future Next adapter to reuse.

## Consequences

- Vite users get immediate field-level runtime and boundary feedback.
- Applications retain ownership of handled production errors.
- Boundary checks add development-time work on file changes; future profiling
  determines whether debounce or incremental scanning is necessary.
