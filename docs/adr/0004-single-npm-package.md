# ADR 0004: Single npm Package Identity

- Status: Accepted
- Date: 2026-07-02
- Supersedes: ADR 0003 package naming decision

## Context

ADR 0003 separated the TypeScript runtime into `@boundra/runtime` while
reserving the unscoped `boundra` name for a possible future CLI installer.
Before the first usable release, `boundra@0.1.0` was accidentally published
from the workspace root. That artifact contains a repository snapshot and has
no runtime exports or CLI entry point.

Publishing a second npm package would also make the public installation and
generated imports more complex without providing a user-facing benefit.

## Decision

- Publish the compiled TypeScript runtime as the unscoped `boundra` package.
- Keep the workspace root private and name it `boundra-workspace` so it cannot
  collide with the public workspace package.
- Generated TypeScript imports use `boundra` directly.
- Do not publish `@boundra/runtime`.
- Treat `boundra@0.1.1` as the first usable npm release because npm package
  versions are immutable and `0.1.0` cannot be replaced.
- Continue distributing the Rust CLI as native GitHub Release archives with a
  `cargo install --git` fallback.
- Keep runtime and CLI versions synchronized under `v<semver>` tags.
- Continue deferring an npm native-CLI wrapper until the binary distribution
  contract has stabilized.

## Consequences

Positive:

- users install `boundra` and Zod instead of learning an internal package split
- generated code imports the product package name
- the existing npm trusted-publisher configuration for `boundra` can be reused
- the runtime tarball remains small and does not include repository files

Negative:

- `boundra` on npm provides runtime APIs but not the native CLI during preview
- `boundra@0.1.0` must remain deprecated as an unusable historical artifact
- all pre-release generated imports must migrate from `@boundra/runtime`

## Follow-up

- publish and verify `boundra@0.1.1` through OIDC
- publish matching native CLI archives in the `v0.1.1` GitHub Release
- reconsider a single-command npm CLI install after preview feedback
