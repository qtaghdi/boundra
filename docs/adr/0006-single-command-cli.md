# ADR 0006: Single-Command npm CLI and Project Initialization

- Status: Accepted
- Date: 2026-07-02
- Extends: ADR 0004

## Context

The first external consumer installed the TypeScript runtime from npm but had
to place the native CLI manually and author its configuration from scratch.
The product name is already a single npm package, so installation should not
expose the runtime/native implementation split.

## Decision

- Add `boundra init` to create a minimal non-destructive workspace config.
- Expose a `boundra` npm `bin` from the existing package.
- Keep the Rust binary as the implementation of CLI commands.
- The npm launcher first honors `BOUNDRA_CLI_PATH` for controlled environments,
  then uses a versioned user cache, and otherwise downloads the matching native
  release archive.
- Verify the archive against `checksums-sha256.txt` before extraction.
- Do not run a networked postinstall script; download occurs only when the user
  invokes the CLI.
- Keep GitHub release archives and `cargo install` as supported alternatives.

## Consequences

- `pnpm dlx boundra init` becomes the default first-run flow.
- The npm tarball remains cross-platform and small.
- First CLI invocation requires network access unless a binary is already
  cached or `BOUNDRA_CLI_PATH` is set.
- Unix requires `tar`; Windows extraction uses PowerShell.

## Follow-up

- test the launcher across the release platform matrix
- surface checksum, platform, download, and extraction failures with concise
  `[BOUNDRA_CLI]` diagnostics
