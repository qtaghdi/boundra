use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

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

    Ok(ProjectModel {
        root: root.to_path_buf(),
        config,
        domains,
    })
}

pub fn load_config(root: &Path) -> io::Result<BoundraConfig> {
    let config_path = root.join("boundra.config.json");
    if !config_path.exists() {
        return Ok(BoundraConfig::default());
    }

    let content = fs::read_to_string(config_path)?;
    let mut config = BoundraConfig::default();

    if let Some(workspace_root) = extract_nested_string(&content, "project", "workspaceRoot") {
        config.project.workspace_root = workspace_root;
    }

    if let Some(apps) = extract_nested_string(&content, "paths", "apps") {
        config.paths.apps = apps;
    }

    if let Some(domains) = extract_nested_string(&content, "paths", "domains") {
        config.paths.domains = domains;
    }

    if let Some(packages) = extract_nested_string(&content, "paths", "packages") {
        config.paths.packages = packages;
    }

    if let Some(crates) = extract_nested_string(&content, "paths", "crates") {
        config.paths.crates = crates;
    }

    if let Some(manifest_file) = extract_string_field(&content, "manifestFile") {
        config.domain.manifest_file = manifest_file;
    }

    if let Some(public_api) = extract_public_api(&content) {
        config.domain.public_api = public_api;
    }

    if let Some(check_boundaries) = extract_object(&content, "checkBoundaries") {
        if let Some(include_extensions) =
            extract_string_array(check_boundaries, "includeExtensions")
        {
            config.check_boundaries.include_extensions = include_extensions;
        }
        if let Some(ignore) = extract_string_array(check_boundaries, "ignore") {
            config.check_boundaries.ignore = ignore;
        }
    }

    Ok(config)
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
    let name = extract_string_field(&content, "name").unwrap_or_else(|| fallback_name.to_string());
    let public_api = extract_public_api(&content).unwrap_or_else(|| default_public_api.clone());
    let depends_on = extract_string_array(&content, "dependsOn").unwrap_or_default();

    Ok(DomainManifest {
        name,
        public_api,
        depends_on,
    })
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

fn extract_public_api(content: &str) -> Option<PublicApi> {
    let object = extract_object(content, "publicApi")?;
    Some(PublicApi {
        client: extract_string_array(object, "client").unwrap_or_default(),
        server: extract_string_array(object, "server").unwrap_or_default(),
        shared: extract_string_array(object, "shared").unwrap_or_default(),
    })
}

fn extract_nested_string(content: &str, object_key: &str, field_key: &str) -> Option<String> {
    let object = extract_object(content, object_key)?;
    extract_string_field(object, field_key)
}

fn extract_object<'a>(content: &'a str, key: &str) -> Option<&'a str> {
    let key_index = content.find(&format!("\"{key}\""))?;
    let after_key = &content[key_index..];
    let object_start = after_key.find('{')?;
    let object = &after_key[object_start..];
    let mut depth = 0;

    for (index, char) in object.char_indices() {
        match char {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(&object[..=index]);
                }
            }
            _ => {}
        }
    }

    None
}

fn extract_string_field(content: &str, key: &str) -> Option<String> {
    let key_index = content.find(&format!("\"{key}\""))?;
    let after_key = &content[key_index..];
    let colon_index = after_key.find(':')?;
    let after_colon = after_key[colon_index + 1..].trim_start();
    extract_quoted_string(after_colon)
}

fn extract_string_array(content: &str, key: &str) -> Option<Vec<String>> {
    let key_index = content.find(&format!("\"{key}\""))?;
    let after_key = &content[key_index..];
    let array_start = after_key.find('[')?;
    let array = &after_key[array_start + 1..];
    let array_end = array.find(']')?;
    let array_content = &array[..array_end];
    let mut values = Vec::new();
    let mut rest = array_content;

    while let Some(start) = rest.find('"') {
        let after_start = &rest[start + 1..];
        let Some(end) = after_start.find('"') else {
            break;
        };
        values.push(after_start[..end].to_string());
        rest = &after_start[end + 1..];
    }

    Some(values)
}

fn extract_quoted_string(input: &str) -> Option<String> {
    let start = input.find('"')?;
    let after_start = &input[start + 1..];
    let end = after_start.find('"')?;
    Some(after_start[..end].to_string())
}

fn invalid_data<T>(message: impl Into<String>) -> io::Result<T> {
    Err(io::Error::new(io::ErrorKind::InvalidData, message.into()))
}
