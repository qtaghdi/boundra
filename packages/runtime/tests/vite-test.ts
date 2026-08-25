import { mkdir, mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";

import {
  formatBoundraBoundarySummary,
  formatBoundraRuntimeSummary,
} from "../src/dev-overlay";
import { boundra } from "../src/vite";

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(message);
}

const root = await mkdtemp(join(tmpdir(), "boundra-vite-test-"));
try {
  await mkdir(join(root, "apps/web/src"), { recursive: true });
  await mkdir(join(root, "domains/tasks/server/internal"), { recursive: true });
  await mkdir(join(root, "domains/tasks/shared"), { recursive: true });
  await writeFile(join(root, "boundra.config.json"), JSON.stringify({
    paths: { apps: "apps", domains: "domains" },
  }));
  await writeFile(join(root, "domains/tasks/shared/public.ts"), "export {};\n");
  await writeFile(join(root, "domains/tasks/server/internal/store.ts"), "export {};\n");
  await writeFile(join(root, "domains/tasks/domain.json"), JSON.stringify({
    name: "tasks",
    publicApi: { client: [], server: [], shared: ["./shared/public.ts"] },
    dependsOn: [],
  }));
  await writeFile(
    join(root, "apps/web/src/main.ts"),
    "import '@domains/tasks/server/internal/store';\n",
  );
  await writeFile(join(root, "tsconfig.json"), JSON.stringify({
    compilerOptions: { paths: { "@domains/*": ["domains/*"] } },
  }));

  const cliPath = resolve("target/debug/boundra");
  const plugin = boundra({ root, cliPath });
  let received: unknown;
  let publishCount = 0;
  plugin.configureServer({
    ws: { send(payload) { received = payload.data; publishCount += 1; } },
  });
  assert(
    JSON.stringify(received).includes("BR-005"),
    "Vite plugin should publish boundary diagnostics",
  );
  const injected = plugin.transformIndexHtml();
  assert(injected[0]?.attrs.src.includes("virtual:boundra-overlay"), "overlay should be injected");
  assert(plugin.apply === "serve", "overlay must be development-only");
  plugin.handleHotUpdate({ file: join(root, "apps/web/src/+page.svelte") });
  assert(publishCount === 2, "Svelte updates should rerun boundary checks");
  plugin.handleHotUpdate({ file: join(root, "apps/web/src/theme.css") });
  assert(publishCount === 2, "non-source updates should not rerun boundary checks");

  const runtimeSummary = formatBoundraRuntimeSummary({
    name: "BoundraRuntimeError",
    code: "RUNTIME-002",
    contract: "list-projects",
    phase: "result",
    message: "contract result rejected",
    suggestion: "return contract-ready projects",
    issues: [
      { code: "invalid_type", path: ["projects", 0, "id"], message: "expected string" },
      { code: "too_big", path: ["projects"], message: "too many projects" },
    ],
  });
  assert(runtimeSummary.includes("Boundra runtime diagnostics (2)"), "runtime view should count every issue");
  assert(runtimeSummary.includes("projects[0].id"), "runtime view should preserve nested paths");
  assert(runtimeSummary.includes("too many projects"), "runtime view should include later issues");

  const boundarySummary = formatBoundraBoundarySummary({
    source: "boundary",
    diagnostics: [
      {
        rule: "BR-005",
        file: "apps/web/src/main.ts",
        line: 1,
        import: "@domains/tasks/server/internal/store",
        message: "app imported a domain internal path",
        suggestion: "import the domain public API",
      },
      {
        rule: "BR-001",
        file: "domains/tasks/client/view.ts",
        line: 4,
        import: "../server/internal/store",
        message: "client imported server code",
        suggestion: "move the contract to shared",
      },
    ],
  });
  assert(boundarySummary.includes("Boundra boundary diagnostics (2)"), "boundary view should count every violation");
  assert(boundarySummary.includes("BR-001"), "boundary view should include later violations");

  console.log("vite-test: OK");
} finally {
  await rm(root, { recursive: true, force: true });
}
