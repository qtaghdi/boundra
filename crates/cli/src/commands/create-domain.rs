use std::fs;
use std::path::{Path, PathBuf};

use boundra_core::{load_config, PublicApi};

use crate::output::{print_error, CliDiagnostic};
use crate::util::{display_path, is_kebab_case};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CreateDomainOptions {
    pub(crate) name: String,
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
    let domain_root = domains_root.join(&options.name);

    if domain_root.exists() {
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

    if let Err(err) = scaffold_domain(&domain_root, &options.name, &config.domain.public_api) {
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

fn scaffold_domain(domain_root: &Path, name: &str, public_api: &PublicApi) -> std::io::Result<()> {
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
        domain_root.join("domain.json"),
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
