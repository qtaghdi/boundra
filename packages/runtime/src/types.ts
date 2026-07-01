export type BoundraContractKind = "route" | "query" | "mutation";

export type BoundraSchema<Output> = {
  parse(value: unknown): Output;
};

export type InferSchema<Schema> =
  Schema extends BoundraSchema<infer Output> ? Output : never;

export type BoundraContract<
  Kind extends BoundraContractKind,
  InputSchema extends BoundraSchema<unknown>,
  ResultSchema extends BoundraSchema<unknown>,
> = {
  kind: Kind;
  name: string;
  input: InputSchema;
  result: ResultSchema;
};

export type BoundraRoute<
  InputSchema extends BoundraSchema<unknown>,
  ResultSchema extends BoundraSchema<unknown>,
> = BoundraContract<"route", InputSchema, ResultSchema>;

export type BoundraQuery<
  InputSchema extends BoundraSchema<unknown>,
  ResultSchema extends BoundraSchema<unknown>,
> = BoundraContract<"query", InputSchema, ResultSchema>;

export type BoundraMutation<
  InputSchema extends BoundraSchema<unknown>,
  ResultSchema extends BoundraSchema<unknown>,
> = BoundraContract<"mutation", InputSchema, ResultSchema>;

export type AnyBoundraContract = BoundraContract<
  BoundraContractKind,
  BoundraSchema<unknown>,
  BoundraSchema<unknown>
>;

export type ContractInput<Contract extends AnyBoundraContract> = InferSchema<
  Contract["input"]
>;

export type ContractResult<Contract extends AnyBoundraContract> = InferSchema<
  Contract["result"]
>;

export function defineRoute<
  InputSchema extends BoundraSchema<unknown>,
  ResultSchema extends BoundraSchema<unknown>,
>(
  contract: Omit<BoundraRoute<InputSchema, ResultSchema>, "kind">,
): BoundraRoute<InputSchema, ResultSchema> {
  return { kind: "route", ...contract };
}

export function defineQuery<
  InputSchema extends BoundraSchema<unknown>,
  ResultSchema extends BoundraSchema<unknown>,
>(
  contract: Omit<BoundraQuery<InputSchema, ResultSchema>, "kind">,
): BoundraQuery<InputSchema, ResultSchema> {
  return { kind: "query", ...contract };
}

export function defineMutation<
  InputSchema extends BoundraSchema<unknown>,
  ResultSchema extends BoundraSchema<unknown>,
>(
  contract: Omit<BoundraMutation<InputSchema, ResultSchema>, "kind">,
): BoundraMutation<InputSchema, ResultSchema> {
  return { kind: "mutation", ...contract };
}
