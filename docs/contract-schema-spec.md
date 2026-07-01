# Contract Schema Spec

## 1. Source of Truth

Each generated contract file in `domains/<domain>/shared/contracts/` owns two
runtime schemas:

- input schema
- result schema

TypeScript input and result types are inferred from those schemas. A separate
handwritten interface is not generated.

## 2. Schema Provider

The initial generator emits Zod schemas. `@boundra/runtime` remains provider
agnostic by requiring only this structural shape:

```ts
export type BoundraSchema<Output> = {
  parse(value: unknown): Output;
};
```

Another schema provider may be used when its schema implements the same parse
contract. Provider-specific features are not part of the Boundra runtime API.

## 3. Contract Shape

```ts
import { defineQuery, type InferSchema } from "@boundra/runtime";
import { z } from "zod";

export const getOrderInputSchema = z.object({});
export const getOrderResultSchema = z.object({});

export type GetOrderQueryInput = InferSchema<typeof getOrderInputSchema>;
export type GetOrderQueryResult = InferSchema<typeof getOrderResultSchema>;

export const getOrderQuery = defineQuery({
  name: "get-order",
  input: getOrderInputSchema,
  result: getOrderResultSchema,
});
```

The generated empty objects are safe scaffolds. Feature work replaces their
fields with real domain schemas; generated files are application-owned after
creation and are never overwritten.

## 4. Runtime Behavior

- client query/mutation calls parse input before transport
- transport results are parsed before returning to application code
- server route execution parses input before the handler and parses the handler
  result before returning
- missing handlers and schema parse failures surface typed runtime errors

## 5. Public Exports

Generation appends an idempotent export to `shared/public.ts`:

```ts
export * from "./contracts/get-order";
```

The direct contract path remains in `domain.json` `publicApi.shared` for
compatibility with current manifests. Consumers should prefer the stable
`shared/public` entrypoint.

## 6. Ownership

- generator owns initial file creation
- application code owns schema fields and handler implementation afterward
- repeated generation must fail rather than overwrite application changes
