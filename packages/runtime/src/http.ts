import type { BoundraTransport } from "./client.js";

export type BoundraHttpTransportOptions = {
  baseUrl: string;
  headers?: Readonly<Record<string, string>>;
  fetch?: typeof globalThis.fetch;
};

export function createHttpTransport(
  options: BoundraHttpTransportOptions,
): BoundraTransport {
  const request = options.fetch ?? globalThis.fetch;
  if (!request) {
    throw new Error("fetch is not available; provide options.fetch");
  }
  const baseUrl = options.baseUrl.replace(/\/$/, "");

  return async (contract, callOptions) => {
    const response = await request(
      `${baseUrl}/contracts/${encodeURIComponent(contract.name)}`,
      {
        method: "POST",
        headers: {
          "content-type": "application/json",
          ...options.headers,
        },
        body: JSON.stringify(contract),
        signal: callOptions?.signal,
      },
    );
    if (!response.ok) {
      throw new Error(`HTTP ${response.status} while executing '${contract.name}'`);
    }

    let payload: unknown;
    try {
      payload = await response.json();
    } catch (cause) {
      throw new Error(`invalid JSON response for '${contract.name}'`, { cause });
    }
    if (!isRecord(payload) || !("result" in payload)) {
      throw new Error(`response for '${contract.name}' must contain a result field`);
    }
    return payload.result;
  };
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}
