import { mkdir, mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";

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
  plugin.configureServer({
    ws: { send(payload) { received = payload.data; } },
  });
  assert(
    JSON.stringify(received).includes("BR-005"),
    "Vite plugin should publish boundary diagnostics",
  );
  const injected = plugin.transformIndexHtml();
  assert(injected[0]?.attrs.src.includes("virtual:boundra-overlay"), "overlay should be injected");
  assert(plugin.apply === "serve", "overlay must be development-only");

  console.log("vite-test: OK");
} finally {
  await rm(root, { recursive: true, force: true });
}
