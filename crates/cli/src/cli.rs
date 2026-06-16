use std::fs;
use std::path::{Path, PathBuf};

use boundra_core::{load_config, load_project_model, PublicApi, Violation};
use boundra_parser::{collect_imports_with_options, ScanOptions};
use boundra_rules::{check_boundaries_with_context, BoundaryContext};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Command {
    CheckBoundaries(CheckBoundariesOptions),
    CreateDomain(CreateDomainOptions),
    GraphDomains(GraphDomainsOptions),
    Generate(GenerateOptions),
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct GraphDomainsOptions {
    format: GraphFormat,
    output: Option<PathBuf>,
    root: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GraphFormat {
    Mermaid,
    Dot,
    Json,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GenerateOptions {
    kind: GenerateKind,
    domain: String,
    name: String,
    root: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GenerateKind {
    Route,
    Query,
    Mutation,
}

#[derive(Debug, Serialize)]
struct CheckBoundariesOutput<'a> {
    status: &'a str,
    violations: Vec<ViolationOutput<'a>>,
    meta: OutputMeta<'a>,
}

#[derive(Debug, Serialize)]
struct ViolationOutput<'a> {
    rule: String,
    file: &'a str,
    line: usize,
    import: &'a str,
    message: &'a str,
    suggestion: &'a str,
}

#[derive(Debug, Serialize)]
struct OutputMeta<'a> {
    command: &'a str,
    violation_count: usize,
}

#[derive(Debug, Serialize)]
struct GraphDomainsOutput<'a> {
    domains: Vec<GraphDomainOutput<'a>>,
    edges: Vec<GraphEdgeOutput<'a>>,
    meta: GraphOutputMeta<'a>,
}

#[derive(Debug, Serialize)]
struct GraphDomainOutput<'a> {
    name: &'a str,
    depends_on: &'a [String],
}

#[derive(Debug, Serialize)]
struct GraphEdgeOutput<'a> {
    from: &'a str,
    to: &'a str,
}

#[derive(Debug, Serialize)]
struct GraphOutputMeta<'a> {
    command: &'a str,
    format: &'a str,
    domain_count: usize,
    edge_count: usize,
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
        Command::GraphDomains(options) => run_graph_domains(&options),
        Command::Generate(options) => run_generate(&options),
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
        "graph-domains" => {
            let options = parse_graph_domains_options(&args[1..])?;
            Ok(Command::GraphDomains(options))
        }
        "generate" => {
            let options = parse_generate_options(&args[1..])?;
            Ok(Command::Generate(options))
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

fn parse_graph_domains_options(args: &[String]) -> Result<GraphDomainsOptions, String> {
    let mut options = GraphDomainsOptions {
        format: GraphFormat::Mermaid,
        output: None,
        root: PathBuf::from("."),
    };
    let mut index = 0;

    while index < args.len() {
        let arg = &args[index];

        if arg == "--format" {
            let Some(value) = args.get(index + 1) else {
                return Err("missing value for --format".to_string());
            };
            options.format = parse_graph_format(value)?;
            index += 2;
            continue;
        }

        if let Some(value) = arg.strip_prefix("--format=") {
            options.format = parse_graph_format(value)?;
            index += 1;
            continue;
        }

        if arg == "--output" {
            let Some(value) = args.get(index + 1) else {
                return Err("missing value for --output".to_string());
            };
            options.output = Some(PathBuf::from(value));
            index += 2;
            continue;
        }

        if let Some(value) = arg.strip_prefix("--output=") {
            options.output = Some(PathBuf::from(value));
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

fn parse_generate_options(args: &[String]) -> Result<GenerateOptions, String> {
    let Some(kind) = args.first() else {
        return Err("missing generate kind".to_string());
    };
    let Some(resource) = args.get(1) else {
        return Err("missing generate resource".to_string());
    };

    let kind = parse_generate_kind(kind)?;
    let (domain, name) = parse_generate_resource(resource)?;
    let mut options = GenerateOptions {
        kind,
        domain,
        name,
        root: PathBuf::from("."),
    };
    let mut index = 2;

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

fn parse_graph_format(value: &str) -> Result<GraphFormat, String> {
    match value {
        "mermaid" => Ok(GraphFormat::Mermaid),
        "dot" => Ok(GraphFormat::Dot),
        "json" => Ok(GraphFormat::Json),
        _ => Err(format!("invalid --format value: {value}")),
    }
}

fn parse_generate_kind(value: &str) -> Result<GenerateKind, String> {
    match value {
        "route" => Ok(GenerateKind::Route),
        "query" => Ok(GenerateKind::Query),
        "mutation" => Ok(GenerateKind::Mutation),
        _ => Err(format!("invalid generate kind: {value}")),
    }
}

fn parse_generate_resource(value: &str) -> Result<(String, String), String> {
    let mut parts = value.split('/');
    let Some(domain) = parts.next() else {
        return Err("generate resource must be <domain>/<name>".to_string());
    };
    let Some(name) = parts.next() else {
        return Err("generate resource must be <domain>/<name>".to_string());
    };

    if parts.next().is_some() || !is_kebab_case(domain) || !is_kebab_case(name) {
        return Err("generate resource must be <kebab-domain>/<kebab-name>".to_string());
    }

    Ok((domain.to_string(), name.to_string()))
}

fn graph_format_name(format: GraphFormat) -> &'static str {
    match format {
        GraphFormat::Mermaid => "mermaid",
        GraphFormat::Dot => "dot",
        GraphFormat::Json => "json",
    }
}

fn generate_kind_name(kind: GenerateKind) -> &'static str {
    match kind {
        GenerateKind::Route => "route",
        GenerateKind::Query => "query",
        GenerateKind::Mutation => "mutation",
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
            path_aliases: project.path_aliases,
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

fn run_graph_domains(options: &GraphDomainsOptions) -> i32 {
    let project = match load_project_model(&options.root) {
        Ok(project) => project,
        Err(err) => {
            eprintln!("failed to load project: {err}");
            return 2;
        }
    };

    let output = match options.format {
        GraphFormat::Mermaid => render_mermaid_graph(&project.domains),
        GraphFormat::Dot => render_dot_graph(&project.domains),
        GraphFormat::Json => render_json_graph(&project.domains),
    };

    if let Some(path) = &options.output {
        if let Some(parent) = path.parent() {
            if let Err(err) = fs::create_dir_all(parent) {
                eprintln!("failed to create graph output directory: {err}");
                return 3;
            }
        }
        if let Err(err) = fs::write(path, output) {
            eprintln!("failed to write graph output: {err}");
            return 3;
        }
        println!("graph-domains: OK ({})", display_path(path));
        return 0;
    }

    println!("{output}");
    0
}

fn run_generate(options: &GenerateOptions) -> i32 {
    let project = match load_project_model(&options.root) {
        Ok(project) => project,
        Err(err) => {
            eprintln!("failed to load project: {err}");
            return 2;
        }
    };

    if !project.domains.contains_key(&options.domain) {
        eprintln!("unknown domain: {}", options.domain);
        return 2;
    }

    let domain_root = options
        .root
        .join(&project.config.paths.domains)
        .join(&options.domain);
    let created = match scaffold_generated_artifact(&domain_root, options) {
        Ok(created) => created,
        Err(err) => {
            eprintln!("failed to generate artifact: {err}");
            return 3;
        }
    };

    println!(
        "generate {}: OK ({}/{})",
        generate_kind_name(options.kind),
        options.domain,
        options.name
    );
    for path in created {
        println!("created: {}", display_path(&path));
    }
    0
}

fn render_mermaid_graph(
    domains: &std::collections::BTreeMap<String, boundra_core::DomainManifest>,
) -> String {
    let mut lines = vec!["graph TD".to_string()];
    for (name, manifest) in domains {
        lines.push(format!("  {name}"));
        for dependency in &manifest.depends_on {
            lines.push(format!("  {name} --> {dependency}"));
        }
    }

    lines.join("\n")
}

fn render_dot_graph(
    domains: &std::collections::BTreeMap<String, boundra_core::DomainManifest>,
) -> String {
    let mut lines = vec!["digraph domains {".to_string()];
    for (name, manifest) in domains {
        lines.push(format!("  \"{name}\";"));
        for dependency in &manifest.depends_on {
            lines.push(format!("  \"{name}\" -> \"{dependency}\";"));
        }
    }
    lines.push("}".to_string());

    lines.join("\n")
}

fn render_json_graph(
    domains: &std::collections::BTreeMap<String, boundra_core::DomainManifest>,
) -> String {
    let edges = domains
        .values()
        .flat_map(|manifest| {
            manifest
                .depends_on
                .iter()
                .map(|dependency| GraphEdgeOutput {
                    from: manifest.name.as_str(),
                    to: dependency.as_str(),
                })
        })
        .collect::<Vec<_>>();
    let output = GraphDomainsOutput {
        domains: domains
            .values()
            .map(|manifest| GraphDomainOutput {
                name: &manifest.name,
                depends_on: &manifest.depends_on,
            })
            .collect(),
        meta: GraphOutputMeta {
            command: "graph-domains",
            format: graph_format_name(GraphFormat::Json),
            domain_count: domains.len(),
            edge_count: edges.len(),
        },
        edges,
    };

    serde_json::to_string_pretty(&output).expect("failed to serialize graph JSON")
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

fn scaffold_generated_artifact(
    domain_root: &Path,
    options: &GenerateOptions,
) -> std::io::Result<Vec<PathBuf>> {
    let type_name = pascal_case(&options.name);
    let mut created = Vec::new();

    match options.kind {
        GenerateKind::Route => {
            let contract_path = domain_root
                .join("shared")
                .join("contracts")
                .join(format!("{}.ts", options.name));
            let route_path = domain_root
                .join("server")
                .join("routes")
                .join(format!("{}.ts", options.name));

            write_new_file(
                &contract_path,
                &format!(
                    "export type {type_name}Input = Record<string, never>;\nexport type {type_name}Result = Record<string, never>;\n"
                ),
            )?;
            write_new_file(
                &route_path,
                &format!(
                    "import type {{ {type_name}Input, {type_name}Result }} from '../../shared/contracts/{name}';\n\nexport async function {function_name}(input: {type_name}Input): Promise<{type_name}Result> {{\n  void input;\n  return {{}};\n}}\n",
                    name = options.name,
                    function_name = camel_case(&options.name)
                ),
            )?;
            created.push(contract_path);
            created.push(route_path);
        }
        GenerateKind::Query => {
            let contract_path = domain_root
                .join("shared")
                .join("contracts")
                .join(format!("{}.ts", options.name));
            let query_path = domain_root
                .join("client")
                .join("queries")
                .join(format!("use-{}.ts", options.name));

            write_new_file(
                &contract_path,
                &format!(
                    "export type {type_name}QueryInput = Record<string, never>;\nexport type {type_name}QueryResult = Record<string, never>;\n"
                ),
            )?;
            write_new_file(
                &query_path,
                &format!(
                    "import type {{ {type_name}QueryInput, {type_name}QueryResult }} from '../../shared/contracts/{name}';\n\nexport function use{type_name}Query(input: {type_name}QueryInput): {type_name}QueryResult {{\n  void input;\n  return {{}};\n}}\n",
                    name = options.name
                ),
            )?;
            created.push(contract_path);
            created.push(query_path);
        }
        GenerateKind::Mutation => {
            let contract_path = domain_root
                .join("shared")
                .join("contracts")
                .join(format!("{}.ts", options.name));
            let mutation_path = domain_root
                .join("client")
                .join("mutations")
                .join(format!("use-{}.ts", options.name));

            write_new_file(
                &contract_path,
                &format!(
                    "export type {type_name}MutationInput = Record<string, never>;\nexport type {type_name}MutationResult = Record<string, never>;\n"
                ),
            )?;
            write_new_file(
                &mutation_path,
                &format!(
                    "import type {{ {type_name}MutationInput, {type_name}MutationResult }} from '../../shared/contracts/{name}';\n\nexport function use{type_name}Mutation(input: {type_name}MutationInput): {type_name}MutationResult {{\n  void input;\n  return {{}};\n}}\n",
                    name = options.name
                ),
            )?;
            created.push(contract_path);
            created.push(mutation_path);
        }
    }

    Ok(created)
}

fn write_new_file(path: &Path, content: &str) -> std::io::Result<()> {
    if path.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            format!("file already exists: {}", display_path(path)),
        ));
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content)
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

fn pascal_case(value: &str) -> String {
    value
        .split('-')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            let Some(first) = chars.next() else {
                return String::new();
            };
            format!("{}{}", first.to_ascii_uppercase(), chars.as_str())
        })
        .collect::<Vec<_>>()
        .join("")
}

fn camel_case(value: &str) -> String {
    let pascal = pascal_case(value);
    let mut chars = pascal.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };

    format!("{}{}", first.to_ascii_lowercase(), chars.as_str())
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
    let output = CheckBoundariesOutput {
        status,
        violations: violations
            .iter()
            .map(|violation| ViolationOutput {
                rule: violation.rule.to_string(),
                file: &violation.file,
                line: violation.line,
                import: &violation.import_path,
                message: &violation.message,
                suggestion: &violation.suggestion,
            })
            .collect(),
        meta: OutputMeta {
            command: "check-boundaries",
            violation_count: violations.len(),
        },
    };

    println!(
        "{}",
        serde_json::to_string_pretty(&output).expect("failed to serialize JSON output")
    );
}

fn print_help() {
    println!("Boundra CLI");
    println!();
    println!("Usage:");
    println!("  boundra check-boundaries [--root <path>] [--format text|json]");
    println!("  boundra create-domain <name> [--root <path>]");
    println!(
        "  boundra graph-domains [--root <path>] [--format mermaid|dot|json] [--output <path>]"
    );
    println!("  boundra generate route|query|mutation <domain>/<name> [--root <path>]");
}
