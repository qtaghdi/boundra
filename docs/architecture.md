# Architecture

## 1. Workspace Layout

```txt
apps/
  web/
  admin/

domains/
  <domain>/
    client/
    server/
    shared/
    mcp/
    tests/

packages/
  ui/
  infra/
  config/
  tooling/

crates/
  core/
  parser/
  analyzer/
  rules/
  codegen/
  graph/
  cli/
```

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
- 브릿지: NAPI-RS
- 빌드/분석: `tsc --build` + Rust analyzer
- 모노레포: pnpm (초기), Nx는 선택적 도입

## 5. Core Crates Responsibility

- `core`: 공통 타입, 진단 포맷
- `parser`: 파일/모듈/manifest 인덱싱
- `analyzer`: import graph, symbol reference
- `rules`: boundary validation
- `codegen`: route/query/mutation 생성
- `graph`: domain dependency graph
- `cli`: 명령어 파싱/실행 진입점

## 6. Expansion Strategy

MCP는 v3 이후 `domains/<domain>/mcp`에서 확장하며, 코어 로직과 분리한다.
