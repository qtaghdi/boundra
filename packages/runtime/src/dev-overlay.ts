import type { BoundraRuntimeErrorJson } from "./errors.js";
import {
  createBoundraErrorView,
  formatBoundraBoundarySummary,
  formatBoundraRuntimeSummary,
  type BoundraOverlayPayload,
} from "./ui/index.js";

export type {
  BoundraBoundaryDiagnostic,
  BoundraOverlayPayload,
} from "./ui/index.js";

export {
  formatBoundraBoundarySummary,
  formatBoundraRuntimeSummary,
};

export function installBoundraDevOverlay() {
  const view = createBoundraErrorView({
    openInEditor(location) {
      return fetch(`/__open-in-editor?file=${encodeURIComponent(location)}`)
        .then(() => undefined);
    },
  });

  const onError = (event: ErrorEvent) => {
    if (isRuntimeError(event.error)) view.reportRuntime(event.error);
  };
  const onRejection = (event: PromiseRejectionEvent) => {
    if (isRuntimeError(event.reason)) view.reportRuntime(event.reason);
  };
  window.addEventListener("error", onError);
  window.addEventListener("unhandledrejection", onRejection);

  return {
    report(payload: BoundraOverlayPayload) {
      view.reportBoundary(payload);
    },
    reportRuntime(error: BoundraRuntimeErrorJson) {
      view.reportRuntime(error);
    },
    clear(source?: "runtime" | "boundary") {
      view.clear(source);
    },
    dispose() {
      window.removeEventListener("error", onError);
      window.removeEventListener("unhandledrejection", onRejection);
      view.dispose();
    },
  };
}

function isRuntimeError(value: unknown): value is BoundraRuntimeErrorJson {
  return typeof value === "object" && value !== null
    && "name" in value && value.name === "BoundraRuntimeError"
    && "issues" in value && Array.isArray(value.issues);
}

export type { BoundraRuntimeErrorJson };
