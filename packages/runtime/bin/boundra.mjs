#!/usr/bin/env node

import { createHash } from "node:crypto";
import { chmod, mkdir, readFile, writeFile } from "node:fs/promises";
import { homedir } from "node:os";
import { dirname, join } from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const packageRoot = dirname(dirname(fileURLToPath(import.meta.url)));
const packageJson = JSON.parse(await readFile(join(packageRoot, "package.json"), "utf8"));
const version = packageJson.version;

function fail(message) {
  console.error(`[BOUNDRA_CLI] ${message}`);
  process.exit(1);
}

function platformTarget() {
  const key = `${process.platform}-${process.arch}`;
  const targets = {
    "darwin-arm64": ["aarch64-apple-darwin", "tar.gz"],
    "darwin-x64": ["x86_64-apple-darwin", "tar.gz"],
    "linux-x64": ["x86_64-unknown-linux-gnu", "tar.gz"],
    "win32-x64": ["x86_64-pc-windows-msvc", "zip"],
  };
  return targets[key];
}

async function download(url) {
  const response = await fetch(url, { redirect: "follow" });
  if (!response.ok) {
    throw new Error(`download failed (${response.status}) for ${url}`);
  }
  return Buffer.from(await response.arrayBuffer());
}

async function resolveNativeCli() {
  if (process.env.BOUNDRA_CLI_PATH) {
    return process.env.BOUNDRA_CLI_PATH;
  }

  const platform = platformTarget();
  if (!platform) {
    throw new Error(`unsupported platform: ${process.platform}-${process.arch}`);
  }
  const [target, extension] = platform;
  const name = `boundra-${version}-${target}`;
  const executable = process.platform === "win32" ? "boundra.exe" : "boundra";
  const cacheRoot = process.env.BOUNDRA_CACHE_DIR
    ?? (process.platform === "win32" && process.env.LOCALAPPDATA
      ? join(process.env.LOCALAPPDATA, "boundra")
      : join(process.env.XDG_CACHE_HOME ?? join(homedir(), ".cache"), "boundra"));
  const releaseDir = join(cacheRoot, version);
  const binaryPath = join(releaseDir, name, executable);

  try {
    await readFile(binaryPath);
    return binaryPath;
  } catch {
    // Cache miss: download the matching signed release artifact below.
  }

  await mkdir(releaseDir, { recursive: true });
  const baseUrl = `https://github.com/qtaghdi/boundra/releases/download/v${version}`;
  const archiveName = `${name}.${extension}`;
  const [archive, checksums] = await Promise.all([
    download(`${baseUrl}/${archiveName}`),
    download(`${baseUrl}/checksums-sha256.txt`),
  ]);
  const expected = checksums
    .toString("utf8")
    .split(/\r?\n/)
    .map((line) => line.trim().split(/\s+/))
    .find((parts) => parts.at(-1) === archiveName)?.[0];
  if (!expected) {
    throw new Error(`checksum entry missing for ${archiveName}`);
  }
  const actual = createHash("sha256").update(archive).digest("hex");
  if (actual !== expected) {
    throw new Error(`checksum mismatch for ${archiveName}`);
  }

  const archivePath = join(releaseDir, archiveName);
  await writeFile(archivePath, archive);
  const extraction = process.platform === "win32"
    ? spawnSync("powershell", ["-NoProfile", "-Command", "Expand-Archive", "-LiteralPath", archivePath, "-DestinationPath", releaseDir, "-Force"], { stdio: "inherit" })
    : spawnSync("tar", ["-xzf", archivePath, "-C", releaseDir], { stdio: "inherit" });
  if (extraction.error || extraction.status !== 0) {
    throw new Error(`failed to extract ${archiveName}`);
  }
  if (process.platform !== "win32") {
    await chmod(binaryPath, 0o755);
  }
  return binaryPath;
}

try {
  const binary = await resolveNativeCli();
  const result = spawnSync(binary, process.argv.slice(2), { stdio: "inherit" });
  if (result.error) throw result.error;
  process.exit(result.status ?? 1);
} catch (error) {
  fail(error instanceof Error ? error.message : String(error));
}
