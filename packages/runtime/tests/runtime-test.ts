import {
  BoundraRuntimeError,
  createBoundraClient,
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
