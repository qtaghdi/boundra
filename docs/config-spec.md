# Boundra Config Spec (Draft)

## 1. File Location

- 루트 파일: `boundra.config.json`

## 2. Purpose

- 프로젝트 구조 경로를 표준화한다.
- boundary rule 동작/레벨을 선언한다.
- CLI 동작(`check-boundaries`, `graph-domains`, `codegen`) 기본값을 제공한다.

## 3. Top-level Fields

- `version`: 설정 버전 (현재 `1`)
- `project`: 프로젝트 메타정보
- `paths`: 모노레포 주요 루트 경로
- `domain`: 도메인 레이어/manifest/public API 기본값
- `naming`: 네이밍 정책 (`kebab-case` 강제)
- `rules`: BR-001~004 활성화 및 메시지
- `checkBoundaries`: 검사 대상/출력/종료코드
- `codegen`: 템플릿 경로
- `graph`: 그래프 출력 형식

## 4. Validation Rules

- `paths.domains`는 존재해야 한다.
- `domain.layers`는 `client/server/shared/mcp/tests`를 모두 포함해야 한다.
- `rules`는 최소 BR-001, BR-002를 활성화해야 한다.
- `checkBoundaries.exitCodes`는 서로 다른 정수여야 한다.

## 5. Backward Compatibility

- `version`이 증가할 때는 이전 버전 마이그레이션 가이드를 제공한다.
- breaking 변경은 minor가 아닌 major 릴리스에서만 허용한다.

## 6. Initial Adoption Note

현재 CLI MVP는 BR-001/BR-002를 하드코딩 검사하며, 이 설정 파일은 다음 단계에서 파서에 연동한다.
