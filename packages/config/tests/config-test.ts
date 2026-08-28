import { defineConfig } from "../src/index";

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

assert(config.paths.domains === "domains", "defineConfig should preserve config values");
assert(
  config.checkBoundaries.capabilities.external.react[0] === "ui",
  "defineConfig should preserve literal capability values",
);

if (false) {
  // @ts-expect-error currently unsupported config fields must not type-check
  defineConfig({ graph: { defaultFormat: "mermaid" } });

  defineConfig({
    paths: {
      // @ts-expect-error configured paths must be strings
      domains: 123,
    },
  });
}

console.log("config: OK");
