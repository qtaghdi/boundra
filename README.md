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
  auth/
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
- 네이밍 규약: [docs/naming-convention.md](docs/naming-convention.md)
- 도메인 매니페스트 명세: [docs/domain-manifest-spec.md](docs/domain-manifest-spec.md)
- 로드맵: [docs/roadmap.md](docs/roadmap.md)
- 1인 개발 컨벤션: [docs/solo-convention.md](docs/solo-convention.md)
- 용어 사전: [docs/glossary.md](docs/glossary.md)
- ADR: [docs/adr](docs/adr)
- 기여 가이드: [CONTRIBUTING.md](CONTRIBUTING.md)

## Current Status

- 저장소 초기화 단계
- 문서 기조 및 규약 선정 완료
- Rust 워크스페이스 및 `check-boundaries` MVP( BR-001 / BR-002 ) 스캐폴드 완료

## Quick Start (MVP)

사전 요구사항:
- Rust toolchain (`cargo`, `rustc`)

실행:

```bash
cargo run -p boundra-cli -- check-boundaries
```

JSON 출력:

```bash
cargo run -p boundra-cli -- check-boundaries --format=json
```

프로젝트 설정:

```bash
cat boundra.config.json
```
