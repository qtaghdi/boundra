import {
  BoundraHttpError,
  BoundraRuntimeError,
  createBoundraClient,
  createHttpTransport,
  defineQuery,
  defineRoute,
  executeContract,
  implementRoute,
} from "../src/index";
import { z } from "zod";

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) {
    throw new Error(message);
  }
}

async function expectRuntimeError(
  code: BoundraRuntimeError["code"],
  action: () => Promise<unknown>,
) {
  try {
    await action();
    throw new Error(`expected ${code}`);
  } catch (error) {
    assert(error instanceof BoundraRuntimeError, "expected BoundraRuntimeError");
    assert(error.code === code, `expected ${code}, received ${error.code}`);
    assert(error.suggestion.length > 0, "runtime errors should suggest a recovery");
    return error;
  }
}

const input = z.object({ id: z.string().min(1) });
const result = z.object({ id: z.string().min(1) });
const query = defineQuery({ name: "runtime-query", input, result });
const route = defineRoute({ name: "runtime-route", input, result });

const client = createBoundraClient(async (request) => request.input);
const response = await client.query(query, { id: "item-001" });
assert(response.id === "item-001", "client should return a parsed result");

let receivedHttpBody = "";
let receivedHttpSignal: AbortSignal | null | undefined;
const httpClient = createBoundraClient(createHttpTransport({
  baseUrl: "https://example.test/api/boundra/",
  fetch: async (_url, init) => {
    receivedHttpBody = String(init?.body);
    receivedHttpSignal = init?.signal;
    const request = JSON.parse(receivedHttpBody) as { input: unknown };
    return new Response(JSON.stringify({ result: request.input }), {
      status: 200,
      headers: { "content-type": "application/json" },
    });
  },
}));
const httpController = new AbortController();
const httpResponse = await httpClient.query(
  query,
  { id: "item-http" },
  { signal: httpController.signal },
);
assert(httpResponse.id === "item-http", "HTTP transport should unwrap result");
assert(receivedHttpBody.includes('"kind":"query"'), "HTTP transport should send contract kind");
assert(receivedHttpSignal === httpController.signal, "HTTP transport should receive the call signal");

const abortController = new AbortController();
abortController.abort();
let receivedCustomSignal: AbortSignal | undefined;
try {
  await createBoundraClient(async (_request, options) => {
    receivedCustomSignal = options?.signal;
    options?.signal?.throwIfAborted();
    return { id: "unreachable" };
  }).query(query, { id: "item-aborted" }, { signal: abortController.signal });
  throw new Error("expected cancellation");
} catch (error) {
  assert(error === abortController.signal.reason, "cancellation should preserve the original error");
}
assert(receivedCustomSignal === abortController.signal, "custom transport should receive call options");

const structuredHttpError = await expectRuntimeError("RUNTIME-003", () =>
  createBoundraClient(createHttpTransport({
    baseUrl: "https://example.test",
    fetch: async () => new Response(JSON.stringify({
      code: "SERVICE_BUSY",
      message: "retry later",
    }), {
      status: 503,
      statusText: "Service Unavailable",
      headers: {
        "content-type": "application/problem+json",
        "retry-after": "30",
      },
    }),
  })).query(query, { id: "item-http" }),
);
assert(
  structuredHttpError.cause instanceof BoundraHttpError,
  "RUNTIME-003 should preserve BoundraHttpError as its cause",
);
assert(structuredHttpError.cause.status === 503, "HTTP error should expose status");
assert(
  structuredHttpError.cause.headers["retry-after"] === "30",
  "HTTP error should expose response headers",
);
assert(
  typeof structuredHttpError.cause.body === "object"
    && structuredHttpError.cause.body !== null
    && "code" in structuredHttpError.cause.body
    && structuredHttpError.cause.body.code === "SERVICE_BUSY",
  "HTTP error should expose parsed JSON body",
);
assert(!structuredHttpError.cause.bodyTruncated, "small HTTP body should not be truncated");

const truncatedHttpError = await expectRuntimeError("RUNTIME-003", () =>
  createBoundraClient(createHttpTransport({
    baseUrl: "https://example.test",
    maxErrorBodyBytes: 4,
    fetch: async () => new Response("failure", {
      status: 409,
      headers: { "content-type": "text/plain" },
    }),
  })).query(query, { id: "item-http" }),
);
assert(truncatedHttpError.cause instanceof BoundraHttpError, "expected bounded HTTP error");
assert(truncatedHttpError.cause.body === "fail", "text error body should respect byte limit");
assert(truncatedHttpError.cause.bodyTruncated, "oversized error body should report truncation");

const inputError = await expectRuntimeError("RUNTIME-001", () =>
  client.query(query, { id: "" }),
);
assert(inputError.issues.length === 1, "input error should expose one issue");
assert(inputError.issues[0]?.path[0] === "id", "issue should expose the field path");
assert(inputError.suggestion.includes("id"), "suggestion should identify the field");
const serializedInputError = inputError.toJSON();
assert(serializedInputError.code === "RUNTIME-001", "serialized error should keep its code");
assert(!("cause" in serializedInputError), "serialized error should omit its cause");
await expectRuntimeError("RUNTIME-002", () =>
  createBoundraClient(async () => ({ id: "" })).query(query, {
    id: "item-001",
  }),
);
await expectRuntimeError("RUNTIME-003", () =>
  createBoundraClient(async () => {
    throw new Error("offline");
  }).query(query, { id: "item-001" }),
);

const providerAgnosticSchema = {
  parse(_value: unknown): { id: string } {
    throw {
      issues: [{ code: "invalid_value", path: ["items", 0, "sku"], message: "SKU is required" }],
    };
  },
};
const providerAgnosticQuery = defineQuery({
  name: "provider-agnostic-query",
  input: providerAgnosticSchema,
  result,
});
const providerError = await expectRuntimeError("RUNTIME-001", () =>
  client.query(providerAgnosticQuery, { id: "item-004" }),
);
assert(
  providerError.suggestion.includes("items[0].sku"),
  "structured issues should not require a Zod dependency",
);

const implementation = implementRoute(route, async (value) => value);
const executed = await executeContract(implementation, { id: "item-002" });
assert(executed.id === "item-002", "route should return a parsed result");

await expectRuntimeError("RUNTIME-003", () =>
  executeContract(
    implementRoute(route, async () => {
      throw new Error("handler failed");
    }),
    { id: "item-003" },
  ),
);

console.log("runtime-test: OK");
