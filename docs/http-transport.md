# HTTP Transport

`createHttpTransport` connects a `BoundraClient` to a framework-owned HTTP
endpoint without prescribing a router or server framework.

```ts
const client = createBoundraClient(createHttpTransport({
  baseUrl: "/api/boundra",
}));
```

For contract `get-order`, the transport sends:

```http
POST /api/boundra/contracts/get-order
content-type: application/json

{"kind":"query","name":"get-order","input":{"orderId":"order-001"}}
```

The endpoint returns `{ "result": ... }`. HTTP, network, and invalid JSON
failures are wrapped by the client as `RUNTIME-003`.
