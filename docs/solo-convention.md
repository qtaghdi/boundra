# Solo Development Convention

Boundra의 1인 개발(Founder Mode) 기준 문서.

## 1. Objective

혼자 개발할 때도 구조 일관성, 판단 기록, 배포 안정성을 유지한다.

핵심 원칙:
- 빠르게 만들되, 나중의 나를 위해 남긴다.
- 추측 구현보다 명세 기반 구현을 우선한다.
- 코드보다 경계와 계약을 먼저 지킨다.

## 2. Daily Workflow (Default)

1. 작업 시작 전
- 오늘 작업 범위를 1~3개로 제한한다.
- 변경 대상 문서/모듈을 먼저 확정한다.

2. 구현 전
- `AGENTS.md`, 관련 명세 문서 확인
- 영향 범위가 크면 먼저 문서 갱신

3. 구현 중
- 도메인 단위로 변경한다.
- 한 번에 하나의 의사결정만 반영한다.

4. 구현 후
- `check-boundaries`
- 테스트/수동 검증
- 변경 이유를 커밋 메시지로 남긴다.

## 3. Scope Control Rules

- 같은 날 `architecture + codegen + rules`를 동시에 크게 건드리지 않는다.
- 새 기능은 "최소 동작" 먼저, 확장은 다음 커밋으로 분리한다.
- 급한 수정이어도 boundary rule 예외를 임시로 허용하지 않는다.

## 4. Branch & Commit Convention (Solo)

브랜치:
- `codex/<area>-<topic>`
- 예: `codex/rules-br003-shared-purity`

커밋:
- 하루에 여러 커밋 허용, 단 의미 단위로 분리
- 권장 prefix: `feat`, `fix`, `docs`, `refactor`, `test`, `chore`

예시:
- `feat(cli): add check-boundaries --format=json`
- `docs(rules): define BR-003 blocked prefixes`
- `test(rules): add client-to-server violation fixture`

## 5. Documentation Discipline

아래 변경은 반드시 문서 선반영 또는 동시 반영:
- boundary rule 변경
- CLI 옵션/출력 변경
- manifest/config schema 변경
- 도메인 구조 규칙 변경

최소 반영 위치:
- `docs/boundary-rules.md`
- `docs/cli-spec-v1.md`
- `docs/domain-manifest-spec.md`
- `docs/config-spec.md`
- 필요 시 `docs/adr/*`

## 6. Decision Logging (Lightweight ADR)

모든 판단을 ADR로 쓰지 않는다. 아래만 ADR 작성:
- 되돌리기 어려운 구조 결정
- 호환성에 영향이 있는 스키마 변경
- 규칙 완화/예외 도입

작성 규칙:
- 10~20줄 내로 짧게
- Context / Decision / Consequence만 작성
- 파일명: `docs/adr/NNNN-<topic>.md`

## 7. Quality Gates (Before Merge/Push)

필수:
- 경계 검사 통과
- 새 규칙/동작에 대한 테스트 또는 재현 경로 존재
- 문서 링크 깨짐 없음

권장:
- JSON 출력 스키마 예시 갱신
- 변경된 규칙 코드에 대한 실패/성공 예제 동시 추가

## 8. AI Pairing Rules

- AI에게 요청할 때: 도메인, 레이어, 출력 형식, 금지사항을 함께 명시한다.
- AI 결과 반영 전: 경계 위반 가능성부터 검토한다.
- 코드 생성 요청은 템플릿/스키마 기준으로 제한한다.

요청 예시 포맷:
```txt
Domain: auth
Layer: server
Task: add login mutation skeleton
Constraints: no client import, shared contract reuse, keep JSON output stable
```

## 9. Naming Convention

- 기본 원칙: `kebab-case only` (파일/폴더/브랜치/CLI 리소스 이름)
- 도메인: 소문자 단수 kebab-case (`auth`, `product`, `order`, `user-auth`)
- 파일: 역할 중심 kebab-case (`login-service.ts`, `public.ts`)
- 규칙 코드: `BR-xxx`
- 설정 버전: 정수 증가 (`version: 1 -> 2`)

상세 규칙: `docs/naming-convention.md`

## 10. End-of-Day Checklist

- 오늘 변경이 미션/원칙과 일치하는가?
- 내일 바로 이어서 작업할 수 있게 맥락이 남아있는가?
- 임시 코드(TODO, 주석, 하드코딩)가 의도적으로 관리되고 있는가?

3줄 로그를 남기는 것을 권장:
- Done:
- Next:
- Risk:
