import { defineQuery, type InferSchema } from "boundra";
import { z } from "zod";

export const getOrderInputSchema = z.object({
  orderId: z.string().min(1),
});
export const getOrderResultSchema = z.object({
  orderId: z.string().min(1),
  status: z.enum(["pending", "submitted"]),
});

export type GetOrderQueryInput = InferSchema<typeof getOrderInputSchema>;
export type GetOrderQueryResult = InferSchema<typeof getOrderResultSchema>;

export const getOrderQuery = defineQuery({
  name: "get-order",
  input: getOrderInputSchema,
  result: getOrderResultSchema,
});
