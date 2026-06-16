# Naming Convention

Boundra 프로젝트의 네이밍 표준은 `kebab-case only`이다.

## 1. Global Rule

- 기본 규칙: 사람이 작성하는 식별 가능한 문자열은 `kebab-case`를 사용한다.
- 금지: `snake_case`, `camelCase`, `PascalCase`를 파일/폴더/브랜치/CLI 리소스 이름에 사용하지 않는다.

적용 대상:
- 파일명
- 디렉토리명
- 브랜치명
- 문서 파일명
- CLI에서 생성하는 route/query/mutation 리소스 이름

## 2. Detailed Rules

### 2.1 Files and Directories

- 허용: `login-service.ts`, `check-boundaries.ts`, `domain-manifest.json`
- 금지: `loginService.ts`, `login_service.ts`, `LoginService.ts`

### 2.2 Branches

- 패턴: `codex/<area>-<topic>`
- 예: `codex/rules-shared-purity`, `codex/cli-check-boundaries-json`

### 2.3 CLI Resource Names

- `create-domain <name>`에서 `<name>`은 kebab-case만 허용
- `generate route|query|mutation <domain>/<name>`에서 `<name>`은 kebab-case만 허용

예:
- `create-domain user-auth`
- `generate route product/list-items`

## 3. Exceptions

예외는 최소화하며 아래만 허용:
- 규칙 코드: `BR-001` 같은 대문자 코드
- 언어/플랫폼 고유 문법: TypeScript의 `camelCase` 변수명, `PascalCase` 타입/클래스명
- Rust 모듈 식별자: `check_boundaries` 같은 `snake_case` 모듈명은 허용

주의:
- 코드 내부 식별자 네이밍은 언어 관례를 따를 수 있지만, 파일/경로/리소스 이름은 kebab-case를 유지한다.
- Rust에서 모듈명과 파일명이 충돌하면 파일명은 `kebab-case`로 유지하고 `#[path = "..."]`로 연결한다.

## 4. Migration Rule

- 기존 이름이 kebab-case가 아니라면, 신규 수정 시점에 점진적으로 kebab-case로 정리한다.
- 대규모 rename은 기능 변경과 분리해 별도 커밋으로 수행한다.

## 5. Review Checklist

- 새 파일/폴더명이 kebab-case인가?
- 브랜치명이 kebab-case 규칙을 따르는가?
- CLI 입력 예시가 kebab-case 기준으로 작성되었는가?
- 문서 예시가 kebab-case 기준과 충돌하지 않는가?
