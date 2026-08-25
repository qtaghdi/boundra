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
- Show every normalized runtime issue and boundary diagnostic instead of
  truncating the view to the first failure.
- Keep the safe diagnostic identity visible (`code`, `contract`, `phase`, field
  path, file, and line) and provide a copyable text summary for debugging and
  incident reports.
- Preserve runtime errors when a boundary re-check becomes clean; each source
  owns its own overlay state so one source cannot accidentally clear another.
- Support keyboard dismissal and responsive, accessible navigation between
  multiple diagnostics.
- Run `check-boundaries --format json` at dev-server start and on source updates,
  then send violations through Vite's custom HMR event channel.
- Clear boundary diagnostics after a clean re-check. Runtime diagnostics remain
  visible until explicitly dismissed because the overlay cannot infer when a
  handled application state has recovered.
- Do not include or activate the overlay in production builds.
- Do not render contract inputs, results, provider error objects, or causes in
  the overlay. The normalized diagnostic shape remains the privacy boundary.
- Keep the transport protocol small enough for a future Next adapter to reuse.

## Consequences

- Vite users get immediate field-level runtime and boundary feedback.
- Applications retain ownership of handled production errors.
- Boundary checks add development-time work on file changes; future profiling
  determines whether debounce or incremental scanning is necessary.
