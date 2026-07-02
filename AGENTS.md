# AGENTS.md

Boundra 프로젝트에서 작업하는 사람/AI 에이전트 공통 운영 규약.

## 1. Mission

Boundra는 도메인 중심 풀스택 개발을 강제하는 TypeScript 프레임워크이며, Rust 기반 툴링 엔진으로 경계를 정적 분석한다.

핵심 목표:
- 구조 강제
- 경계 자동 차단
- 계약 중심 일관성
- AI 친화 코드베이스

## 2. Source of Truth

아래 문서가 우선 순위를 가진다.
1. `docs/project-charter.md`
2. `docs/architecture.md`
3. `docs/boundary-rules.md`
4. `docs/cli-spec-v1.md`
5. `docs/domain-manifest-spec.md`
6. `docs/naming-convention.md`

충돌 시 더 상위 문서를 따른다.

## 3. Non-Goals (초기)

아래는 구현하지 않는다.
- 자체 JS 런타임
- 자체 번들러
- 자체 UI 프레임워크
- 자체 ORM
- 자체 상태관리
- 자체 테스트 러너
- Next.js 대체

## 4. Domain-First Rules

기본 구조:

```txt
apps/
domains/<domain>/{client,server,shared,mcp,tests}
packages/
crates/
```

레이어 책임:
- `client`: UI/hooks/client logic
- `server`: API/business logic
- `shared`: schema/types/contracts (pure)
- `mcp`: AI adapter layer
- `tests`: 도메인 테스트

## 5. Boundary Rules (Hard Constraints)

절대 규칙:
- `client -> server` import 금지 (BR-001)
- `server -> client` import 금지 (BR-002)
- `shared -> UI/DB/infra runtime` 의존 금지 (BR-003)
- 도메인 간 내부 경로 직접 import 금지, public API만 허용 (BR-004)
- 앱에서 도메인 내부 경로 직접 import 금지, public API만 허용 (BR-005)

허용:
- `client -> shared`
- `server -> shared`
- `domain -> other-domain public API`
- `app -> domain public API`

## 6. Agent Working Agreement

모든 작업자는 아래를 지킨다.
- 문서 먼저, 코드 나중 (`Docs First Rule`)
- 경계 예외는 코드로 우회하지 않고 ADR로 기록
- "빠른 구현"보다 "유지되는 구조"를 우선
- 코드 생성/수정 시 도메인 경계를 먼저 확인
- 추측 구현 금지: 규격 불명확하면 문서에 먼저 명시

## 7. CLI-First Workflow

기본 워크플로우:
1. `create-domain <name>`
2. 기능 구현
3. `check-boundaries`
4. 필요 시 `graph-domains`로 영향 분석
5. PR 전에 규칙/문서/테스트 확인

현재 MVP 기준 필수 커맨드:
- `check-boundaries`

## 8. Implementation Priority (MVP)

1. `check-boundaries` 안정화 (BR-001/002 -> 003/004 확장)
2. `create-domain` 스캐폴드 자동화
3. `graph-domains`
4. `generate route/query/mutation`

## 9. Code & Review Standards

- 변경은 작고 독립적으로 유지
- 진단 메시지는 rule code + file + line + suggestion 포함
- Machine-readable 출력(JSON)은 스키마 안정성 유지
- 공유 계층(`shared`) 순수성 훼손 금지
- 리뷰는 "기능 동작"보다 "경계/계약 위반 리스크"를 먼저 본다
- 파일/폴더/브랜치/CLI 리소스 이름은 `kebab-case only`를 적용한다
- 커밋 메시지는 `<type>(<scope>): <message>` 형식을 사용한다. 예: `config(domain): add commit convention`
- 커밋 `type`과 `scope`는 `kebab-case only`를 적용한다
- 브랜치 이름은 `<type>/<scope>` 또는 `<type>/<scope>-<short-description>` 형식을 사용한다. 예: `config/coderabbit`, `feat/cli-schema-codegen`
- 브랜치 `type`, `scope`, `short-description`은 `kebab-case only`를 적용한다

## 10. PR Checklist

- 변경이 프로젝트 미션/원칙에 부합하는가?
- boundary 규칙을 위반하지 않는가?
- 문서(명세/ADR) 업데이트가 필요한 변경인가?
- 테스트/검증 경로가 포함되었는가?
- 진단/에러 메시지가 사용자 친화적인가?

## 11. ADR Policy

다음 변경은 ADR 필수:
- 아키텍처 방향 변경
- boundary rule 변경
- CLI 인터페이스 breaking change
- manifest schema breaking change

ADR 위치: `docs/adr/`

## 12. Definition of Done

작업 완료 조건:
- 경계 규칙/문서와 일치
- 로컬 검증 커맨드 통과
- 필요한 문서/ADR 반영 완료
- 다음 작업자가 즉시 이어받을 수 있는 상태

## 13. Naming Policy

- 표준: `kebab-case only`
- 상세 규칙: `docs/naming-convention.md`

## 14. Release Notes Policy

- 릴리스 노트는 `docs/releases/v<major>-<minor>-<patch>.md`에 작성한다.
  예: 태그 `v0.1.1`의 노트는 `docs/releases/v0-1-1.md`다.
- 모든 릴리스 노트는 아래 섹션을 순서대로 포함한다.
  1. `# Boundra v<semver>`
  2. `## Release Summary`
  3. `## Highlights`
  4. `## Installation`
  5. `## Breaking Changes`
  6. `## Known Limitations`
  7. `## Verification`
- 릴리스 노트는 커밋 목록이 아니라 사용자 관점의 변화, 설치 방법,
  호환성, 마이그레이션, 알려진 제약을 설명한다.
- breaking change나 known limitation이 없더라도 해당 섹션을 생략하지
  않고 `없음`이라고 명시한다.
- 릴리스 준비 PR은 대상 버전의 릴리스 노트와 `CHANGELOG.md`를 함께
  갱신해야 한다.
- PR CI는 모든 릴리스 노트의 파일명과 필수 섹션을 검증한다.
- `v*` 태그 릴리스는 대응하는 릴리스 노트 파일이 없으면 실패하며,
  GitHub Release 본문은 해당 파일을 그대로 사용한다.
- PR은 기본적으로 ready 상태로 생성한다. 사용자가 명시적으로 요청한
  경우에만 draft PR을 생성한다.
