use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleCode {
    Br001,
    Br002,
    Br003,
    Br004,
}

impl RuleCode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Br001 => "BR-001",
            Self::Br002 => "BR-002",
            Self::Br003 => "BR-003",
            Self::Br004 => "BR-004",
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
            check_boundaries: CheckBoundariesConfig {
                include_extensions: vec![
                    "ts".to_string(),
                    "tsx".to_string(),
                    "js".to_string(),
                    "jsx".to_string(),
                ],
                ignore: vec![
                    "**/node_modules/**".to_string(),
                    "**/dist/**".to_string(),
                    "**/build/**".to_string(),
                    "**/coverage/**".to_string(),
                    "**/target/**".to_string(),
                ],
            },
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

    Ok(())
}

fn load_tsconfig_path_aliases(root: &Path) -> io::Result<Vec<PathAlias>> {
    let tsconfig_path = root.join("tsconfig.json");
    if !tsconfig_path.exists() {
        return Ok(Vec::new());
    }

    // TypeScript의 compilerOptions.paths를 Boundra 내부 경로로 해석하기 위한 준비 단계다.
    // 예: "@domains/*": ["domains/*"] -> prefix "@domains/", target_prefix "domains/"
    let content = fs::read_to_string(&tsconfig_path)?;
    let raw = parse_json_file::<RawTsConfig>(&tsconfig_path, &content)?;
    let Some(compiler_options) = raw.compiler_options else {
        return Ok(Vec::new());
    };
    let Some(paths) = compiler_options.paths else {
        return Ok(Vec::new());
    };

    let mut aliases = Vec::new();
    for (alias, targets) in paths {
        let Some(target) = targets.first() else {
            continue;
        };
        let (prefix, target_prefix) = normalize_alias_pair(&alias, target);
        if !prefix.is_empty() {
            aliases.push(PathAlias {
                prefix,
                target_prefix,
            });
        }
    }

    // 더 구체적인 alias가 먼저 매칭되도록 긴 prefix를 우선한다.
    // 예: "@/domains/*"가 "@/*"보다 먼저 처리되어야 한다.
    aliases.sort_by(|left, right| right.prefix.len().cmp(&left.prefix.len()));
    Ok(aliases)
}

fn normalize_alias_pair(alias: &str, target: &str) -> (String, String) {
    // paths 패턴의 '*'와 './'를 제거해 단순 prefix 치환 형태로 바꾼다.
    let prefix = alias.strip_suffix('*').unwrap_or(alias).to_string();
    let target_prefix = target
        .strip_prefix("./")
        .unwrap_or(target)
        .strip_suffix('*')
        .unwrap_or_else(|| target.strip_prefix("./").unwrap_or(target))
        .to_string();

    (prefix, target_prefix)
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
    compiler_options: Option<RawCompilerOptions>,
}

#[derive(Debug, Default, Deserialize)]
struct RawCompilerOptions {
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
