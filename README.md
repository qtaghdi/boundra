# Boundra

도메인 중심 풀스택 개발을 강제하는 TypeScript 프레임워크 (Rust 기반 툴링 엔진)

## Why Boundra

Boundra는 "모든 걸 대체하는 프레임워크"가 아닙니다. 구조와 경계를 강제해서, 팀 규모가 커져도 코드베이스 일관성을 유지하도록 돕는 플랫폼입니다.

해결하려는 핵심 문제:
- 도메인 단위 분리가 되지 않아 변경 영향 범위가 불명확한 문제
- 모노레포에서 import 자유도로 경계가 붕괴되는 문제
- 풀스택 개발에서 구조 일관성이 빠르게 깨지는 문제

핵심 가치:
- 구조 강제를 통한 유지보수 비용 절감
- AI가 이해하고 작업하기 쉬운 코드베이스
- 서버/클라이언트/타입/정책 일관성
- 모노레포 확장성

## Non-Goals

초기에는 아래 영역을 직접 만들지 않습니다.
- JS 런타임
- 번들러
- UI 프레임워크
- ORM
- 상태관리
- 테스트 러너
- Next.js 대체

## Design Principles

1. 앱보다 도메인이 먼저다
2. shared는 최소화한다
3. 경계는 규칙이 아니라 도구로 강제한다
4. 타입 공유보다 계약(shared contract)을 우선한다
5. 자유보다 안전한 제약을 제공한다
6. AI가 이해하기 쉬운 구조를 만든다
7. MCP는 코어가 아니라 확장 레이어다

## Target Structure

```txt
apps/
  web/
  admin/

domains/
  <domain-name>/
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

## V1 Scope

반드시 포함:
- `create-domain`
- `check-boundaries`
- `graph-domains`
- schema 기반 계약
- route/query/mutation 코드 생성

## Documentation Map

- 프로젝트 헌장: [docs/project-charter.md](docs/project-charter.md)
- 아키텍처: [docs/architecture.md](docs/architecture.md)
- 경계 규칙: [docs/boundary-rules.md](docs/boundary-rules.md)
- CLI 명세(v1): [docs/cli-spec-v1.md](docs/cli-spec-v1.md)
- 설정 명세: [docs/config-spec.md](docs/config-spec.md)
- 프레임워크 표면: [docs/framework-surface.md](docs/framework-surface.md)
- CI 연동: [docs/ci-integration.md](docs/ci-integration.md)
- 네이밍 규약: [docs/naming-convention.md](docs/naming-convention.md)
- 도메인 매니페스트 명세: [docs/domain-manifest-spec.md](docs/domain-manifest-spec.md)
- 계약 스키마 명세: [docs/contract-schema-spec.md](docs/contract-schema-spec.md)
- 진단 명세: [docs/diagnostic-spec.md](docs/diagnostic-spec.md)
- 패키징 명세: [docs/packaging-spec.md](docs/packaging-spec.md)
- 공개 퀵스타트: [docs/quickstart.md](docs/quickstart.md)
- CLI 설치: [docs/cli-install.md](docs/cli-install.md)
- 릴리스 체크리스트: [docs/release-checklist.md](docs/release-checklist.md)
- 로드맵: [docs/roadmap.md](docs/roadmap.md)
- 1인 개발 컨벤션: [docs/solo-convention.md](docs/solo-convention.md)
- 용어 사전: [docs/glossary.md](docs/glossary.md)
- ADR: [docs/adr](docs/adr)
- 기여 가이드: [CONTRIBUTING.md](CONTRIBUTING.md)

## Current Status

- Repository bootstrap is complete.
- Core documentation and conventions are drafted.
- Rust workspace and `check-boundaries` MVP are available for BR-001 through BR-004.
- `create-domain`, `graph-domains`, and initial `generate` workflows are available.
- `packages/runtime` provides the first TypeScript helper surface for generated contracts.
- Generated route/query/mutation contracts are registered in `domain.json` `publicApi.shared`.
- Generated contracts use runtime schemas and inferred TypeScript types.
- `@boundra/runtime` validates client transports and server handlers.
- `add-dependency` manages domain graph declarations without manual JSON edits.
- CLI failures include stable codes, context, suggestions, and JSON output where requested.
- `examples/order-billing` is a committed two-domain generated-contract flow.
- `pnpm verify-example` repeats TypeScript, runtime, Rust, boundary, and graph validation.
- CLI fixture tests cover text/json output, boundary behavior, scaffolding, graph output, code generation, and manifest updates.

## Quick Start (MVP)

사전 요구사항:
- Rust toolchain (`cargo`, `rustc`)
- Node.js 24 and pnpm 11

예제 실행:

```bash
pnpm example:order-billing
cargo run -p boundra-cli -- check-boundaries --root examples/order-billing
cargo run -p boundra-cli -- graph-domains --root examples/order-billing --format mermaid
```

전체 예제 검증:

```bash
pnpm install
pnpm verify-example
```

Run against an explicit project root:

```bash
cargo run -p boundra-cli -- check-boundaries --root examples/order-billing
```

JSON 출력:

```bash
cargo run -p boundra-cli -- check-boundaries --root examples/order-billing --format json
```

CI 실행:

```bash
pnpm verify-example
```
