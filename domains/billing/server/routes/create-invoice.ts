import { implementRoute } from "@boundra/runtime";

import { createInvoiceRoute } from "../../shared/contracts/create-invoice";

export const createInvoice = implementRoute(
  createInvoiceRoute,
  async (input) => ({
    invoiceId: `invoice-${input.orderId}`,
    orderId: input.orderId,
  }),
);
