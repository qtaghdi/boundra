# Order and Billing Example

This example demonstrates two Boundra domains connected through a declared
dependency and public contract APIs.

- `order` owns the `get-order` query and `submit-order` mutation.
- `billing` depends on `order` and owns the `create-invoice` route.
- runtime validation rejects invalid contract input with `RUNTIME-001`.

From the repository root:

```bash
pnpm example:order-billing
cargo run -p boundra-cli -- check-boundaries --root examples/order-billing
cargo run -p boundra-cli -- graph-domains --root examples/order-billing --format mermaid
```
