import { spawnSync } from "node:child_process";
import { existsSync } from "node:fs";
import {
  mkdtemp,
  mkdir,
  realpath,
  readdir,
  readFile,
  rm,
  writeFile,
} from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";

const repositoryRoot = resolve(import.meta.dirname, "..");
const offline = process.argv.includes("--offline");
const cleanRoom = await mkdtemp(join(tmpdir(), "boundra-clean-room-"));
const artifacts = join(cleanRoom, "artifacts");
const project = join(cleanRoom, "project");
const binaryName = process.platform === "win32" ? "boundra.exe" : "boundra";
const binary = join(repositoryRoot, "target", "release", binaryName);

function fileDependency(path) {
  return `file:${path.replaceAll("\\", "/")}`;
}

function run(command, args, options = {}) {
  const result = spawnSync(command, args, {
    cwd: options.cwd ?? repositoryRoot,
    encoding: "utf8",
    env: { ...process.env, ...options.env },
    stdio: options.capture ? "pipe" : "inherit",
  });

  if (result.error) {
    throw result.error;
  }
  if (result.status !== 0) {
    const output = [result.stdout, result.stderr].filter(Boolean).join("\n");
    throw new Error(`${command} ${args.join(" ")} failed\n${output}`);
  }
  return result.stdout ?? "";
}

async function writeJson(path, value) {
  await mkdir(dirname(path), { recursive: true });
  await writeFile(path, `${JSON.stringify(value, null, 2)}\n`);
}

try {
  await mkdir(artifacts, { recursive: true });
  await mkdir(join(project, "src"), { recursive: true });

  run("cargo", ["build", "--release", "-p", "boundra-cli"]);
  if (!existsSync(binary)) {
    throw new Error(`release binary was not created: ${binary}`);
  }

  run("pnpm", [
    "--filter",
    "boundra",
    "pack",
    "--pack-destination",
    artifacts,
  ]);
  const tarballName = (await readdir(artifacts)).find((name) =>
    name.endsWith(".tgz"),
  );
  if (!tarballName) {
    throw new Error("runtime pack did not produce a tarball");
  }
  const tarball = join(artifacts, tarballName);
  const zodSource = offline
    ? fileDependency(
        await realpath(join(repositoryRoot, "node_modules", "zod")),
      )
    : "^4.4.3";
  const typescriptSource = offline
    ? fileDependency(
        await realpath(join(repositoryRoot, "node_modules", "typescript")),
      )
    : "^5.9.3";

  await writeJson(join(project, "package.json"), {
    name: "boundra-clean-room",
    private: true,
    type: "module",
    dependencies: {
      boundra: fileDependency(tarball),
      zod: zodSource,
    },
    devDependencies: {
      typescript: typescriptSource,
    },
  });
  await writeJson(join(project, "tsconfig.json"), {
    compilerOptions: {
      target: "ES2022",
      module: "ESNext",
      moduleResolution: "Bundler",
      strict: true,
      noEmit: true,
      skipLibCheck: true,
    },
    include: ["src/**/*.ts", "domains/**/*.ts"],
  });
  await writeFile(
    join(project, "src", "index.ts"),
    `import { createBoundraClient } from "boundra";
import { getOrderQuery } from "../domains/order/shared/public";

const client = createBoundraClient(async () => ({}));
void client.query(getOrderQuery, {});
`,
  );

  run("pnpm", offline ? ["install", "--offline"] : ["install"], {
    cwd: project,
  });

  run(
    "pnpm",
    ["exec", "boundra", "init", "--root", project, "--name", "boundra-clean-room"],
    { cwd: project, env: { BOUNDRA_CLI_PATH: binary } },
  );

  const cli = (...args) => run(binary, [...args, "--root", project]);
  cli("create-domain", "order");
  cli("create-domain", "billing");
  cli("add-dependency", "billing/order");
  cli("generate", "route", "billing/create-invoice");
  cli("generate", "query", "order/get-order");
  cli("generate", "mutation", "order/submit-order");

  run("pnpm", ["exec", "tsc", "--noEmit"], { cwd: project });

  const boundaryOutput = run(
    binary,
    ["check-boundaries", "--root", project, "--format", "json"],
    { capture: true },
  );
  const boundary = JSON.parse(boundaryOutput);
  if (boundary.status !== "passed" || boundary.meta.violation_count !== 0) {
    throw new Error(`unexpected boundary output: ${boundaryOutput}`);
  }

  const graphOutput = run(
    binary,
    ["graph-domains", "--root", project, "--format", "json"],
    { capture: true },
  );
  const graph = JSON.parse(graphOutput);
  const hasExpectedEdge = graph.edges.some(
    (edge) => edge.from === "billing" && edge.to === "order",
  );
  if (!hasExpectedEdge) {
    throw new Error(`missing billing -> order edge: ${graphOutput}`);
  }

  const packedPackage = JSON.parse(
    await readFile(join(project, "node_modules/boundra/package.json"), "utf8"),
  );
  if (packedPackage.exports?.["."]?.import !== "./dist/index.js") {
    throw new Error("clean-room project did not install the compiled runtime export");
  }

  console.log("clean-room: OK");
} finally {
  if (process.env.BOUNDRA_KEEP_CLEAN_ROOM) {
    console.log(`clean-room preserved: ${cleanRoom}`);
  } else {
    await rm(cleanRoom, { recursive: true, force: true });
  }
}
