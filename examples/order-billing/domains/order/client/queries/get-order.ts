import type { BoundraClient } from "@boundra/runtime";

import {
  getOrderQuery,
  type GetOrderQueryInput,
} from "../../shared/contracts/get-order";

export function getOrder(client: BoundraClient, input: GetOrderQueryInput) {
  return client.query(getOrderQuery, input);
}
