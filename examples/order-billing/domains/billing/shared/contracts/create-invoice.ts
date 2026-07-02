import { defineRoute, type InferSchema } from "boundra";
import { z } from "zod";

export const createInvoiceInputSchema = z.object({
  orderId: z.string().min(1),
});
export const createInvoiceResultSchema = z.object({
  invoiceId: z.string().min(1),
  orderId: z.string().min(1),
});

export type CreateInvoiceInput = InferSchema<typeof createInvoiceInputSchema>;
export type CreateInvoiceResult = InferSchema<typeof createInvoiceResultSchema>;

export const createInvoiceRoute = defineRoute({
  name: "create-invoice",
  input: createInvoiceInputSchema,
  result: createInvoiceResultSchema,
});
