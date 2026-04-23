# Contributing

## 1. Working Agreement

- 문서화된 규칙 없이 구조를 변경하지 않는다.
- 도메인 경계 예외는 반드시 ADR로 남긴다.
- 구현 전에 관련 문서(헌장/아키텍처/규칙)를 먼저 확인한다.

## 2. Branch Strategy

- 기본 브랜치: `develop` (팀 정책에 맞춰 조정 가능)
- 기능 브랜치 권장: `codex/<scope>-<short-desc>`

## 3. Commit Convention (Recommended)

- `feat:` 기능
- `fix:` 버그
- `docs:` 문서
- `refactor:` 리팩터링
- `test:` 테스트
- `chore:` 유지보수

예:
- `feat(rules): add BR-001 client-server import check`
- `docs(architecture): define domain manifest spec`

## 4. Pull Request Checklist

- 변경 목적이 명확한가?
- 도메인 경계를 위반하지 않는가?
- 문서/명세 반영이 되었는가?
- 테스트 또는 검증 방법이 포함되었는가?

## 5. Code Standards

- TypeScript strict 모드 유지
- shared 계층은 순수성 유지 (UI/DB 의존 금지)
- public API 이외 내부 경로 import 금지
- 파일/폴더/브랜치/문서/CLI 리소스 이름은 `kebab-case only` 적용

## 6. Docs First Rule

아래 변경은 코드보다 문서를 먼저 업데이트한다.
- 아키텍처 방향 변경
- boundary 규칙 변경
- CLI 인터페이스 변경
- manifest 스키마 변경

## 7. Naming Policy

- 프로젝트 표준은 `kebab-case only`
- 상세 규칙은 `docs/naming-convention.md`를 따른다.
