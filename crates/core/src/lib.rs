use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::Deserialize;

pub const DEFAULT_BOUNDARY_IGNORE_PATTERNS: &[&str] = &[
    "**/node_modules/**",
    "**/.next/**",
    "**/.turbo/**",
    "**/.claude/worktrees/**",
    "**/dist/**",
    "**/build/**",
    "**/coverage/**",
    "**/target/**",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleCode {
    Br001,
    Br002,
    Br003,
    Br004,
    Br005,
    Br006,
}

impl RuleCode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Br001 => "BR-001",
            Self::Br002 => "BR-002",
            Self::Br003 => "BR-003",
            Self::Br004 => "BR-004",
            Self::Br005 => "BR-005",
            Self::Br006 => "BR-006",
        }
    }
}

impl Display for RuleCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violation {
    pub rule: RuleCode,
    pub file: String,
    pub line: usize,
    pub import_path: String,
    pub message: String,
    pub suggestion: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layer {
    Client,
    Server,
    Shared,
    Mcp,
    Tests,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectModel {
    pub root: PathBuf,
    pub config: BoundraConfig,
    pub domains: BTreeMap<String, DomainManifest>,
    pub path_aliases: Vec<PathAlias>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundraConfig {
    pub project: ProjectConfig,
    pub paths: ProjectPaths,
    pub domain: DomainDefaults,
    pub check_boundaries: CheckBoundariesConfig,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectConfig {
    pub workspace_root: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectPaths {
    pub apps: String,
    pub domains: String,
    pub packages: String,
    pub crates: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DomainDefaults {
    pub manifest_file: String,
    pub public_api: PublicApi,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DomainManifest {
    pub name: String,
    pub public_api: PublicApi,
    pub depends_on: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicApi {
    pub client: Vec<String>,
    pub server: Vec<String>,
    pub shared: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckBoundariesConfig {
    pub include_extensions: Vec<String>,
    pub ignore: Vec<String>,
    pub capabilities: CapabilityConfig,
    pub policy: BoundaryPolicyConfig,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityConfig {
    pub external: BTreeMap<String, Vec<String>>,
    pub packages: BTreeMap<String, Vec<String>>,
    pub apps: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BoundaryPolicyConfig {
    pub shared: LayerCapabilityPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayerCapabilityPolicy {
    pub deny_capabilities: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathAlias {
    pub prefix: String,
    pub target_prefix: String,
}

impl PublicApi {
    pub fn all_paths(&self) -> impl Iterator<Item = &str> {
        self.client
            .iter()
            .chain(self.server.iter())
            .chain(self.shared.iter())
            .map(String::as_str)
    }
}

impl Default for BoundraConfig {
    fn default() -> Self {
        Self {
            project: ProjectConfig {
                workspace_root: ".".to_string(),
            },
            paths: ProjectPaths {
                apps: "apps".to_string(),
                domains: "domains".to_string(),
                packages: "packages".to_string(),
                crates: "crates".to_string(),
            },
            domain: DomainDefaults {
                manifest_file: "domain.json".to_string(),
                public_api: PublicApi::default(),
            },
            check_boundaries: CheckBoundariesConfig::default(),
        }
    }
}

impl Default for CheckBoundariesConfig {
    fn default() -> Self {
        Self {
            include_extensions: vec![
                "ts".to_string(),
                "tsx".to_string(),
                "js".to_string(),
                "jsx".to_string(),
                "svelte".to_string(),
            ],
            ignore: DEFAULT_BOUNDARY_IGNORE_PATTERNS
                .iter()
                .map(|pattern| (*pattern).to_string())
                .collect(),
            capabilities: CapabilityConfig::default(),
            policy: BoundaryPolicyConfig::default(),
        }
    }
}

impl Default for CapabilityConfig {
    fn default() -> Self {
        Self {
            external: BTreeMap::from([
                ("react".to_string(), vec!["ui".to_string()]),
                ("react-dom".to_string(), vec!["ui".to_string()]),
                (
                    "next".to_string(),
                    vec!["ui".to_string(), "runtime".to_string()],
                ),
                ("@prisma/client".to_string(), vec!["database".to_string()]),
                ("fs".to_string(), vec!["runtime".to_string()]),
                ("path".to_string(), vec!["runtime".to_string()]),
                ("crypto".to_string(), vec!["runtime".to_string()]),
                ("child_process".to_string(), vec!["runtime".to_string()]),
                ("stream".to_string(), vec!["runtime".to_string()]),
                ("http".to_string(), vec!["runtime".to_string()]),
                ("https".to_string(), vec!["runtime".to_string()]),
                ("os".to_string(), vec!["runtime".to_string()]),
                ("process".to_string(), vec!["runtime".to_string()]),
                ("node:*".to_string(), vec!["runtime".to_string()]),
            ]),
            packages: BTreeMap::from([
                ("ui".to_string(), vec!["ui".to_string()]),
                ("db".to_string(), vec!["database".to_string()]),
                ("infra".to_string(), vec!["runtime".to_string()]),
            ]),
            apps: vec!["runtime".to_string()],
        }
    }
}

impl Default for LayerCapabilityPolicy {
    fn default() -> Self {
        Self {
            deny_capabilities: vec![
                "ui".to_string(),
                "database".to_string(),
                "runtime".to_string(),
            ],
        }
    }
}

impl Default for PublicApi {
    fn default() -> Self {
        Self {
            client: vec!["./client/public.ts".to_string()],
            server: vec!["./server/public.ts".to_string()],
            shared: vec!["./shared/public.ts".to_string()],
        }
    }
}

pub fn load_project_model(root: &Path) -> io::Result<ProjectModel> {
    let config = load_config(root)?;
    validate_config(root, &config)?;
    let domains = load_domain_manifests(root, &config)?;
    let path_aliases = load_tsconfig_path_aliases(root)?;

    Ok(ProjectModel {
        root: root.to_path_buf(),
        config,
        domains,
        path_aliases,
    })
}

pub fn load_config(root: &Path) -> io::Result<BoundraConfig> {
    let config_path = root.join("boundra.config.json");
    if !config_path.exists() {
        return Ok(BoundraConfig::default());
    }

    let content = fs::read_to_string(&config_path)?;
    let raw = parse_json_file::<RawBoundraConfig>(&config_path, &content)?;

    Ok(raw.into_config())
}

fn validate_config(root: &Path, config: &BoundraConfig) -> io::Result<()> {
    validate_relative_path("project.workspaceRoot", &config.project.workspace_root)?;
    validate_relative_path("paths.apps", &config.paths.apps)?;
    validate_relative_path("paths.domains", &config.paths.domains)?;
    validate_relative_path("paths.packages", &config.paths.packages)?;
    validate_relative_path("paths.crates", &config.paths.crates)?;

    if !root.join(&config.paths.domains).exists() {
        return invalid_data(format!(
            "paths.domains does not exist: {}",
            config.paths.domains
        ));
    }

    if config.domain.manifest_file.is_empty() || config.domain.manifest_file.contains('/') {
        return invalid_data("domain.manifestFile must be a file name");
    }

    for public_path in config.domain.public_api.all_paths() {
        validate_public_api_path(public_path)?;
    }

    if config.check_boundaries.include_extensions.is_empty() {
        return invalid_data("checkBoundaries.includeExtensions must not be empty");
    }

    validate_capability_config(&config.check_boundaries.capabilities)?;
    validate_capability_names(
        "checkBoundaries.policy.shared.denyCapabilities",
        &config.check_boundaries.policy.shared.deny_capabilities,
    )?;

    Ok(())
}

fn validate_relative_path(field: &str, value: &str) -> io::Result<()> {
    if value.is_empty() {
        return invalid_data(format!("{field} must not be empty"));
    }
    if Path::new(value).is_absolute() {
        return invalid_data(format!("{field} must be relative"));
    }
    Ok(())
}

fn validate_capability_config(config: &CapabilityConfig) -> io::Result<()> {
    for (source, capabilities) in &config.external {
        if source.trim().is_empty() {
            return invalid_data("checkBoundaries.capabilities.external keys must not be empty");
        }
        let wildcard_count = source.chars().filter(|value| *value == '*').count();
        if wildcard_count > 0 && (wildcard_count != 1 || !source.ends_with('*')) {
            return invalid_data(format!(
                "external capability matcher must use at most one trailing '*': {source}"
            ));
        }
        validate_capability_names(
            &format!("checkBoundaries.capabilities.external.{source}"),
            capabilities,
        )?;
    }

    for (package, capabilities) in &config.packages {
        if package.trim().is_empty() || package.contains('/') || package.contains('\\') {
            return invalid_data(format!(
                "workspace capability package must be a direct package name: {package}"
            ));
        }
        validate_capability_names(
            &format!("checkBoundaries.capabilities.packages.{package}"),
            capabilities,
        )?;
    }

    validate_capability_names("checkBoundaries.capabilities.apps", &config.apps)
}

fn validate_capability_names(field: &str, capabilities: &[String]) -> io::Result<()> {
    if capabilities
        .iter()
        .any(|capability| capability.trim().is_empty())
    {
        return invalid_data(format!("{field} must not contain empty capability names"));
    }
    Ok(())
}

fn load_domain_manifests(
    root: &Path,
    config: &BoundraConfig,
) -> io::Result<BTreeMap<String, DomainManifest>> {
    let mut domains = BTreeMap::new();
    let domains_root = root.join(&config.paths.domains);

    if !domains_root.exists() {
        return Ok(domains);
    }

    for entry in fs::read_dir(domains_root)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let fallback_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .to_string();
        if fallback_name.is_empty() {
            continue;
        }

        let manifest_path = path.join(&config.domain.manifest_file);
        let has_manifest = manifest_path.exists();
        let manifest = if has_manifest {
            load_domain_manifest(&manifest_path, &fallback_name, &config.domain.public_api)?
        } else {
            DomainManifest {
                name: fallback_name.clone(),
                public_api: config.domain.public_api.clone(),
                depends_on: Vec::new(),
            }
        };

        if has_manifest {
            validate_domain_manifest(&path, &fallback_name, &manifest)?;
        }
        domains.insert(manifest.name.clone(), manifest);
    }

    validate_domain_dependencies(&domains)?;
    Ok(domains)
}

pub fn load_domain_manifest(
    path: &Path,
    fallback_name: &str,
    default_public_api: &PublicApi,
) -> io::Result<DomainManifest> {
    let content = fs::read_to_string(path)?;
    let raw = parse_json_file::<RawDomainManifest>(path, &content)?;

    Ok(raw.into_manifest(fallback_name, default_public_api))
}

fn validate_domain_manifest(
    domain_root: &Path,
    folder_name: &str,
    manifest: &DomainManifest,
) -> io::Result<()> {
    if manifest.name != folder_name {
        return invalid_data(format!(
            "domain manifest name '{}' must match folder name '{}'",
            manifest.name, folder_name
        ));
    }

    for public_path in manifest.public_api.all_paths() {
        validate_public_api_path(public_path)?;
        let relative = public_path.strip_prefix("./").unwrap_or(public_path);
        let file_path = domain_root.join(relative);
        if !file_path.exists() {
            return invalid_data(format!(
                "public API path does not exist for domain '{}': {}",
                manifest.name, public_path
            ));
        }
    }

    Ok(())
}

fn validate_domain_dependencies(domains: &BTreeMap<String, DomainManifest>) -> io::Result<()> {
    for manifest in domains.values() {
        for dependency in &manifest.depends_on {
            if !domains.contains_key(dependency) {
                return invalid_data(format!(
                    "domain '{}' depends on unknown domain '{}'",
                    manifest.name, dependency
                ));
            }
        }
    }

    if let Some(cycle) = find_domain_dependency_cycle(domains) {
        return invalid_data(format!(
            "domain dependency cycle detected: {}",
            cycle.join(" -> ")
        ));
    }

    Ok(())
}

pub fn find_domain_dependency_cycle(
    domains: &BTreeMap<String, DomainManifest>,
) -> Option<Vec<String>> {
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum VisitState {
        Visiting,
        Visited,
    }

    fn visit(
        domain: &str,
        domains: &BTreeMap<String, DomainManifest>,
        states: &mut BTreeMap<String, VisitState>,
        stack: &mut Vec<String>,
    ) -> Option<Vec<String>> {
        if states.get(domain) == Some(&VisitState::Visited) {
            return None;
        }
        if states.get(domain) == Some(&VisitState::Visiting) {
            let start = stack.iter().position(|entry| entry == domain).unwrap_or(0);
            let mut cycle = stack[start..].to_vec();
            cycle.push(domain.to_string());
            return Some(cycle);
        }

        states.insert(domain.to_string(), VisitState::Visiting);
        stack.push(domain.to_string());

        if let Some(manifest) = domains.get(domain) {
            for dependency in &manifest.depends_on {
                if let Some(cycle) = visit(dependency, domains, states, stack) {
                    return Some(cycle);
                }
            }
        }

        stack.pop();
        states.insert(domain.to_string(), VisitState::Visited);
        None
    }

    let mut states = BTreeMap::new();
    let mut stack = Vec::new();
    for domain in domains.keys() {
        if let Some(cycle) = visit(domain, domains, &mut states, &mut stack) {
            return Some(cycle);
        }
    }

    None
}

fn load_tsconfig_path_aliases(root: &Path) -> io::Result<Vec<PathAlias>> {
    let workspace_root = absolute_normalized_path(root)?;
    let tsconfig_path = workspace_root.join("tsconfig.json");
    if !tsconfig_path.exists() {
        return Ok(Vec::new());
    }

    let mut aliases = BTreeMap::new();
    let mut visiting = Vec::new();
    load_tsconfig_aliases_recursive(&workspace_root, &tsconfig_path, &mut visiting, &mut aliases)?;

    let mut aliases = aliases.into_values().collect::<Vec<_>>();
    // More specific aliases must be matched before broad aliases.
    aliases.sort_by_key(|alias| std::cmp::Reverse(alias.prefix.len()));
    Ok(aliases)
}

fn load_tsconfig_aliases_recursive(
    workspace_root: &Path,
    tsconfig_path: &Path,
    visiting: &mut Vec<PathBuf>,
    aliases: &mut BTreeMap<String, PathAlias>,
) -> io::Result<()> {
    let tsconfig_path = normalize_filesystem_path(tsconfig_path);
    if let Some(start) = visiting.iter().position(|path| path == &tsconfig_path) {
        let mut cycle = visiting[start..]
            .iter()
            .map(|path| display_path(path))
            .collect::<Vec<_>>();
        cycle.push(display_path(&tsconfig_path));
        return invalid_data(format!(
            "cyclic tsconfig extends chain: {}",
            cycle.join(" -> ")
        ));
    }

    visiting.push(tsconfig_path.clone());
    let raw = parse_tsconfig(&tsconfig_path)?;

    if let Some(extends) = &raw.extends {
        for parent in extends.values() {
            if !is_relative_or_absolute_tsconfig_reference(parent) {
                continue;
            }
            let parent_path = resolve_tsconfig_reference(&tsconfig_path, parent)?;
            load_tsconfig_aliases_recursive(workspace_root, &parent_path, visiting, aliases)?;
        }
    }

    if let Some(compiler_options) = raw.compiler_options {
        let base_url = compiler_options.base_url.as_deref().unwrap_or(".");
        if let Some(paths) = compiler_options.paths {
            for (alias, targets) in paths {
                let Some(target) = targets.first() else {
                    continue;
                };
                let prefix = alias.strip_suffix('*').unwrap_or(&alias).to_string();
                if prefix.is_empty() {
                    continue;
                }
                let Some(target_prefix) =
                    normalize_alias_target(workspace_root, &tsconfig_path, base_url, target)
                else {
                    continue;
                };
                aliases.insert(
                    prefix.clone(),
                    PathAlias {
                        prefix,
                        target_prefix,
                    },
                );
            }
        }
    }

    visiting.pop();
    Ok(())
}

fn parse_tsconfig(path: &Path) -> io::Result<RawTsConfig> {
    let content = fs::read_to_string(path)?;
    let parse_options = jsonc_parser::ParseOptions {
        allow_comments: true,
        allow_loose_object_property_names: false,
        allow_trailing_commas: true,
        allow_missing_commas: false,
        allow_single_quoted_strings: false,
        allow_hexadecimal_numbers: false,
        allow_unary_plus_numbers: false,
    };
    jsonc_parser::parse_to_serde_value::<RawTsConfig>(&content, &parse_options).map_err(|err| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("invalid JSONC in {}: {err}", display_path(path)),
        )
    })
}

fn is_relative_or_absolute_tsconfig_reference(reference: &str) -> bool {
    reference.starts_with('.') || Path::new(reference).is_absolute()
}

fn resolve_tsconfig_reference(tsconfig_path: &Path, reference: &str) -> io::Result<PathBuf> {
    let config_dir = tsconfig_path.parent().unwrap_or_else(|| Path::new("."));
    let mut candidate = if Path::new(reference).is_absolute() {
        PathBuf::from(reference)
    } else {
        config_dir.join(reference)
    };
    candidate = normalize_filesystem_path(&candidate);

    let candidates = if candidate.extension().is_some() {
        vec![candidate]
    } else {
        vec![
            candidate.clone(),
            candidate.with_extension("json"),
            candidate.join("tsconfig.json"),
        ]
    };
    candidates
        .into_iter()
        .find(|path| path.is_file())
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "extended tsconfig does not exist: {reference} (from {})",
                    display_path(tsconfig_path)
                ),
            )
        })
}

fn normalize_alias_target(
    workspace_root: &Path,
    tsconfig_path: &Path,
    base_url: &str,
    target: &str,
) -> Option<String> {
    let target_without_wildcard = target.strip_suffix('*').unwrap_or(target);
    let keep_trailing_separator =
        target_without_wildcard.ends_with('/') || target_without_wildcard.ends_with('\\');
    let config_dir = tsconfig_path.parent().unwrap_or_else(|| Path::new("."));
    let absolute_target =
        normalize_filesystem_path(&config_dir.join(base_url).join(target_without_wildcard));
    let relative = absolute_target.strip_prefix(workspace_root).ok()?;
    let mut normalized = display_path(relative);
    if keep_trailing_separator && !normalized.ends_with('/') {
        normalized.push('/');
    }
    Some(normalized)
}

fn absolute_normalized_path(path: &Path) -> io::Result<PathBuf> {
    if path.is_absolute() {
        return Ok(normalize_filesystem_path(path));
    }
    Ok(normalize_filesystem_path(
        &std::env::current_dir()?.join(path),
    ))
}

fn normalize_filesystem_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    normalized
}

fn validate_public_api_path(path: &str) -> io::Result<()> {
    if path.is_empty() {
        return invalid_data("public API path must not be empty");
    }
    if Path::new(path).is_absolute() {
        return invalid_data(format!("public API path must be relative: {path}"));
    }
    if path.contains("/internal/")
        || path.starts_with("internal/")
        || path.starts_with("./internal/")
    {
        return invalid_data(format!(
            "public API path must not expose internal paths: {path}"
        ));
    }
    Ok(())
}

fn invalid_data<T>(message: impl Into<String>) -> io::Result<T> {
    Err(io::Error::new(io::ErrorKind::InvalidData, message.into()))
}

fn parse_json_file<'a, T>(path: &Path, content: &'a str) -> io::Result<T>
where
    T: Deserialize<'a>,
{
    serde_json::from_str(content).map_err(|err| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("invalid JSON in {}: {err}", display_path(path)),
        )
    })
}

fn display_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawBoundraConfig {
    project: Option<RawProjectConfig>,
    paths: Option<RawProjectPaths>,
    domain: Option<RawDomainDefaults>,
    check_boundaries: Option<RawCheckBoundariesConfig>,
}

impl RawBoundraConfig {
    fn into_config(self) -> BoundraConfig {
        let mut config = BoundraConfig::default();

        if let Some(project) = self.project {
            if let Some(workspace_root) = project.workspace_root {
                config.project.workspace_root = workspace_root;
            }
        }

        if let Some(paths) = self.paths {
            if let Some(apps) = paths.apps {
                config.paths.apps = apps;
            }
            if let Some(domains) = paths.domains {
                config.paths.domains = domains;
            }
            if let Some(packages) = paths.packages {
                config.paths.packages = packages;
            }
            if let Some(crates) = paths.crates {
                config.paths.crates = crates;
            }
        }

        if let Some(domain) = self.domain {
            if let Some(manifest_file) = domain.manifest_file {
                config.domain.manifest_file = manifest_file;
            }
            if let Some(public_api) = domain.public_api {
                config.domain.public_api = public_api.into_public_api_with_default(&PublicApi {
                    client: Vec::new(),
                    server: Vec::new(),
                    shared: Vec::new(),
                });
            }
        }

        if let Some(check_boundaries) = self.check_boundaries {
            if let Some(include_extensions) = check_boundaries.include_extensions {
                config.check_boundaries.include_extensions = include_extensions;
            }
            if let Some(ignore) = check_boundaries.ignore {
                config.check_boundaries.ignore = ignore;
            }
            if let Some(capabilities) = check_boundaries.capabilities {
                if let Some(external) = capabilities.external {
                    config
                        .check_boundaries
                        .capabilities
                        .external
                        .extend(external);
                }
                if let Some(packages) = capabilities.packages {
                    config
                        .check_boundaries
                        .capabilities
                        .packages
                        .extend(packages);
                }
                if let Some(apps) = capabilities.apps {
                    config.check_boundaries.capabilities.apps = apps;
                }
            }
            if let Some(policy) = check_boundaries.policy {
                if let Some(shared) = policy.shared {
                    if let Some(deny_capabilities) = shared.deny_capabilities {
                        config.check_boundaries.policy.shared.deny_capabilities = deny_capabilities;
                    }
                }
            }
        }

        config
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawProjectConfig {
    workspace_root: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct RawProjectPaths {
    apps: Option<String>,
    domains: Option<String>,
    packages: Option<String>,
    crates: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawDomainDefaults {
    manifest_file: Option<String>,
    public_api: Option<RawPublicApi>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawCheckBoundariesConfig {
    include_extensions: Option<Vec<String>>,
    ignore: Option<Vec<String>>,
    capabilities: Option<RawCapabilityConfig>,
    policy: Option<RawBoundaryPolicyConfig>,
}

#[derive(Debug, Default, Deserialize)]
struct RawCapabilityConfig {
    external: Option<BTreeMap<String, Vec<String>>>,
    packages: Option<BTreeMap<String, Vec<String>>>,
    apps: Option<Vec<String>>,
}

#[derive(Debug, Default, Deserialize)]
struct RawBoundaryPolicyConfig {
    shared: Option<RawLayerCapabilityPolicy>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawLayerCapabilityPolicy {
    deny_capabilities: Option<Vec<String>>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawDomainManifest {
    name: Option<String>,
    public_api: Option<RawPublicApi>,
    depends_on: Option<Vec<String>>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawTsConfig {
    extends: Option<RawTsConfigExtends>,
    compiler_options: Option<RawCompilerOptions>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum RawTsConfigExtends {
    One(String),
    Many(Vec<String>),
}

impl RawTsConfigExtends {
    fn values(&self) -> Vec<&str> {
        match self {
            Self::One(value) => vec![value],
            Self::Many(values) => values.iter().map(String::as_str).collect(),
        }
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawCompilerOptions {
    base_url: Option<String>,
    paths: Option<BTreeMap<String, Vec<String>>>,
}

impl RawDomainManifest {
    fn into_manifest(self, fallback_name: &str, default_public_api: &PublicApi) -> DomainManifest {
        DomainManifest {
            name: self.name.unwrap_or_else(|| fallback_name.to_string()),
            public_api: self
                .public_api
                .map(|public_api| public_api.into_public_api_with_default(default_public_api))
                .unwrap_or_else(|| default_public_api.clone()),
            depends_on: self.depends_on.unwrap_or_default(),
        }
    }
}

#[derive(Debug, Default, Deserialize)]
struct RawPublicApi {
    client: Option<Vec<String>>,
    server: Option<Vec<String>>,
    shared: Option<Vec<String>>,
}

impl RawPublicApi {
    fn into_public_api_with_default(self, default_public_api: &PublicApi) -> PublicApi {
        PublicApi {
            client: self
                .client
                .unwrap_or_else(|| default_public_api.client.clone()),
            server: self
                .server
                .unwrap_or_else(|| default_public_api.server.clone()),
            shared: self
                .shared
                .unwrap_or_else(|| default_public_api.shared.clone()),
        }
    }
}
