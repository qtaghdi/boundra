use crate::commands;
use crate::output::{print_error, print_error_json, CliDiagnostic};
use crate::parsing::{parse_command, Command};

pub fn run(args: &[String]) -> i32 {
    let command = match parse_command(args) {
        Ok(command) => command,
        Err(err) => {
            let command = args.first().map(String::as_str).unwrap_or("boundra");
            let diagnostic = CliDiagnostic::new(
                "CLI-001",
                err,
                "run 'boundra --help' to see valid commands and options",
            )
            .with_context("command", command);
            if requests_json(args) {
                print_error_json(command, &diagnostic);
            } else {
                print_error(&diagnostic);
            }
            return 2;
        }
    };

    match command {
        Command::AddDependency(options) => commands::add_dependency::run(&options),
        Command::CheckBoundaries(options) => commands::check_boundaries::run(&options),
        Command::CreateDomain(options) => commands::create_domain::run(&options),
        Command::GraphDomains(options) => commands::graph_domains::run(&options),
        Command::Generate(options) => commands::generate::run(&options),
        Command::Help => {
            print_help();
            0
        }
    }
}

fn requests_json(args: &[String]) -> bool {
    args.windows(2)
        .any(|pair| pair[0] == "--format" && pair[1] == "json")
        || args.iter().any(|arg| arg == "--format=json")
}

fn print_help() {
    println!("Boundra CLI");
    println!();
    println!("Usage:");
    println!("  boundra add-dependency <domain>/<dependency> [--root <path>]");
    println!("  boundra check-boundaries [--root <path>] [--format text|json]");
    println!("  boundra create-domain <name> [--root <path>]");
    println!(
        "  boundra graph-domains [--root <path>] [--format mermaid|dot|json] [--output <path>]"
    );
    println!("  boundra generate route|query|mutation <domain>/<name> [--root <path>]");
}
