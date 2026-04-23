# Domain Manifest Spec (Draft)

## 1. File Location

`domains/<domain>/domain.json`

## 2. Purpose

- 도메인 메타데이터 표준화
- public API와 의존 관계 선언
- analyzer/rules/codegen의 단일 참조점 제공

## 3. Proposed Schema

```json
{
  "$schema": "https://boundra.dev/schemas/domain-manifest.v1.json",
  "name": "auth",
  "version": "0.1.0",
  "publicApi": {
    "client": ["./client/public.ts"],
    "server": ["./server/public.ts"],
    "shared": ["./shared/public.ts"]
  },
  "dependsOn": ["product"],
  "policies": {
    "allowCrossDomainServerImport": false,
    "allowMcpWrite": false
  }
}
```

## 4. Validation Rules

- `name`은 디렉토리명과 일치해야 한다.
- `dependsOn`는 존재하는 도메인만 참조 가능하다.
- public API 경로는 실제 파일이어야 한다.
- internal 경로 노출은 금지한다.

## 5. Versioning

- 초기 버전: `v1`
- 변경 원칙: additive first (breaking은 major)
