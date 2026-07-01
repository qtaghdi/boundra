import {
  BoundraRuntimeError,
  createBoundraClient,
  executeContract,
  type BoundraTransport,
} from "@boundra/runtime";
import { createInvoice } from "@domains/billing/server/routes/create-invoice";
import { submitOrder } from "@domains/order/client/mutations/submit-order";
import { getOrder } from "@domains/order/client/queries/get-order";

const transport: BoundraTransport = async (request) => {
  switch (request.name) {
    case "get-order": {
      const input = request.input as { orderId: string };
      return { orderId: input.orderId, status: "pending" };
    }
    case "submit-order":
      return { orderId: "order-001", status: "submitted" };
    default:
      throw new Error(`unsupported contract: ${request.name}`);
  }
};

export async function runExample() {
  const client = createBoundraClient(transport);
  const order = await getOrder(client, { orderId: "order-001" });
  const submitted = await submitOrder(client, {
    customerId: "customer-001",
    items: [{ sku: "sku-001", quantity: 1 }],
  });
  const invoice = await executeContract(createInvoice, {
    orderId: submitted.orderId,
  });

  if (order.orderId !== invoice.orderId) {
    throw new Error("example contract flow returned inconsistent order IDs");
  }

  try {
    await getOrder(client, { orderId: "" });
    throw new Error("invalid contract input should have failed");
  } catch (error) {
    if (!(error instanceof BoundraRuntimeError) || error.code !== "RUNTIME-001") {
      throw error;
    }
  }

  return { invoice, order, submitted };
}

const result = await runExample();
console.log(`order-billing example: OK (${result.invoice.invoiceId})`);
