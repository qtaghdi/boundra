import { spawnSync } from "node:child_process";
import { existsSync } from "node:fs";
import { mkdir, mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { performance } from "node:perf_hooks";

function option(name, fallback) {
  const inline = process.argv.find((arg) => arg.startsWith(`${name}=`));
  if (inline) return inline.slice(name.length + 1);
  const index = process.argv.indexOf(name);
  return index >= 0 ? process.argv[index + 1] : fallback;
}

const files = Number(option("--files", "1000"));
const iterations = Number(option("--iterations", "3"));
const binary = resolve(option("--binary", "target/release/boundra"));
const maxMs = Number(option("--max-ms", "0"));
const maxRssMb = Number(option("--max-rss-mb", "0"));

if (!Number.isInteger(files) || files < 1 || !Number.isInteger(iterations) || iterations < 1) {
  throw new Error("--files and --iterations must be positive integers");
}
if (!existsSync(binary)) {
  throw new Error(`CLI not found at ${binary}; run cargo build --release -p boundra-cli`);
}

const root = await mkdtemp(join(tmpdir(), "boundra-benchmark-"));

async function writeJson(path, value) {
  await mkdir(dirname(path), { recursive: true });
  await writeFile(path, `${JSON.stringify(value, null, 2)}\n`);
}

async function createFixture() {
  await writeJson(join(root, "boundra.config.json"), {
    project: { workspaceRoot: "." },
    paths: { apps: "apps", domains: "domains", packages: "packages", crates: "crates" },
  });
  await writeJson(join(root, "tsconfig.json"), {
    compilerOptions: { paths: { "@domains/*": ["domains/*"] } },
  });
  await writeJson(join(root, "domains/catalog/domain.json"), {
    name: "catalog",
    publicApi: { client: [], server: [], shared: ["./shared/public.ts"] },
    dependsOn: [],
  });
  await mkdir(join(root, "domains/catalog/shared"), { recursive: true });
  await writeFile(join(root, "domains/catalog/shared/public.ts"), "export type CatalogId = string;\n");

  const pending = [];
  for (let index = 0; index < files; index += 1) {
    const directory = join(root, "apps/web/src", String(Math.floor(index / 100)));
    pending.push((async () => {
      await mkdir(directory, { recursive: true });
      await writeFile(
        join(directory, `module-${index}.ts`),
        `import type { CatalogId } from "@domains/catalog/shared/public";\nexport const value${index}: CatalogId = "item-${index}";\n`,
      );
    })());
    if (pending.length === 250) await Promise.all(pending.splice(0));
  }
  await Promise.all(pending);
}

function parseRss(stderr) {
  const linux = stderr.match(/Maximum resident set size \(kbytes\):\s+(\d+)/);
  if (linux) return Number(linux[1]) / 1024;
  const mac = stderr.match(/(\d+)\s+maximum resident set size/);
  if (mac) return Number(mac[1]) / 1024 / 1024;
  return null;
}

function runOnce() {
  const args = ["check-boundaries", "--root", root, "--format", "json"];
  const useTime = existsSync("/usr/bin/time") && process.platform !== "win32";
  const command = useTime ? "/usr/bin/time" : binary;
  const commandArgs = useTime
    ? [process.platform === "darwin" ? "-l" : "-v", binary, ...args]
    : args;
  const started = performance.now();
  const result = spawnSync(command, commandArgs, { encoding: "utf8" });
  const durationMs = performance.now() - started;
  let output;
  try {
    output = JSON.parse(result.stdout);
  } catch {
    throw new Error(`benchmark command failed\n${result.stderr}\n${result.stdout}`);
  }
  if (output.status !== "passed") {
    throw new Error(`benchmark fixture produced diagnostics\n${result.stdout}`);
  }
  return { durationMs, rssMb: parseRss(result.stderr), violations: output.meta.violation_count };
}

try {
  await createFixture();
  const runs = Array.from({ length: iterations }, runOnce);
  const warm = runs.slice(1).map((run) => run.durationMs).sort((a, b) => a - b);
  const warmMedian = warm.length === 0
    ? runs[0].durationMs
    : warm[Math.floor(warm.length / 2)];
  const observedRss = runs.map((run) => run.rssMb).filter((value) => value !== null);
  const report = {
    files,
    iterations,
    cold_ms: Number(runs[0].durationMs.toFixed(2)),
    warm_median_ms: Number(warmMedian.toFixed(2)),
    max_rss_mb: observedRss.length ? Number(Math.max(...observedRss).toFixed(2)) : null,
    violations: runs[0].violations,
    environment: { platform: process.platform, arch: process.arch, node: process.version },
  };
  console.log(JSON.stringify(report, null, 2));
  if (maxMs > 0 && report.warm_median_ms > maxMs) {
    throw new Error(`warm median ${report.warm_median_ms}ms exceeded ${maxMs}ms`);
  }
  if (maxRssMb > 0 && report.max_rss_mb !== null && report.max_rss_mb > maxRssMb) {
    throw new Error(`max RSS ${report.max_rss_mb}MB exceeded ${maxRssMb}MB`);
  }
} finally {
  await rm(root, { recursive: true, force: true });
}
