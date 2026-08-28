# ADR 0007: Development-Only Error Overlay

- Status: Accepted
- Date: 2026-07-02

## Context

Boundra runtime and CLI diagnostics are structured, but application developers
must build their own view to see them in a browser. Boundra should provide
framework-grade development feedback without becoming a UI framework or
controlling production error pages.

## Decision

- Keep the framework-neutral browser renderer under `packages/ui` and publish
  it from the existing npm package as `boundra/ui`.
- Export `createBoundraErrorView()` for host frameworks to report normalized
  runtime and boundary diagnostics explicitly. The UI source does not depend on
  React, Next.js, Vite, or a schema provider.
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
- Do not auto-install the view in non-Vite hosts. Next.js and other frameworks
  retain control of error boundaries and decide where development diagnostics
  are mounted.
- Do not render contract inputs, results, provider error objects, or causes in
  the overlay. The normalized diagnostic shape remains the privacy boundary.
- Keep the normalized reporter protocol small enough for framework adapters and
  handled application errors to reuse.

## Consequences

- Vite users get immediate field-level runtime and boundary feedback.
- Next.js and other browser hosts can render the same view without depending on
  Vite internals or duplicating Boundra's diagnostic presentation.
- Applications retain ownership of handled production errors.
- Boundary checks add development-time work on file changes; future profiling
  determines whether debounce or incremental scanning is necessary.
