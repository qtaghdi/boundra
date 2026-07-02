# ADR 0003: Public Packaging

- Status: Superseded by ADR 0004
- Date: 2026-07-01

## Context

Boundra's committed example runs from workspace source files and invokes the CLI
through `cargo run`. That proves framework behavior but does not prove that a
user can install or execute published artifacts outside the repository.

The runtime and Rust CLI have different distribution needs. Combining both in
one JavaScript package would add a native-binary download layer before the CLI
interface and target matrix have stabilized.

## Decision

- Publish `@boundra/runtime` as a compiled ESM npm package with declarations.
- Keep Zod out of the runtime dependency graph; generated applications install
  Zod because generated contract source imports it directly.
- Distribute the `boundra` CLI as native archives attached to GitHub Releases.
- Support `cargo install --git <repository> boundra-cli` as a source
  installation fallback during the public preview.
- Do not publish the internal Rust crates to crates.io in the first preview.
- Synchronize runtime and CLI preview versions under one `v<semver>` release
  tag, beginning with `v0.1.0`.
- Produce SHA-256 checksums for native archives.
- Defer an npm-based native CLI wrapper until binary names, update behavior, and
  supported targets have stabilized through at least one preview release.

## Consequences

Positive:

- TypeScript consumers receive normal JavaScript and declaration artifacts
- CLI users do not need Node.js when using native binaries
- the first release avoids maintaining a binary downloader and postinstall hook
- clean-room tests can validate npm and native artifacts independently

Negative:

- installation instructions differ for runtime and CLI
- GitHub Release automation must build and archive multiple targets
- npm scope ownership must be confirmed before the first publish

## Follow-up

- add a compiled runtime package and package-content checks
- add native release archives for the supported target matrix
- verify a fresh project using only packed/published artifacts
- revisit an npm CLI wrapper after preview feedback
