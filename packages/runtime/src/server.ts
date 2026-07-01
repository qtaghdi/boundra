import {
  BoundraRuntimeError,
  describeCause,
  parseContractValue,
} from "./errors";
import type {
  AnyBoundraContract,
  BoundraMutation,
  BoundraQuery,
  BoundraRoute,
  BoundraSchema,
  ContractInput,
  ContractResult,
} from "./types";

export type BoundraHandler<Contract extends AnyBoundraContract> = (
  input: ContractInput<Contract>,
) => ContractResult<Contract> | Promise<ContractResult<Contract>>;

export type BoundraImplementation<Contract extends AnyBoundraContract> = {
  contract: Contract;
  handler: BoundraHandler<Contract>;
};

export function implementRoute<
  InputSchema extends BoundraSchema<unknown>,
  ResultSchema extends BoundraSchema<unknown>,
>(
  contract: BoundraRoute<InputSchema, ResultSchema>,
  handler: BoundraHandler<BoundraRoute<InputSchema, ResultSchema>>,
): BoundraImplementation<BoundraRoute<InputSchema, ResultSchema>> {
  return { contract, handler };
}

export function implementQuery<
  InputSchema extends BoundraSchema<unknown>,
  ResultSchema extends BoundraSchema<unknown>,
>(
  contract: BoundraQuery<InputSchema, ResultSchema>,
  handler: BoundraHandler<BoundraQuery<InputSchema, ResultSchema>>,
): BoundraImplementation<BoundraQuery<InputSchema, ResultSchema>> {
  return { contract, handler };
}

export function implementMutation<
  InputSchema extends BoundraSchema<unknown>,
  ResultSchema extends BoundraSchema<unknown>,
>(
  contract: BoundraMutation<InputSchema, ResultSchema>,
  handler: BoundraHandler<BoundraMutation<InputSchema, ResultSchema>>,
): BoundraImplementation<BoundraMutation<InputSchema, ResultSchema>> {
  return { contract, handler };
}

export async function executeContract<Contract extends AnyBoundraContract>(
  implementation: BoundraImplementation<Contract>,
  rawInput: unknown,
): Promise<ContractResult<Contract>> {
  const { contract, handler } = implementation;
  const input = parseContractValue(
    contract.input,
    rawInput,
    contract.name,
    "input",
  ) as ContractInput<Contract>;

  let rawResult: ContractResult<Contract>;
  try {
    rawResult = await handler(input);
  } catch (cause) {
    throw new BoundraRuntimeError({
      code: "RUNTIME-003",
      contract: contract.name,
      phase: "handler",
      message: `handler failed for contract '${contract.name}': ${describeCause(cause)}`,
      suggestion: "inspect the contract handler and its dependencies",
      cause,
    });
  }

  return parseContractValue(
    contract.result,
    rawResult,
    contract.name,
    "result",
  ) as ContractResult<Contract>;
}
