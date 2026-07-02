use std::fs;
use std::path::PathBuf;

use crate::output::{print_error, CliDiagnostic};
use crate::util::{display_path, is_kebab_case};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct InitOptions {
    pub(crate) name: Option<String>,
    pub(crate) root: PathBuf,
}

pub(crate) fn run(options: &InitOptions) -> i32 {
    let name = options.name.clone().unwrap_or_else(|| {
        options
            .root
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("boundra-app")
            .to_string()
    });

    if !is_kebab_case(&name) {
        print_error(
            &CliDiagnostic::new(
                "PROJECT-004",
                format!("invalid project name '{name}'"),
                "pass --name with a kebab-case value such as 'order-app'",
            )
            .with_context("project", &name),
        );
        return 2;
    }

    let config_path = options.root.join("boundra.config.json");
    if config_path.exists() {
        print_error(
            &CliDiagnostic::new(
                "PROJECT-003",
                "boundra.config.json already exists",
                "use the existing Boundra workspace or remove the config intentionally before retrying",
            )
            .with_context("path", display_path(&config_path)),
        );
        return 2;
    }

    let result = (|| -> std::io::Result<()> {
        fs::create_dir_all(&options.root)?;
        fs::create_dir_all(options.root.join("apps"))?;
        fs::create_dir_all(options.root.join("domains"))?;
        fs::write(&config_path, config_json(&name))?;
        Ok(())
    })();

    if let Err(err) = result {
        print_error(
            &CliDiagnostic::new(
                "PROJECT-005",
                format!("failed to initialize project: {err}"),
                "check the target path permissions and retry",
            )
            .with_context("root", display_path(&options.root)),
        );
        return 3;
    }

    println!("init: OK ({name})");
    println!("created: {}", display_path(&config_path));
    0
}

fn config_json(name: &str) -> String {
    format!(
        r#"{{
  "$schema": "https://boundra.dev/schemas/boundra-config.v1.json",
  "version": 1,
  "project": {{
    "name": "{name}",
    "workspaceRoot": "."
  }},
  "paths": {{
    "apps": "apps",
    "domains": "domains",
    "packages": "packages",
    "crates": "crates"
  }},
  "domain": {{
    "manifestFile": "domain.json",
    "publicApi": {{
      "client": ["./client/public.ts"],
      "server": ["./server/public.ts"],
      "shared": ["./shared/public.ts"]
    }}
  }},
  "checkBoundaries": {{
    "includeExtensions": ["ts", "tsx", "js", "jsx"],
    "ignore": [
      "**/node_modules/**",
      "**/dist/**",
      "**/build/**",
      "**/coverage/**",
      "**/target/**"
    ]
  }}
}}
"#
    )
}
