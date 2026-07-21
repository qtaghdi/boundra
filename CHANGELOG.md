# Changelog

All notable changes to Boundra are documented in this file.

The format follows Keep a Changelog and versions follow Semantic Versioning.

## [Unreleased]

### Added

- BR-006 enforcement for cross-domain public imports missing from `dependsOn`
- Svelte scanning and Vite hot-update diagnostics by default
- relative `tsconfig extends` alias resolution with `baseUrl` support

### Changed

- apply configured app, domain, and package roots throughout boundary analysis
- reject cyclic domain dependency graphs and prevent cycle-producing dependency updates

## [0.1.2] - 2026-07-03

### Added

- BR-005 enforcement for app imports that bypass a domain's declared public API
- provider-neutral validation issues and safe JSON serialization for runtime errors
- a development-only Vite overlay for runtime and boundary diagnostics
- `boundra init` and a checksummed native CLI launcher in the npm package
- a framework-neutral HTTP transport and CRUD resource contract generator
- reproducible 1k/10k-file boundary benchmarks with a CI regression smoke gate

## [0.1.1] - 2026-07-02

### Added

- schema-backed route, query, and mutation contracts
- framework-neutral TypeScript client and server runtime
- BR-001 through BR-004 boundary analysis
- domain scaffolding, dependency graphing, and code generation
- structured CLI diagnostics and machine-readable boundary output
- clean-room packaging verification

### Changed

- publish the TypeScript runtime through the single `boundra` npm package
- distribute the native CLI through checksummed GitHub Release archives
- generate contracts that import runtime APIs from `boundra`

## [0.1.0] - 2026-07-01

Accidental workspace snapshot. This version has no supported runtime exports or
CLI entry point and should not be installed.
