import {
  BoundraRuntimeError,
  describeCause,
  parseContractValue,
} from "./errors";
import type {
  BoundraMutation,
  BoundraQuery,
  BoundraSchema,
  InferSchema,
} from "./types";

export type BoundraTransportRequest = {
  kind: "query" | "mutation";
  name: string;
  input: unknown;
};

export type BoundraTransport = (
  request: BoundraTransportRequest,
) => Promise<unknown>;

export type BoundraClient = {
  query<
    InputSchema extends BoundraSchema<unknown>,
    ResultSchema extends BoundraSchema<unknown>,
  >(
    contract: BoundraQuery<InputSchema, ResultSchema>,
    input: InferSchema<InputSchema>,
  ): Promise<InferSchema<ResultSchema>>;
  mutation<
    InputSchema extends BoundraSchema<unknown>,
    ResultSchema extends BoundraSchema<unknown>,
  >(
    contract: BoundraMutation<InputSchema, ResultSchema>,
    input: InferSchema<InputSchema>,
  ): Promise<InferSchema<ResultSchema>>;
};

export function createBoundraClient(transport: BoundraTransport): BoundraClient {
  async function send<
    InputSchema extends BoundraSchema<unknown>,
    ResultSchema extends BoundraSchema<unknown>,
  >(
    contract:
      | BoundraQuery<InputSchema, ResultSchema>
      | BoundraMutation<InputSchema, ResultSchema>,
    input: InferSchema<InputSchema>,
  ): Promise<InferSchema<ResultSchema>> {
    const parsedInput = parseContractValue(
      contract.input,
      input,
      contract.name,
      "input",
    );

    let rawResult: unknown;
    try {
      rawResult = await transport({
        kind: contract.kind,
        name: contract.name,
        input: parsedInput,
      });
    } catch (cause) {
      throw new BoundraRuntimeError({
        code: "RUNTIME-003",
        contract: contract.name,
        phase: "transport",
        message: `transport failed for contract '${contract.name}': ${describeCause(cause)}`,
        suggestion: "check the transport connection and contract routing",
        cause,
      });
    }

    return parseContractValue(
      contract.result,
      rawResult,
      contract.name,
      "result",
    );
  }

  return {
    query: (contract, input) => send(contract, input),
    mutation: (contract, input) => send(contract, input),
  };
}
