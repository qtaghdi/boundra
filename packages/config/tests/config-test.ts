import { readFileSync } from "node:fs";

import { defineConfig, type BoundraConfig } from "../src/index";

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(message);
}

const config = defineConfig({
  project: {
    workspaceRoot: ".",
  },
  paths: {
    apps: "apps",
    domains: "domains",
    packages: "packages",
    crates: "crates",
  },
  domain: {
    manifestFile: "domain.json",
    publicApi: {
      client: ["./client/public.ts"],
      server: ["./server/public.ts"],
      shared: ["./shared/public.ts"],
    },
  },
  checkBoundaries: {
    includeExtensions: ["ts", "tsx", "svelte"],
    capabilities: {
      external: {
        react: ["ui"],
        "node:*": ["runtime"],
      },
      packages: {
        db: ["database"],
      },
      apps: ["runtime"],
    },
    policy: {
      shared: {
        denyCapabilities: ["ui", "database", "runtime"],
      },
    },
  },
});

const typedConfig: BoundraConfig = config;
void typedConfig;

assert(config.paths.domains === "domains", "defineConfig should preserve config values");
assert(
  config.checkBoundaries.capabilities.external.react[0] === "ui",
  "defineConfig should preserve literal capability values",
);

const fixturePath = new URL(
  "../../../fixtures/config-conformance/boundra.config.json",
  import.meta.url,
);
const fixture = JSON.parse(readFileSync(fixturePath, "utf8")) as BoundraConfig;
const fixtureConfig = defineConfig(fixture);

assert(
  fixtureConfig.project?.workspaceRoot === "workspace",
  "shared config fixture should be consumable through boundra/config",
);
assert(
  fixtureConfig.checkBoundaries?.policy?.shared?.denyCapabilities?.includes("application") === true,
  "shared config fixture should preserve nested policy values",
);

if (false) {
  const unsupportedConfig: BoundraConfig = {
    // @ts-expect-error currently unsupported config fields must not type-check
    graph: { defaultFormat: "mermaid" },
  };
  void unsupportedConfig;

  defineConfig({
    paths: {
      // @ts-expect-error configured paths must be strings
      domains: 123,
    },
  });
}

console.log("config: OK");
