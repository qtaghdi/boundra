use std::path::PathBuf;

use crate::commands::add_dependency::AddDependencyOptions;
use crate::commands::check_boundaries::CheckBoundariesOptions;
use crate::commands::create_domain::CreateDomainOptions;
use crate::commands::generate::{GenerateKind, GenerateOptions};
use crate::commands::graph_domains::{GraphDomainsOptions, GraphFormat};
use crate::output::OutputFormat;
use crate::util::is_kebab_case;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Command {
    AddDependency(AddDependencyOptions),
    CheckBoundaries(CheckBoundariesOptions),
    CreateDomain(CreateDomainOptions),
    GraphDomains(GraphDomainsOptions),
    Generate(GenerateOptions),
    Help,
    Version,
}

pub(crate) fn parse_command(args: &[String]) -> Result<Command, String> {
    let Some(command) = args.first() else {
        return Err("missing command".to_string());
    };

    match command.as_str() {
        "add-dependency" => {
            let options = parse_add_dependency_options(&args[1..])?;
            Ok(Command::AddDependency(options))
        }
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
        "--version" | "-V" => Ok(Command::Version),
        _ => Err(format!("unknown command: {command}")),
    }
}

fn parse_add_dependency_options(args: &[String]) -> Result<AddDependencyOptions, String> {
    let Some(resource) = args.first() else {
        return Err("missing dependency resource".to_string());
    };
    let (domain, dependency) = parse_dependency_resource(resource)?;
    let mut options = AddDependencyOptions {
        domain,
        dependency,
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

fn parse_dependency_resource(value: &str) -> Result<(String, String), String> {
    let mut parts = value.split('/');
    let Some(domain) = parts.next() else {
        return Err("dependency resource must be <domain>/<dependency>".to_string());
    };
    let Some(dependency) = parts.next() else {
        return Err("dependency resource must be <domain>/<dependency>".to_string());
    };

    if parts.next().is_some() || !is_kebab_case(domain) || !is_kebab_case(dependency) {
        return Err("dependency resource must be <kebab-domain>/<kebab-dependency>".to_string());
    }

    Ok((domain.to_string(), dependency.to_string()))
}
