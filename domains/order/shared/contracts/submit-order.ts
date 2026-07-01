import { defineMutation, type InferSchema } from "@boundra/runtime";
import { z } from "zod";

export const submitOrderInputSchema = z.object({
  customerId: z.string().min(1),
  items: z
    .array(
      z.object({
        sku: z.string().min(1),
        quantity: z.number().int().positive(),
      }),
    )
    .min(1),
});
export const submitOrderResultSchema = z.object({
  orderId: z.string().min(1),
  status: z.literal("submitted"),
});

export type SubmitOrderMutationInput = InferSchema<
  typeof submitOrderInputSchema
>;
export type SubmitOrderMutationResult = InferSchema<
  typeof submitOrderResultSchema
>;

export const submitOrderMutation = defineMutation({
  name: "submit-order",
  input: submitOrderInputSchema,
  result: submitOrderResultSchema,
});
