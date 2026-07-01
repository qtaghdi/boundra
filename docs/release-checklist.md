# Public Preview Release Checklist

## Code and Contracts

- [x] BR-001 through BR-004 are implemented and tested
- [x] schema-backed generated contracts are runtime validated
- [x] CLI diagnostics include stable codes and recovery suggestions
- [x] manifest, diagnostic, contract, and packaging specs are documented
- [x] generated-code breaking decisions are recorded in ADR 0002
- [x] packaging direction is recorded in ADR 0003

## Artifact Verification

- [x] runtime builds to ESM JavaScript and declarations
- [x] runtime tarball contains only public package files
- [x] release CLI reports `--version`
- [x] offline clean-room installation passes outside the repository
- [ ] `cargo install --git` source fallback passes with registry access
- [x] online clean-room installation passes in GitHub Actions
- [x] Node.js 20, 22, and 24 runtime jobs pass
- [ ] Linux, macOS Intel/arm64, and Windows CLI jobs pass
- [ ] release workflow dry-run produces all four CLI archives and runtime tarball

## Repository Readiness

- [x] MIT `LICENSE`
- [x] `SECURITY.md`
- [x] `CHANGELOG.md`
- [x] public quickstart and CLI install guide
- [x] Node and native target support matrix
- [x] npm trusted-publishing workflow with provenance is configured
- [ ] merge the framework and release-packaging pull requests
- [x] enable GitHub private vulnerability reporting

## Registry and Release

- [ ] confirm ownership of the `@boundra` npm scope
- [ ] configure npm trusted publishing or a scoped automation token
- [ ] publish `@boundra/runtime@0.1.0`
- [ ] verify the public npm package in a fresh project
- [ ] create and push the signed `v0.1.0` tag
- [ ] verify GitHub Release checksums and installation instructions

No public release should be announced while any unchecked item in Artifact
Verification, Repository Readiness, or Registry and Release remains.
