export type BoundraRuntimeDiagnostic = {
  name: "BoundraRuntimeError";
  code: string;
  contract: string;
  phase: "input" | "handler" | "result" | "transport";
  message: string;
  suggestion: string;
  issues: ReadonlyArray<{
    code: string;
    path: ReadonlyArray<string | number>;
    message: string;
  }>;
};

export type BoundraBoundaryDiagnostic = {
  rule: string;
  file: string;
  line: number;
  import: string;
  message: string;
  suggestion: string;
};

export type BoundraOverlayPayload = {
  source: "boundary";
  diagnostics: BoundraBoundaryDiagnostic[];
};

export type BoundraErrorViewOptions = {
  target?: HTMLElement;
  openInEditor?: (location: string) => void | Promise<void>;
};

export type BoundraErrorViewController = {
  reportRuntime(error: BoundraRuntimeDiagnostic): void;
  reportBoundary(payload: BoundraOverlayPayload): void;
  clear(source?: "runtime" | "boundary"): void;
  dispose(): void;
};

export function createBoundraErrorView(
  options?: BoundraErrorViewOptions,
): BoundraErrorViewController;

export function formatBoundraRuntimeSummary(
  error: BoundraRuntimeDiagnostic,
): string;

export function formatBoundraBoundarySummary(
  payload: BoundraOverlayPayload,
): string;
