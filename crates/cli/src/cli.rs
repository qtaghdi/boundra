use std::fs;
use std::path::{Path, PathBuf};

use boundra_core::{load_config, load_project_model, PublicApi, Violation};
use boundra_parser::{collect_imports_with_options, ScanOptions};
use boundra_rules::{check_boundaries_with_context, BoundaryContext};

#[derive(Debug, Clone, PartialEq, Eq)]
enum Command {
    CheckBoundaries(CheckBoundariesOptions),
    CreateDomain(CreateDomainOptions),
    Help,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CheckBoundariesOptions {
    format: OutputFormat,
    root: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CreateDomainOptions {
    name: String,
    root: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OutputFormat {
    Text,
    Json,
}

pub fn run(args: &[String]) -> i32 {
    let command = match parse_command(args) {
        Ok(command) => command,
        Err(err) => {
            eprintln!("{err}");
            print_help();
            return 2;
        }
    };

    match command {
        Command::CheckBoundaries(options) => run_check_boundaries(&options),
        Command::CreateDomain(options) => run_create_domain(&options),
        Command::Help => {
            print_help();
            0
        }
    }
}

fn parse_command(args: &[String]) -> Result<Command, String> {
    let Some(command) = args.first() else {
        return Err("missing command".to_string());
    };

    match command.as_str() {
        "check-boundaries" => {
            let options = parse_check_boundaries_options(&args[1..])?;
            Ok(Command::CheckBoundaries(options))
        }
        "create-domain" => {
            let options = parse_create_domain_options(&args[1..])?;
            Ok(Command::CreateDomain(options))
        }
        "help" | "--help" | "-h" => Ok(Command::Help),
        _ => Err(format!("unknown command: {command}")),
    }
}

fn parse_check_boundaries_options(args: &[String]) -> Result<CheckBoundariesOptions, String> {
    let mut options = CheckBoundariesOptions {
        format: OutputFormat::Text,
        root: PathBuf::from("."),
    };
    let mut index = 0;

    while index < args.len() {
        let arg = &args[index];

        if arg == "--format" {
            let Some(value) = args.get(index + 1) else {
                return Err("missing value for --format".to_string());
            };
            options.format = parse_output_format(value)?;
            index += 2;
            continue;
        }

        if let Some(value) = arg.strip_prefix("--format=") {
            options.format = parse_output_format(value)?;
            index += 1;
            continue;
        }

        if arg == "--root" {
            let Some(value) = args.get(index + 1) else {
                return Err("missing value for --root".to_string());
            };
            options.root = PathBuf::from(value);
            index += 2;
            continue;
        }

        if let Some(value) = arg.strip_prefix("--root=") {
            options.root = PathBuf::from(value);
            index += 1;
            continue;
        }

        return Err(format!("unknown option: {arg}"));
    }

    Ok(options)
}

fn parse_create_domain_options(args: &[String]) -> Result<CreateDomainOptions, String> {
    let Some(name) = args.first() else {
        return Err("missing domain name".to_string());
    };

    if name.starts_with('-') {
        return Err("missing domain name".to_string());
    }

    let mut options = CreateDomainOptions {
        name: name.clone(),
        root: PathBuf::from("."),
    };
    let mut index = 1;

    while index < args.len() {
        let arg = &args[index];

        if arg == "--root" {
            let Some(value) = args.get(index + 1) else {
                return Err("missing value for --root".to_string());
            };
            options.root = PathBuf::from(value);
            index += 2;
            continue;
        }

        if let Some(value) = arg.strip_prefix("--root=") {
            options.root = PathBuf::from(value);
            index += 1;
            continue;
        }

        return Err(format!("unknown option: {arg}"));
    }

    Ok(options)
}

fn parse_output_format(value: &str) -> Result<OutputFormat, String> {
    match value {
        "text" => Ok(OutputFormat::Text),
        "json" => Ok(OutputFormat::Json),
        _ => Err(format!("invalid --format value: {value}")),
    }
}

fn run_check_boundaries(options: &CheckBoundariesOptions) -> i32 {
    let project = match load_project_model(&options.root) {
        Ok(project) => project,
        Err(err) => {
            eprintln!("failed to load project: {err}");
            return 2;
        }
    };
    let scan_options = ScanOptions {
        include_extensions: project.config.check_boundaries.include_extensions.clone(),
        ignore: project.config.check_boundaries.ignore.clone(),
    };
    let imports = match collect_imports_with_options(&options.root, &scan_options) {
        Ok(v) => v,
        Err(err) => {
            eprintln!("failed to scan project: {err}");
            return 3;
        }
    };

    let violations = check_boundaries_with_context(
        &imports,
        &BoundaryContext {
            domains: project.domains,
        },
    );

    match options.format {
        OutputFormat::Text => print_text(&violations),
        OutputFormat::Json => print_json(&violations),
    }

    if violations.is_empty() {
        0
    } else {
        1
    }
}

fn run_create_domain(options: &CreateDomainOptions) -> i32 {
    if !is_kebab_case(&options.name) {
        eprintln!("invalid domain name: {}", options.name);
        eprintln!("domain names must use kebab-case");
        return 2;
    }

    let config = match load_config(&options.root) {
        Ok(config) => config,
        Err(err) => {
            eprintln!("failed to load config: {err}");
            return 2;
        }
    };

    let domains_root = options.root.join(&config.paths.domains);
    let domain_root = domains_root.join(&options.name);

    if domain_root.exists() {
        eprintln!("domain already exists: {}", options.name);
        return 2;
    }

    if let Err(err) = scaffold_domain(&domain_root, &options.name, &config.domain.public_api) {
        eprintln!("failed to create domain: {err}");
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

fn is_kebab_case(value: &str) -> bool {
    if value.is_empty() || value.starts_with('-') || value.ends_with('-') {
        return false;
    }

    value
        .chars()
        .all(|char| char.is_ascii_lowercase() || char.is_ascii_digit() || char == '-')
        && !value.contains("--")
}

fn display_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn print_text(violations: &[Violation]) {
    if violations.is_empty() {
        println!("check-boundaries: OK (no violations)");
        return;
    }

    for violation in violations {
        println!("[BOUNDARY_VIOLATION] {}", violation.rule);
        println!("file: {}", violation.file);
        println!("import: {}", violation.import_path);
        println!("line: {}", violation.line);
        println!("message: {}", violation.message);
        println!("suggestion: {}", violation.suggestion);
        println!();
    }

    println!(
        "check-boundaries: FAILED ({} violation(s))",
        violations.len()
    );
}

fn print_json(violations: &[Violation]) {
    let status = if violations.is_empty() {
        "passed"
    } else {
        "failed"
    };
    println!("{{");
    println!("  \"status\": \"{}\",", status);
    println!("  \"violations\": [");

    for (index, v) in violations.iter().enumerate() {
        let suffix = if index + 1 == violations.len() {
            ""
        } else {
            ","
        };
        println!("    {{");
        println!("      \"rule\": \"{}\",", v.rule);
        println!("      \"file\": \"{}\",", escape_json(&v.file));
        println!("      \"line\": {},", v.line);
        println!("      \"import\": \"{}\",", escape_json(&v.import_path));
        println!("      \"message\": \"{}\",", escape_json(&v.message));
        println!("      \"suggestion\": \"{}\"", escape_json(&v.suggestion));
        println!("    }}{}", suffix);
    }

    println!("  ]");
    println!("}}");
}

fn escape_json(input: &str) -> String {
    input
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

fn print_help() {
    println!("Boundra CLI");
    println!();
    println!("Usage:");
    println!("  boundra check-boundaries [--root <path>] [--format text|json]");
    println!("  boundra create-domain <name> [--root <path>]");
}
