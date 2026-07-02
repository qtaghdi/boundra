# Boundary Rules

## 1. Rule Set (Hard Constraints)

### BR-001: Client-to-Server Import Ban
- Rule: `domains/*/client/**`에서 `domains/*/server/**` import 금지
- Reason: 배포 경계/실행 환경 분리

### BR-002: Server-to-Client Import Ban
- Rule: `domains/*/server/**`에서 `domains/*/client/**` import 금지
- Reason: 계층 역전 방지

### BR-003: Shared Purity
- Rule: `domains/*/shared/**`는 UI/DB/infra runtime 의존 금지
- Reason: 계약 계층의 순수성 보장

### BR-004: Cross-Domain Access Policy
- Rule: 도메인 간 직접 내부 경로 import 금지
- Allow: `domains/<other>/shared/public` 또는 명시된 public API만 허용
- Reason: 내부 구현 은닉과 변경 안전성 확보

### BR-005: App-to-Domain Public API Policy
- Rule: `apps/**`에서 `domains/**`를 import할 때 해당 도메인의 manifest에 선언된 public API만 허용
- Allow: `domains/<domain>/client/public`, `server/public`, `shared/public` 또는 manifest에 명시된 public API
- Reason: composition root인 앱이 도메인 내부 구현에 결합되는 것을 방지

## 2. Allow List

- `client -> shared`
- `server -> shared`
- `domain -> other-domain public API`
- `app -> domain public API`

## 3. Violation Output Format

CLI는 위반 시 아래 구조를 제공해야 한다.

```txt
[BOUNDARY_VIOLATION] BR-001
file: domains/order/client/use-order.ts
import: domains/order/server/order-service
line: 12
message: client layer cannot import server layer.
suggestion: move contract/type to shared or call via public API.
```

## 4. Exit Codes

- `0`: 위반 없음
- `1`: 위반 존재
- `2`: 설정/manifest 파싱 오류
- `3`: 내부 실행 오류

## 5. Example Violations

### Example A
- Source: `domains/product/client/list.ts`
- Import: `domains/product/server/repository.ts`
- Result: BR-001

### Example B
- Source: `domains/auth/shared/types.ts`
- Import: `packages/ui/button`
- Result: BR-003

### Example C
- Source: `domains/order/server/checkout.ts`
- Import: `domains/product/server/internal/stock.ts`
- Result: BR-004

### Example D
- Source: `apps/web/src/checkout.ts`
- Import: `domains/order/server/internal/checkout.ts`
- Result: BR-005
