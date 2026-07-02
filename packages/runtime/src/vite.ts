import { spawnSync } from "node:child_process";
import { resolve } from "node:path";

import type { BoundraOverlayPayload } from "./dev-overlay.js";

const virtualId = "virtual:boundra-overlay";
const resolvedVirtualId = `\0${virtualId}`;

export type BoundraViteOptions = {
  root?: string;
  cliPath?: string;
};

type ViteServer = {
  ws: { send(payload: { type: "custom"; event: string; data: BoundraOverlayPayload }): void };
};

export function boundra(options: BoundraViteOptions = {}) {
  const root = resolve(options.root ?? process.cwd());
  let server: ViteServer | undefined;

  const publish = () => {
    const payload = runBoundaryCheck(root, options.cliPath);
    server?.ws.send({ type: "custom", event: "boundra:diagnostics", data: payload });
  };

  return {
    name: "boundra",
    apply: "serve" as const,
    resolveId(id: string) {
      return id === virtualId ? resolvedVirtualId : undefined;
    },
    load(id: string) {
      if (id !== resolvedVirtualId) return undefined;
      return `import { installBoundraDevOverlay } from "boundra/dev-overlay";
const overlay = installBoundraDevOverlay();
if (import.meta.hot) {
  import.meta.hot.on("boundra:diagnostics", (payload) => overlay.report(payload));
  import.meta.hot.dispose(() => overlay.dispose());
}`;
    },
    transformIndexHtml() {
      return [{ tag: "script", attrs: { type: "module", src: `/@id/${virtualId}` }, injectTo: "head" }];
    },
    configureServer(value: ViteServer) {
      server = value;
      publish();
    },
    handleHotUpdate(context: { file: string }) {
      if (/\.[cm]?[jt]sx?$/.test(context.file)) publish();
    },
  };
}

export function runBoundaryCheck(root: string, cliPath = "boundra"): BoundraOverlayPayload {
  const result = spawnSync(cliPath, ["check-boundaries", "--root", root, "--format", "json"], {
    encoding: "utf8",
  });
  try {
    const output = JSON.parse(result.stdout || "{}") as { violations?: BoundraOverlayPayload["diagnostics"] };
    return { source: "boundary", diagnostics: output.violations ?? [] };
  } catch {
    return {
      source: "boundary",
      diagnostics: result.status === 0 ? [] : [{
        rule: "PROJECT-002",
        file: root,
        line: 0,
        import: "",
        message: result.stderr?.trim() || "failed to run Boundra boundary check",
        suggestion: "ensure the Boundra CLI is installed and the project config is valid",
      }],
    };
  }
}
