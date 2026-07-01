import type { BoundraSchema, InferSchema } from "./types";

export type BoundraRuntimeErrorCode =
  | "RUNTIME-001"
  | "RUNTIME-002"
  | "RUNTIME-003";

export class BoundraRuntimeError extends Error {
  readonly code: BoundraRuntimeErrorCode;
  readonly contract: string;
  readonly phase: "input" | "handler" | "result" | "transport";
  readonly suggestion: string;

  constructor(options: {
    code: BoundraRuntimeErrorCode;
    contract: string;
    phase: "input" | "handler" | "result" | "transport";
    message: string;
    suggestion: string;
    cause?: unknown;
  }) {
    super(options.message, { cause: options.cause });
    this.name = "BoundraRuntimeError";
    this.code = options.code;
    this.contract = options.contract;
    this.phase = options.phase;
    this.suggestion = options.suggestion;
  }
}

export function parseContractValue<Schema extends BoundraSchema<unknown>>(
  schema: Schema,
  value: unknown,
  contract: string,
  phase: "input" | "result",
): InferSchema<Schema> {
  try {
    return schema.parse(value) as InferSchema<Schema>;
  } catch (cause) {
    throw new BoundraRuntimeError({
      code: phase === "input" ? "RUNTIME-001" : "RUNTIME-002",
      contract,
      phase,
      message: `contract '${contract}' rejected ${phase}: ${describeCause(cause)}`,
      suggestion:
        phase === "input"
          ? "provide input that matches the contract input schema"
          : "return a value that matches the contract result schema",
      cause,
    });
  }
}

export function describeCause(cause: unknown): string {
  return cause instanceof Error ? cause.message : String(cause);
}
