import type { GetOrderQueryResult } from "@domains/order/shared/public";

import type {
  CreateInvoiceInput,
  CreateInvoiceResult,
} from "../shared/contracts/create-invoice";

export async function createInvoiceWorkflow(
  input: CreateInvoiceInput,
  order: GetOrderQueryResult,
): Promise<CreateInvoiceResult> {
  void input;
  return {
    invoiceId: `invoice-${order.orderId}`,
    orderId: order.orderId,
  };
}
