# HTTP Transport

`createHttpTransport` connects a `BoundraClient` to a framework-owned HTTP
endpoint without prescribing a router or server framework.

```ts
const client = createBoundraClient(createHttpTransport({
  baseUrl: "/api/boundra",
}));

const controller = new AbortController();
const task = await client.query(
  getTask,
  { id: "task-001" },
  { signal: controller.signal },
);
```

For contract `get-order`, the transport sends:

```http
POST /api/boundra/contracts/get-order
content-type: application/json

{"kind":"query","name":"get-order","input":{"orderId":"order-001"}}
```

The endpoint returns `{ "result": ... }`. HTTP, network, and invalid JSON
failures are wrapped by the client as `RUNTIME-003`.

## Custom transports

`BoundraCallOptions` is also passed as the second argument to custom transports.
Applications can therefore keep multipart encoding, endpoint selection, and
framework-specific error bodies inside their domain adapter while receiving the
same per-call cancellation signal:

```ts
const transport: BoundraTransport = async (request, options) => {
  const input = request.input as { image: File };
  const form = new FormData();
  form.set("image", input.image);

  const response = await fetch("/api/analyze", {
    method: "POST",
    body: form,
    signal: options?.signal,
  });
  return response.json();
};
```

When the supplied signal has been aborted, Boundra rethrows the transport's
original cancellation error. Intentional cancellation is not reported as
`RUNTIME-003`.
