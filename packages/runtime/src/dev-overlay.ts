import type { BoundraRuntimeErrorJson } from "./errors.js";

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

const overlayId = "__boundra_dev_overlay__";

export function installBoundraDevOverlay() {
  const renderRuntime = (error: BoundraRuntimeErrorJson) => {
    const issue = error.issues[0];
    renderOverlay({
      eyebrow: `${error.code} · ${error.phase}`,
      title: `Contract '${error.contract}' rejected ${error.phase}`,
      detail: issue
        ? `${formatPath(issue.path)} — ${issue.message}`
        : error.message,
      suggestion: error.suggestion,
    });
  };

  const onError = (event: ErrorEvent) => {
    if (isRuntimeError(event.error)) renderRuntime(event.error);
  };
  const onRejection = (event: PromiseRejectionEvent) => {
    if (isRuntimeError(event.reason)) renderRuntime(event.reason);
  };
  window.addEventListener("error", onError);
  window.addEventListener("unhandledrejection", onRejection);

  return {
    report(payload: BoundraOverlayPayload) {
      if (payload.diagnostics.length === 0) {
        document.getElementById(overlayId)?.remove();
        return;
      }
      const diagnostic = payload.diagnostics[0]!;
      renderOverlay({
        eyebrow: `${diagnostic.rule} · boundary`,
        title: `${diagnostic.file}:${diagnostic.line}`,
        detail: diagnostic.message,
        suggestion: diagnostic.suggestion,
      });
    },
    dispose() {
      window.removeEventListener("error", onError);
      window.removeEventListener("unhandledrejection", onRejection);
      document.getElementById(overlayId)?.remove();
    },
  };
}

function renderOverlay(content: {
  eyebrow: string;
  title: string;
  detail: string;
  suggestion: string;
}) {
  document.getElementById(overlayId)?.remove();
  const root = document.createElement("div");
  root.id = overlayId;
  root.setAttribute("role", "alert");
  root.innerHTML = `
    <style>
      #${overlayId}{position:fixed;inset:0;z-index:2147483647;padding:7vh 7vw;background:rgba(16,22,18,.94);color:#f7f8f4;font-family:ui-monospace,SFMono-Regular,Menlo,monospace;overflow:auto}
      #${overlayId} .b-card{max-width:920px;margin:auto;border:1px solid #5c6e61;border-radius:5px 28px 5px 5px;background:#1c2820;box-shadow:0 30px 90px #0008;overflow:hidden}
      #${overlayId} .b-head{display:flex;justify-content:space-between;gap:24px;padding:18px 24px;background:#c8ed6b;color:#17211b;font-size:12px;font-weight:800;letter-spacing:.08em;text-transform:uppercase}
      #${overlayId} button{border:0;background:transparent;color:#17211b;font:inherit;cursor:pointer}
      #${overlayId} .b-body{padding:34px 36px 40px}#${overlayId} h1{margin:0 0 28px;font:700 clamp(24px,4vw,42px)/1.15 system-ui,sans-serif;letter-spacing:-.04em}
      #${overlayId} pre{white-space:pre-wrap;padding:18px;border-left:3px solid #c8ed6b;background:#111913;color:#e7ece7;line-height:1.6}
      #${overlayId} .b-fix{margin-top:24px;color:#b9c7bc;font:14px/1.65 system-ui,sans-serif}#${overlayId} .b-fix strong{color:#c8ed6b}
    </style>
    <div class="b-card"><div class="b-head"><span>${escapeHtml(content.eyebrow)}</span><button type="button" aria-label="Close Boundra error overlay">ESC ×</button></div>
      <div class="b-body"><h1>${escapeHtml(content.title)}</h1><pre>${escapeHtml(content.detail)}</pre><p class="b-fix"><strong>How to fix</strong><br>${escapeHtml(content.suggestion)}</p></div>
    </div>`;
  root.querySelector("button")?.addEventListener("click", () => root.remove());
  document.body.append(root);
}

function isRuntimeError(value: unknown): value is BoundraRuntimeErrorJson {
  return typeof value === "object" && value !== null
    && "name" in value && value.name === "BoundraRuntimeError"
    && "issues" in value && Array.isArray(value.issues);
}

function formatPath(path: ReadonlyArray<string | number>) {
  return path.reduce<string>((value, part) =>
    typeof part === "number" ? `${value}[${part}]` : value ? `${value}.${part}` : part, "") || "value";
}

function escapeHtml(value: string) {
  return value.replace(/[&<>'"]/g, (character) => ({
    "&": "&amp;", "<": "&lt;", ">": "&gt;", "'": "&#39;", "\"": "&quot;",
  })[character]!);
}
