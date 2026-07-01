# Packaging Spec

## 1. Release Units

### `@boundra/runtime`

Public npm package containing:

- ESM JavaScript under `dist/`
- TypeScript declarations under `dist/`
- package metadata and README
- no TypeScript source dependency at runtime

The package must not publish tests, workspace configuration, or dogfood files.

### `boundra` CLI

Native archives attached to a GitHub Release:

- `boundra-<version>-x86_64-unknown-linux-gnu.tar.gz`
- `boundra-<version>-x86_64-apple-darwin.tar.gz`
- `boundra-<version>-aarch64-apple-darwin.tar.gz`
- `boundra-<version>-x86_64-pc-windows-msvc.zip`
- `checksums-sha256.txt`

Each archive contains the `boundra` executable, `LICENSE`, and a short install
README.

## 2. Supported Toolchains

- Node.js: 20, 22, and 24
- package manager used by this repository: pnpm 11
- Rust source installation: current stable toolchain
- CLI release targets: Linux x64, macOS x64/arm64, Windows x64

The runtime is package-manager agnostic after publication.
Runtime compatibility jobs use npm so the pnpm 11 development-tool requirement
does not incorrectly raise the runtime's minimum Node.js version.

## 3. Runtime Package Contract

`packages/runtime/package.json` must provide:

- `type: module`
- conditional `exports` with `types` and `import`
- `files: ["dist", "README.md", "LICENSE"]`
- `sideEffects: false`
- `engines.node >= 20`
- public access metadata

`pnpm --filter @boundra/runtime pack` must contain only the intended public
files, and a fresh project must be able to import every public runtime export.

## 4. CLI Contract

The release binary must:

- run `boundra --help` without repository files
- create domains and generate route/query/mutation files
- emit the same diagnostic and JSON schemas tested in the workspace
- report its version through `boundra --version`

## 5. Clean-Room Gate

A temporary project outside the repository must:

1. install the packed runtime artifact, Zod, and TypeScript
2. execute the release-mode Boundra CLI binary
3. create `order` and `billing`
4. add `billing -> order`
5. generate one route, query, and mutation
6. type-check generated code against the packed runtime
7. pass `check-boundaries --format json`
8. produce the expected domain graph

## 6. Versioning

- preview version starts at `0.1.0`
- breaking generated-code, diagnostic, or manifest changes increment the minor
  version while the project remains below `1.0.0`
- release tags use `v<version>`
- runtime package and CLI version must match the release tag
