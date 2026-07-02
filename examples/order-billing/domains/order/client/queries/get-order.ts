import type { BoundraClient } from "boundra";

import {
  getOrderQuery,
  type GetOrderQueryInput,
} from "../../shared/contracts/get-order";

export function getOrder(client: BoundraClient, input: GetOrderQueryInput) {
  return client.query(getOrderQuery, input);
}
