import type { BoundraSchema, InferSchema } from "./types.js";

export type BoundraRuntimeErrorCode =
  | "RUNTIME-001"
  | "RUNTIME-002"
  | "RUNTIME-003";

export type BoundraValidationIssue = {
  code: string;
  path: ReadonlyArray<string | number>;
  message: string;
};

export type BoundraRuntimeErrorJson = {
  name: "BoundraRuntimeError";
  code: BoundraRuntimeErrorCode;
  contract: string;
  phase: "input" | "handler" | "result" | "transport";
  message: string;
  suggestion: string;
  issues: ReadonlyArray<BoundraValidationIssue>;
};

export class BoundraRuntimeError extends Error {
  readonly code: BoundraRuntimeErrorCode;
  readonly contract: string;
  readonly phase: "input" | "handler" | "result" | "transport";
  readonly suggestion: string;
  readonly issues: ReadonlyArray<BoundraValidationIssue>;

  constructor(options: {
    code: BoundraRuntimeErrorCode;
    contract: string;
    phase: "input" | "handler" | "result" | "transport";
    message: string;
    suggestion: string;
    issues?: ReadonlyArray<BoundraValidationIssue>;
    cause?: unknown;
  }) {
    super(options.message, { cause: options.cause });
    this.name = "BoundraRuntimeError";
    this.code = options.code;
    this.contract = options.contract;
    this.phase = options.phase;
    this.suggestion = options.suggestion;
    this.issues = options.issues?.map((issue) => ({
      ...issue,
      path: [...issue.path],
    })) ?? [];
  }

  toJSON(): BoundraRuntimeErrorJson {
    return {
      name: "BoundraRuntimeError",
      code: this.code,
      contract: this.contract,
      phase: this.phase,
      message: this.message,
      suggestion: this.suggestion,
      issues: this.issues,
    };
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
    const issues = normalizeValidationIssues(cause);
    const firstIssue = issues[0];
    const issuePath = firstIssue ? formatIssuePath(firstIssue.path) : undefined;
    const location = issuePath ? ` at '${issuePath}'` : "";
    const detail = firstIssue ? `: ${firstIssue.message}` : "";
    throw new BoundraRuntimeError({
      code: phase === "input" ? "RUNTIME-001" : "RUNTIME-002",
      contract,
      phase,
      message: `contract '${contract}' rejected ${phase}${location}${detail}`,
      suggestion:
        phase === "input"
          ? firstIssue
            ? `fix input field '${issuePath}': ${firstIssue.message}`
            : "provide input that matches the contract input schema"
          : firstIssue
            ? `fix result field '${issuePath}': ${firstIssue.message}`
            : "return a value that matches the contract result schema",
      issues,
      cause,
    });
  }
}

function normalizeValidationIssues(
  cause: unknown,
): ReadonlyArray<BoundraValidationIssue> {
  if (!isRecord(cause) || !Array.isArray(cause.issues)) {
    return [];
  }

  return cause.issues.flatMap((issue) => {
    if (!isRecord(issue) || typeof issue.message !== "string") {
      return [];
    }

    const path = Array.isArray(issue.path)
      ? issue.path.filter(
          (part): part is string | number =>
            typeof part === "string" || typeof part === "number",
        )
      : [];

    return [{
      code: typeof issue.code === "string" ? issue.code : "validation_error",
      path,
      message: issue.message,
    }];
  });
}

function formatIssuePath(path: ReadonlyArray<string | number>): string {
  if (path.length === 0) {
    return "value";
  }

  return path.reduce<string>((formatted, part) => {
    if (typeof part === "number") {
      return `${formatted}[${part}]`;
    }
    return formatted ? `${formatted}.${part}` : part;
  }, "");
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

export function describeCause(cause: unknown): string {
  return cause instanceof Error ? cause.message : String(cause);
}
