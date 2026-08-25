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

type OverlaySource = "runtime" | "boundary";

type OverlayEntry = {
  code: string;
  title: string;
  message: string;
  suggestion: string;
  metadata: ReadonlyArray<readonly [label: string, value: string]>;
  editorLocation?: string;
};

const overlayId = "__boundra_dev_overlay__";

export function installBoundraDevOverlay() {
  const entries: Record<OverlaySource, OverlayEntry[]> = {
    runtime: [],
    boundary: [],
  };
  let activeSource: OverlaySource = "runtime";
  let activeIndex = 0;

  const render = () => {
    if (entries[activeSource].length === 0) {
      activeSource = activeSource === "runtime" ? "boundary" : "runtime";
      activeIndex = 0;
    }
    if (entries[activeSource].length === 0) {
      document.getElementById(overlayId)?.remove();
      return;
    }
    activeIndex = Math.min(activeIndex, entries[activeSource].length - 1);
    renderOverlay({
      entries,
      activeSource,
      activeIndex,
      onSelectSource(source) {
        activeSource = source;
        activeIndex = 0;
        render();
      },
      onSelectEntry(index) {
        activeIndex = index;
        render();
      },
      onDismiss() {
        entries[activeSource] = [];
        render();
      },
      onCopy(button) {
        void copyText(formatOverlaySummary(activeSource, entries[activeSource]))
          .then(() => setCopyStatus(button, "Copied"))
          .catch(() => setCopyStatus(button, "Copy unavailable"));
      },
    });
  };

  const renderRuntime = (error: BoundraRuntimeErrorJson) => {
    entries.runtime = createRuntimeEntries(error);
    activeSource = "runtime";
    activeIndex = 0;
    render();
  };

  const onError = (event: ErrorEvent) => {
    if (isRuntimeError(event.error)) renderRuntime(event.error);
  };
  const onRejection = (event: PromiseRejectionEvent) => {
    if (isRuntimeError(event.reason)) renderRuntime(event.reason);
  };
  const onKeyDown = (event: KeyboardEvent) => {
    if (event.key === "Escape" && document.getElementById(overlayId)) {
      entries[activeSource] = [];
      render();
    }
  };
  window.addEventListener("error", onError);
  window.addEventListener("unhandledrejection", onRejection);
  window.addEventListener("keydown", onKeyDown);

  return {
    report(payload: BoundraOverlayPayload) {
      entries.boundary = createBoundaryEntries(payload.diagnostics);
      if (entries.boundary.length > 0) {
        activeSource = "boundary";
        activeIndex = 0;
      }
      render();
    },
    dispose() {
      window.removeEventListener("error", onError);
      window.removeEventListener("unhandledrejection", onRejection);
      window.removeEventListener("keydown", onKeyDown);
      document.getElementById(overlayId)?.remove();
    },
  };
}

export function formatBoundraRuntimeSummary(error: BoundraRuntimeErrorJson): string {
  return formatOverlaySummary("runtime", createRuntimeEntries(error));
}

export function formatBoundraBoundarySummary(payload: BoundraOverlayPayload): string {
  return formatOverlaySummary("boundary", createBoundaryEntries(payload.diagnostics));
}

function createRuntimeEntries(error: BoundraRuntimeErrorJson): OverlayEntry[] {
  if (error.issues.length === 0) {
    return [{
      code: error.code,
      title: `Contract '${error.contract}' rejected ${error.phase}`,
      message: error.message,
      suggestion: error.suggestion,
      metadata: [
        ["contract", error.contract],
        ["phase", error.phase],
        ["path", "value"],
      ],
    }];
  }

  return error.issues.map((issue) => ({
    code: error.code,
    title: formatPath(issue.path),
    message: issue.message,
    suggestion: error.suggestion,
    metadata: [
      ["contract", error.contract],
      ["phase", error.phase],
      ["issue", issue.code],
      ["path", formatPath(issue.path)],
    ],
  }));
}

function createBoundaryEntries(diagnostics: ReadonlyArray<BoundraBoundaryDiagnostic>): OverlayEntry[] {
  return diagnostics.map((diagnostic) => ({
    code: diagnostic.rule,
    title: `${diagnostic.file}:${diagnostic.line}`,
    message: diagnostic.message,
    suggestion: diagnostic.suggestion,
    metadata: [
      ["file", diagnostic.file],
      ["line", String(diagnostic.line)],
      ...(diagnostic.import ? [["import", diagnostic.import] as const] : []),
    ],
    editorLocation: diagnostic.line > 0
      ? `${diagnostic.file}:${diagnostic.line}`
      : undefined,
  }));
}

function formatOverlaySummary(source: OverlaySource, entries: ReadonlyArray<OverlayEntry>): string {
  const heading = `Boundra ${source} diagnostics (${entries.length})`;
  const details = entries.map((entry, index) => {
    const metadata = entry.metadata.map(([label, value]) => `${label}: ${value}`).join("\n");
    return [
      `${index + 1}. [${entry.code}] ${entry.title}`,
      metadata,
      `message: ${entry.message}`,
      `suggestion: ${entry.suggestion}`,
    ].filter(Boolean).join("\n");
  });
  return [heading, ...details].join("\n\n");
}

function renderOverlay(options: {
  entries: Record<OverlaySource, OverlayEntry[]>;
  activeSource: OverlaySource;
  activeIndex: number;
  onSelectSource(source: OverlaySource): void;
  onSelectEntry(index: number): void;
  onDismiss(): void;
  onCopy(button: HTMLButtonElement): void;
}) {
  document.getElementById(overlayId)?.remove();
  const activeEntries = options.entries[options.activeSource];
  const selected = activeEntries[options.activeIndex]!;
  const root = document.createElement("div");
  root.id = overlayId;
  root.setAttribute("role", "alertdialog");
  root.setAttribute("aria-modal", "true");
  root.setAttribute("aria-labelledby", `${overlayId}_title`);
  root.tabIndex = -1;
  root.innerHTML = `
    <style>
      #${overlayId}{position:fixed;inset:0;z-index:2147483647;padding:clamp(16px,5vh,56px) clamp(12px,5vw,72px);background:rgba(10,15,12,.94);color:#f7f8f4;font-family:ui-monospace,SFMono-Regular,Menlo,monospace;overflow:auto}
      #${overlayId} *{box-sizing:border-box}#${overlayId} button{font:inherit}#${overlayId} .b-shell{width:min(1180px,100%);min-height:min(720px,88vh);margin:auto;border:1px solid #53665a;border-radius:8px 30px 8px 8px;background:#17221b;box-shadow:0 30px 90px #0009;overflow:hidden}
      #${overlayId} .b-head{display:flex;align-items:center;justify-content:space-between;gap:20px;padding:15px 20px;background:#c8ed6b;color:#17211b}#${overlayId} .b-brand{font-size:12px;font-weight:900;letter-spacing:.1em;text-transform:uppercase}#${overlayId} .b-actions{display:flex;gap:8px}
      #${overlayId} .b-action{border:1px solid #526333;border-radius:999px;padding:7px 11px;background:#e9ffc0;color:#17211b;font-size:12px;font-weight:800;cursor:pointer}#${overlayId} .b-action:hover{background:#fff}#${overlayId} .b-action:focus-visible,#${overlayId} .b-tab:focus-visible,#${overlayId} .b-item:focus-visible{outline:3px solid #fff;outline-offset:2px}
      #${overlayId} .b-tabs{display:flex;gap:8px;padding:14px 20px;border-bottom:1px solid #35443a;background:#111913}#${overlayId} .b-tab{border:1px solid #526056;border-radius:999px;padding:8px 13px;background:transparent;color:#aebbb1;cursor:pointer}#${overlayId} .b-tab[aria-selected=true]{border-color:#c8ed6b;background:#263322;color:#eaffbd}#${overlayId} .b-tab:disabled{opacity:.4;cursor:not-allowed}
      #${overlayId} .b-layout{display:grid;grid-template-columns:minmax(240px,34%) 1fr;min-height:620px}#${overlayId} .b-list{margin:0;padding:14px;list-style:none;border-right:1px solid #35443a;background:#121b16;overflow:auto}#${overlayId} .b-item{display:block;width:100%;margin:0 0 8px;border:1px solid #34463a;border-radius:6px;padding:14px;background:#1a271f;color:#eef3ef;text-align:left;cursor:pointer}#${overlayId} .b-item[aria-current=true]{border-color:#c8ed6b;background:#253324}#${overlayId} .b-code{display:block;margin-bottom:7px;color:#c8ed6b;font-size:11px;font-weight:900;letter-spacing:.08em}#${overlayId} .b-item-title{display:block;overflow:hidden;font-size:13px;line-height:1.45;text-overflow:ellipsis;white-space:nowrap}
      #${overlayId} .b-detail{padding:clamp(24px,4vw,48px);overflow:auto}#${overlayId} .b-kicker{margin:0 0 10px;color:#c8ed6b;font-size:12px;font-weight:900;letter-spacing:.08em;text-transform:uppercase}#${overlayId} h1{margin:0 0 28px;font:750 clamp(25px,4vw,44px)/1.14 system-ui,sans-serif;letter-spacing:-.04em;overflow-wrap:anywhere}#${overlayId} dl{display:grid;grid-template-columns:max-content 1fr;gap:8px 18px;margin:0 0 24px;padding:16px 18px;border:1px solid #35443a;border-radius:7px;background:#111913;font-size:12px}#${overlayId} dt{color:#92a197}#${overlayId} dd{margin:0;color:#f1f5f2;overflow-wrap:anywhere}
      #${overlayId} .b-message{margin:0;padding:18px;border-left:3px solid #c8ed6b;background:#0e1611;color:#e7ece7;white-space:pre-wrap;overflow-wrap:anywhere;line-height:1.65}#${overlayId} .b-fix{margin:24px 0 0;color:#b9c7bc;font:14px/1.65 system-ui,sans-serif}#${overlayId} .b-fix strong{color:#c8ed6b}#${overlayId} .b-editor{margin-top:20px;border:0;border-bottom:1px solid #c8ed6b;padding:0 0 2px;background:transparent;color:#c8ed6b;cursor:pointer}
      @media(max-width:720px){#${overlayId}{padding:0}#${overlayId} .b-shell{min-height:100%;border:0;border-radius:0}#${overlayId} .b-head{align-items:flex-start}#${overlayId} .b-layout{display:block}#${overlayId} .b-list{display:flex;min-height:0;border-right:0;border-bottom:1px solid #35443a;overflow-x:auto}#${overlayId} .b-item{min-width:220px;margin:0 8px 0 0}#${overlayId} .b-detail{padding:24px 20px}#${overlayId} dl{grid-template-columns:1fr;gap:4px}#${overlayId} dd{margin-bottom:8px}}
      @media(prefers-reduced-motion:reduce){#${overlayId} *{scroll-behavior:auto!important}}
    </style>
    <section class="b-shell">
      <header class="b-head"><span class="b-brand">Boundra development error view</span><div class="b-actions"><button class="b-action" data-copy type="button">Copy safe report</button><button class="b-action" data-dismiss type="button" aria-label="Dismiss current Boundra diagnostics">ESC ×</button></div></header>
      <nav class="b-tabs" role="tablist" aria-label="Diagnostic sources">${renderSourceTab("runtime", options)}${renderSourceTab("boundary", options)}</nav>
      <div class="b-layout">
        <ol class="b-list" aria-label="${escapeHtml(options.activeSource)} diagnostics">${activeEntries.map((entry, index) => renderEntryButton(entry, index, index === options.activeIndex)).join("")}</ol>
        <main class="b-detail"><p class="b-kicker">${escapeHtml(selected.code)} · ${options.activeIndex + 1} of ${activeEntries.length}</p><h1 id="${overlayId}_title">${escapeHtml(selected.title)}</h1><dl>${selected.metadata.map(([label, value]) => `<dt>${escapeHtml(label)}</dt><dd>${escapeHtml(value)}</dd>`).join("")}</dl><pre class="b-message">${escapeHtml(selected.message)}</pre><p class="b-fix"><strong>How to fix</strong><br>${escapeHtml(selected.suggestion)}</p>${selected.editorLocation ? `<button class="b-editor" data-editor="${escapeHtml(selected.editorLocation)}" type="button">Open in editor ↗</button>` : ""}</main>
      </div>
    </section>`;

  root.querySelectorAll<HTMLButtonElement>("[data-source]").forEach((button) => {
    button.addEventListener("click", () => options.onSelectSource(button.dataset.source as OverlaySource));
  });
  root.querySelectorAll<HTMLButtonElement>("[data-index]").forEach((button) => {
    button.addEventListener("click", () => options.onSelectEntry(Number(button.dataset.index)));
  });
  root.querySelector<HTMLButtonElement>("[data-dismiss]")?.addEventListener("click", options.onDismiss);
  root.querySelector<HTMLButtonElement>("[data-copy]")?.addEventListener("click", (event) => options.onCopy(event.currentTarget as HTMLButtonElement));
  root.querySelector<HTMLButtonElement>("[data-editor]")?.addEventListener("click", (event) => {
    const location = (event.currentTarget as HTMLButtonElement).dataset.editor;
    if (location) void fetch(`/__open-in-editor?file=${encodeURIComponent(location)}`);
  });
  document.body.append(root);
  root.focus({ preventScroll: true });
}

function renderSourceTab(source: OverlaySource, options: {
  entries: Record<OverlaySource, OverlayEntry[]>;
  activeSource: OverlaySource;
}) {
  const count = options.entries[source].length;
  return `<button class="b-tab" data-source="${source}" type="button" role="tab" aria-selected="${source === options.activeSource}" ${count === 0 ? "disabled" : ""}>${source === "runtime" ? "Runtime" : "Boundaries"} · ${count}</button>`;
}

function renderEntryButton(entry: OverlayEntry, index: number, selected: boolean) {
  return `<li><button class="b-item" data-index="${index}" type="button" aria-current="${selected}"><span class="b-code">${escapeHtml(entry.code)}</span><span class="b-item-title">${escapeHtml(entry.title)}</span></button></li>`;
}

async function copyText(value: string) {
  if (!navigator.clipboard?.writeText) throw new Error("Clipboard API unavailable");
  await navigator.clipboard.writeText(value);
}

function setCopyStatus(button: HTMLButtonElement, status: string) {
  button.textContent = status;
  window.setTimeout(() => {
    if (button.isConnected) button.textContent = "Copy safe report";
  }, 1600);
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
