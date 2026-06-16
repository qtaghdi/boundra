export type BoundraRoute<Input, Result> = {
  kind: "route";
  name: string;
  handler?: BoundraRouteHandler<Input, Result>;
};

export type BoundraRouteHandler<Input, Result> = (
  input: Input
) => Result | Promise<Result>;

export type BoundraQuery<Input, Result> = {
  kind: "query";
  name: string;
  handler?: BoundraQueryHandler<Input, Result>;
};

export type BoundraQueryHandler<Input, Result> = (
  input: Input
) => Result | Promise<Result>;

export type BoundraMutation<Input, Result> = {
  kind: "mutation";
  name: string;
  handler?: BoundraMutationHandler<Input, Result>;
};

export type BoundraMutationHandler<Input, Result> = (
  input: Input
) => Result | Promise<Result>;
