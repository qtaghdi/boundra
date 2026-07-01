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

## 5. Exit Codes

- `0`: success
- `1`: boundary violations
- `2`: usage, config, manifest, or resource validation error
- `3`: unexpected I/O or internal execution error
