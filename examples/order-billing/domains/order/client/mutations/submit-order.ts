import type { BoundraClient } from "boundra";

import {
  submitOrderMutation,
  type SubmitOrderMutationInput,
} from "../../shared/contracts/submit-order";

export function submitOrder(
  client: BoundraClient,
  input: SubmitOrderMutationInput,
) {
  return client.mutation(submitOrderMutation, input);
}
