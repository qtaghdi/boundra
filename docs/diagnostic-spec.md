# Diagnostic Spec

## 1. Goal

Every CLI failure should answer four questions:

1. What failed?
2. Where or in which resource did it fail?
3. Why did it fail?
4. What should the user do next?

## 2. Text Shape

```txt
[ERROR] DOMAIN-001
message: unknown domain 'payment'
domain: payment
available: billing, order
suggestion: run 'boundra create-domain payment' or choose an existing domain
```

Fields:

- stable diagnostic code
- concise message
- zero or more context fields
- actionable suggestion

Boundary violations keep their existing `BR-*` text shape and additionally
follow the same message/context/suggestion principle.

## 3. JSON Shape

When JSON output is requested, failures must remain machine-readable:

```json
{
  "status": "error",
  "errors": [
    {
      "code": "DOMAIN-001",
      "message": "unknown domain 'payment'",
      "context": {
        "domain": "payment",
        "available": "billing, order"
      },
      "suggestion": "run 'boundra create-domain payment' or choose an existing domain"
    }
  ],
  "meta": {
    "command": "generate"
  }
}
```

Successful `check-boundaries` JSON and violation fields remain backward
compatible.

## 4. Code Families

- `CLI-*`: command syntax and option errors
- `PROJECT-*`: config, manifest, and project loading errors
- `DOMAIN-*`: domain creation and lookup errors
- `DEPENDENCY-*`: domain dependency workflow errors
- `GEN-*`: generated artifact and public API update errors
- `RUNTIME-*`: TypeScript contract execution errors

## 5. Runtime Validation Issues

`BoundraRuntimeError`는 framework adapter와 개발 오버레이가 특정 schema
provider에 의존하지 않도록 정규화된 `issues`를 제공한다.

```json
{
  "name": "BoundraRuntimeError",
  "code": "RUNTIME-001",
  "contract": "create-task",
  "phase": "input",
  "message": "contract 'create-task' rejected input at 'title': 제목은 두 글자 이상 입력해 주세요.",
  "suggestion": "fix input field 'title': 제목은 두 글자 이상 입력해 주세요.",
  "issues": [
    {
      "code": "too_small",
      "path": ["title"],
      "message": "제목은 두 글자 이상 입력해 주세요."
    }
  ]
}
```

규칙:

- `issues`는 항상 배열이며 validation issue가 없으면 빈 배열이다.
- issue는 `code`, `path`, `message`만 안정적으로 노출한다.
- 입력 원문이나 schema provider의 전체 오류 객체는 직렬화하지 않는다.
- `toJSON()`은 위 구조를 반환하고 내부 `cause`는 제외한다.
- runtime package는 Zod에 직접 의존하지 않고 구조적으로 호환되는 issue를 정규화한다.

## 5. Exit Codes

- `0`: success
- `1`: boundary violations
- `2`: usage, config, manifest, or resource validation error
- `3`: unexpected I/O or internal execution error
