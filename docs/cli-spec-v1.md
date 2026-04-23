# CLI Spec v1

## 1. Commands

### 1.1 `create-domain <name>`

도메인 스캐폴드 생성.

입력 규칙:
- `<name>`은 `kebab-case`만 허용

생성 대상:
- `domains/<name>/client/`
- `domains/<name>/server/`
- `domains/<name>/shared/`
- `domains/<name>/mcp/`
- `domains/<name>/tests/`
- `domains/<name>/domain.json`

옵션:
- `--with-mcp` (default: true)
- `--with-tests` (default: true)

### 1.2 `check-boundaries`

모든 도메인 import를 분석해 boundary rule 위반을 탐지.

옵션:
- `--format text|json` (default: text)
- `--fail-on-warning` (default: false)
- `--changed-only` (git diff 기반 선택 분석, optional)

### 1.3 `graph-domains`

도메인 의존 그래프 생성.

옵션:
- `--format mermaid|json|dot` (default: mermaid)
- `--output <path>`

### 1.4 `generate route <domain>/<name>`
### 1.5 `generate query <domain>/<name>`
### 1.6 `generate mutation <domain>/<name>`

계약 기반 템플릿 파일 생성.

입력 규칙:
- `<domain>`, `<name>`은 `kebab-case`만 허용

## 2. CLI UX Principles

- 에러 메시지는 규칙 코드(BR-xxx)와 수정 힌트를 포함한다.
- 성공 출력은 요약 1줄 + 상세 경로를 제공한다.
- JSON 출력은 CI/AI 파싱에 안정적인 스키마를 유지한다.

## 3. Machine-Readable Output (JSON)

```json
{
  "status": "failed",
  "rule": "BR-001",
  "violations": [
    {
      "file": "domains/order/client/use-order.ts",
      "line": 12,
      "import": "domains/order/server/order-service",
      "message": "client layer cannot import server layer",
      "suggestion": "move contract/type to shared"
    }
  ]
}
```
