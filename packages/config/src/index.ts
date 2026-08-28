export type BoundraCapability = string;

export interface BoundraProjectConfig {
  workspaceRoot?: string;
}

export interface BoundraProjectPaths {
  apps?: string;
  domains?: string;
  packages?: string;
  crates?: string;
}

export interface BoundraPublicApiConfig {
  client?: readonly string[];
  server?: readonly string[];
  shared?: readonly string[];
}

export interface BoundraDomainConfig {
  manifestFile?: string;
  publicApi?: BoundraPublicApiConfig;
}

export interface BoundraCapabilityConfig {
  external?: Readonly<Record<string, readonly BoundraCapability[]>>;
  packages?: Readonly<Record<string, readonly BoundraCapability[]>>;
  apps?: readonly BoundraCapability[];
}

export interface BoundraSharedPolicyConfig {
  denyCapabilities?: readonly BoundraCapability[];
}

export interface BoundraBoundaryPolicyConfig {
  shared?: BoundraSharedPolicyConfig;
}

export interface BoundraCheckBoundariesConfig {
  includeExtensions?: readonly string[];
  ignore?: readonly string[];
  capabilities?: BoundraCapabilityConfig;
  policy?: BoundraBoundaryPolicyConfig;
}

/**
 * The user-authored Boundra configuration shape currently consumed by the
 * native CLI from `boundra.config.json`.
 *
 * This intentionally contains only fields that the Rust project model reads.
 * Planned fields such as `rules`, `codegen`, and `graph` are not exposed until
 * the CLI implements them.
 */
export interface BoundraConfig {
  project?: BoundraProjectConfig;
  paths?: BoundraProjectPaths;
  domain?: BoundraDomainConfig;
  checkBoundaries?: BoundraCheckBoundariesConfig;
}

/**
 * Preserve literal inference while checking a configuration object against the
 * currently supported Boundra config contract.
 *
 * `defineConfig` does not load files or replace native CLI validation.
 */
export const defineConfig = <const TConfig extends BoundraConfig>(
  config: TConfig,
): TConfig => config;
