use std::fs;
use std::path::{Component, Path, PathBuf};

use boundra_core::{load_config, load_project_model, PublicApi};

use crate::output::{print_error, CliDiagnostic};
use crate::util::{display_path, is_kebab_case};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CreateDomainOptions {
    pub(crate) name: String,
    pub(crate) path: Option<PathBuf>,
    pub(crate) root: PathBuf,
}

pub(crate) fn run(options: &CreateDomainOptions) -> i32 {
    if !is_kebab_case(&options.name) {
        print_error(
            &CliDiagnostic::new(
                "DOMAIN-001",
                format!("invalid domain name '{}'", options.name),
                "use a kebab-case name such as 'user-auth'",
            )
            .with_context("domain", &options.name),
        );
        return 2;
    }
    if let Some(path) = &options.path {
        if let Err(message) = validate_parent_path(path) {
            print_error(
                &CliDiagnostic::new(
                    "DOMAIN-005",
                    message,
                    "use a relative kebab-case path such as 'commerce/core'",
                )
                .with_context("path", path.display().to_string()),
            );
            return 2;
        }
    }

    let config = match load_config(&options.root) {
        Ok(config) => config,
        Err(err) => {
            print_error(
                &CliDiagnostic::new(
                    "PROJECT-001",
                    format!("failed to load config: {err}"),
                    "fix boundra.config.json and run the command again",
                )
                .with_context("root", options.root.display().to_string()),
            );
            return 2;
        }
    };

    let domains_root = options.root.join(&config.paths.domains);
    if domains_root.exists() {
        match load_project_model(&options.root) {
            Ok(project) if project.domains.contains_key(&options.name) => {
                let existing = project
                    .domain_root(&options.name)
                    .expect("loaded domain has a root");
                print_error(
                    &CliDiagnostic::new(
                        "DOMAIN-002",
                        format!("domain '{}' already exists", options.name),
                        "choose a new domain name or use the existing domain",
                    )
                    .with_context("path", display_path(&existing)),
                );
                return 2;
            }
            Ok(_) => {}
            Err(err) => {
                print_error(
                    &CliDiagnostic::new(
                        "PROJECT-001",
                        format!("failed to load project: {err}"),
                        "fix the reported config or domain manifest and retry",
                    )
                    .with_context("root", options.root.display().to_string()),
                );
                return 2;
            }
        }
    }
    let domain_parent = match prepare_domain_parent(&domains_root, options.path.as_deref()) {
        Ok(parent) => parent,
        Err(err) => {
            print_error(
                &CliDiagnostic::new(
                    "DOMAIN-006",
                    format!("unsafe domain destination: {err}"),
                    "remove symbolic links below paths.domains or choose another --path",
                )
                .with_context("path", display_path(&domains_root)),
            );
            return 2;
        }
    };
    let domain_root = domain_parent.join(&options.name);

    match fs::symlink_metadata(&domain_root) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            print_error(
                &CliDiagnostic::new(
                    "DOMAIN-006",
                    "unsafe domain destination: target is a symbolic link",
                    "remove the symbolic link or choose another domain name",
                )
                .with_context("path", display_path(&domain_root)),
            );
            return 2;
        }
        Ok(_) => {
            print_error(
                &CliDiagnostic::new(
                    "DOMAIN-002",
                    format!("domain '{}' already exists", options.name),
                    "choose a new domain name or use the existing domain",
                )
                .with_context("path", display_path(&domain_root)),
            );
            return 2;
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
        Err(err) => {
            print_error(
                &CliDiagnostic::new(
                    "DOMAIN-006",
                    format!("failed to validate domain destination: {err}"),
                    "check the destination permissions and symbolic links",
                )
                .with_context("path", display_path(&domain_root)),
            );
            return 2;
        }
    }

    if let Err(err) = scaffold_domain(
        &domain_root,
        &options.name,
        &config.domain.manifest_file,
        &config.domain.public_api,
    ) {
        print_error(
            &CliDiagnostic::new(
                "DOMAIN-003",
                format!("failed to create domain '{}': {err}", options.name),
                "check workspace permissions and remove any partial scaffold before retrying",
            )
            .with_context("path", display_path(&domain_root)),
        );
        return 3;
    }

    println!("create-domain: OK ({})", options.name);
    println!("created: {}", display_path(&domain_root));
    0
}

fn validate_parent_path(path: &Path) -> Result<(), String> {
    if path.as_os_str().is_empty() || path.is_absolute() {
        return Err("domain parent path must be a non-empty relative path".to_string());
    }
    for component in path.components() {
        let Component::Normal(segment) = component else {
            return Err("domain parent path cannot contain '.' or '..'".to_string());
        };
        let Some(segment) = segment.to_str() else {
            return Err("domain parent path must be valid UTF-8".to_string());
        };
        if !is_kebab_case(segment) {
            return Err(format!(
                "domain parent path segment '{segment}' must be kebab-case"
            ));
        }
    }
    Ok(())
}

fn prepare_domain_parent(domains_root: &Path, parent: Option<&Path>) -> std::io::Result<PathBuf> {
    fs::create_dir_all(domains_root)?;
    let mut current = fs::canonicalize(domains_root)?;

    if let Some(parent) = parent {
        for component in parent.components() {
            let Component::Normal(segment) = component else {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "domain parent contains an invalid path component",
                ));
            };
            let next = current.join(segment);
            match fs::symlink_metadata(&next) {
                Ok(metadata) if metadata.file_type().is_symlink() => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!(
                            "symbolic link is not allowed below paths.domains: {}",
                            display_path(&next)
                        ),
                    ));
                }
                Ok(metadata) if !metadata.is_dir() => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!(
                            "domain parent component is not a directory: {}",
                            display_path(&next)
                        ),
                    ));
                }
                Ok(_) => {}
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => fs::create_dir(&next)?,
                Err(err) => return Err(err),
            }
            current = next;
        }
    }

    Ok(current)
}

fn scaffold_domain(
    domain_root: &Path,
    name: &str,
    manifest_file: &str,
    public_api: &PublicApi,
) -> std::io::Result<()> {
    for layer in ["client", "server", "shared", "mcp", "tests"] {
        fs::create_dir_all(domain_root.join(layer))?;
    }

    for public_path in public_api.all_paths() {
        let relative = public_path.strip_prefix("./").unwrap_or(public_path);
        let path = domain_root.join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        if !path.exists() {
            fs::write(&path, "export {};\n")?;
        }
    }

    fs::write(
        domain_root.join(manifest_file),
        domain_manifest_json(name, public_api),
    )?;
    Ok(())
}

fn domain_manifest_json(name: &str, public_api: &PublicApi) -> String {
    format!(
        r#"{{
  "$schema": "https://boundra.dev/schemas/domain-manifest.v1.json",
  "name": "{name}",
  "version": "0.1.0",
  "publicApi": {{
    "client": {client},
    "server": {server},
    "shared": {shared}
  }},
  "dependsOn": [],
  "policies": {{
    "allowCrossDomainServerImport": false,
    "allowMcpWrite": false
  }}
}}
"#,
        client = string_array_json(&public_api.client),
        server = string_array_json(&public_api.server),
        shared = string_array_json(&public_api.shared)
    )
}

fn string_array_json(values: &[String]) -> String {
    let values = values
        .iter()
        .map(|value| format!("\"{}\"", value.replace('"', "\\\"")))
        .collect::<Vec<_>>()
        .join(", ");

    format!("[{values}]")
}
