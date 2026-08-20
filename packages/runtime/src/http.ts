import type { BoundraTransport } from "./client.js";

export type BoundraHttpTransportOptions = {
  baseUrl: string;
  headers?: Readonly<Record<string, string>>;
  fetch?: typeof globalThis.fetch;
  maxErrorBodyBytes?: number;
};

export type BoundraHttpErrorOptions = {
  contract: string;
  status: number;
  statusText: string;
  headers: Readonly<Record<string, string>>;
  body: unknown;
  bodyTruncated: boolean;
  cause?: unknown;
};

export class BoundraHttpError extends Error {
  readonly contract: string;
  readonly status: number;
  readonly statusText: string;
  readonly headers: Readonly<Record<string, string>>;
  readonly body: unknown;
  readonly bodyTruncated: boolean;

  constructor(options: BoundraHttpErrorOptions) {
    super(`HTTP ${options.status} while executing '${options.contract}'`, {
      cause: options.cause,
    });
    this.name = "BoundraHttpError";
    this.contract = options.contract;
    this.status = options.status;
    this.statusText = options.statusText;
    this.headers = Object.freeze({ ...options.headers });
    this.body = options.body;
    this.bodyTruncated = options.bodyTruncated;
  }
}

const DEFAULT_MAX_ERROR_BODY_BYTES = 64 * 1024;

export function createHttpTransport(
  options: BoundraHttpTransportOptions,
): BoundraTransport {
  const request = options.fetch ?? globalThis.fetch;
  if (!request) {
    throw new Error("fetch is not available; provide options.fetch");
  }
  const baseUrl = options.baseUrl.replace(/\/$/, "");
  const maxErrorBodyBytes = options.maxErrorBodyBytes
    ?? DEFAULT_MAX_ERROR_BODY_BYTES;
  if (!Number.isSafeInteger(maxErrorBodyBytes) || maxErrorBodyBytes < 0) {
    throw new TypeError("maxErrorBodyBytes must be a non-negative safe integer");
  }

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
      throw await createHttpError(response, contract.name, maxErrorBodyBytes);
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

async function createHttpError(
  response: Response,
  contract: string,
  maxBodyBytes: number,
): Promise<BoundraHttpError> {
  const headers: Record<string, string> = {};
  response.headers.forEach((value, key) => {
    headers[key] = value;
  });

  try {
    const captured = await readBoundedBody(response, maxBodyBytes);
    return new BoundraHttpError({
      contract,
      status: response.status,
      statusText: response.statusText,
      headers,
      body: parseErrorBody(captured.text, response.headers.get("content-type")),
      bodyTruncated: captured.truncated,
    });
  } catch (cause) {
    return new BoundraHttpError({
      contract,
      status: response.status,
      statusText: response.statusText,
      headers,
      body: undefined,
      bodyTruncated: false,
      cause,
    });
  }
}

async function readBoundedBody(
  response: Response,
  maxBytes: number,
): Promise<{ text: string; truncated: boolean }> {
  if (!response.body) {
    return { text: "", truncated: false };
  }

  const reader = response.body.getReader();
  const decoder = new TextDecoder();
  let text = "";
  let bytesRead = 0;

  while (true) {
    const { done, value } = await reader.read();
    if (done) {
      text += decoder.decode();
      return { text, truncated: false };
    }

    const remaining = maxBytes - bytesRead;
    if (remaining <= 0) {
      await reader.cancel().catch(() => undefined);
      text += decoder.decode();
      return { text, truncated: true };
    }

    const captured = value.subarray(0, remaining);
    bytesRead += captured.byteLength;
    text += decoder.decode(captured, { stream: true });
    if (captured.byteLength < value.byteLength) {
      await reader.cancel().catch(() => undefined);
      text += decoder.decode();
      return { text, truncated: true };
    }
  }
}

function parseErrorBody(text: string, contentType: string | null): unknown {
  if (text.length === 0) {
    return undefined;
  }
  if (contentType?.toLowerCase().match(/(?:application\/json|\+json)(?:;|$)/)) {
    try {
      return JSON.parse(text) as unknown;
    } catch {
      return text;
    }
  }
  return text;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}
