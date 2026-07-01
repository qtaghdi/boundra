# Architecture

## 1. Workspace Layout

```txt
apps/
  web/
  admin/

examples/
  order-billing/
    domains/
    src/

domains/
  <domain>/
    client/
    server/
    shared/
    mcp/
    tests/

packages/
  runtime/
  ui/
  infra/
  config/
  tooling/

crates/
  core/
  parser/
  rules/
  cli/
```

`apps/` contains deployable product entry points owned by the repository.
`examples/` contains self-contained learning and framework-verification
projects. An example must keep its own Boundra config, TypeScript config, and
domains so it can be checked through an explicit `--root` without treating the
framework repository itself as an application.

## 2. Domain Layers

- `client/`: UI, hooks, browser-side orchestration
- `server/`: API handlers, business rules, data orchestration
- `shared/`: schema, DTO, contract, pure utilities
- `mcp/`: AI adapter layer (tool/resource/prompt)
- `tests/`: domain 테스트 (unit/integration)

## 3. Dependency Direction

허용:
- `client -> shared`
- `server -> shared`
- `domain -> other-domain public API`

금지:
- `client -> server`
- `server -> client`
- `shared -> UI/DB/infra runtime`
- `domain -> other-domain internal path`

## 4. Runtime and Tooling

- 사용자 레벨: TypeScript
- 코어 엔진: Rust
- 브릿지: NAPI-RS (future candidate)
- 빌드/분석: Rust analyzer CLI first; TypeScript build integration later
- 모노레포: pnpm (초기), Nx는 선택적 도입
- 프레임워크 표면: `docs/framework-surface.md`를 따른다.

## 5. Core Crates Responsibility

- `core`: project config, domain manifest, shared diagnostic/domain types
- `parser`: workspace file scanning and import extraction
- `rules`: boundary validation
- `cli`: command parsing, execution, graph output, and code generation

## 6. Package Responsibility

- `packages/runtime`: pure TypeScript helper types used by generated contracts

## 7. Expansion Strategy

MCP는 v3 이후 `domains/<domain>/mcp`에서 확장하며, 코어 로직과 분리한다.

확장 DSL은 실제 dogfooding 이후 검토한다. 코어 확장은 Lua보다 Starlark를 우선 후보로 둔다.
