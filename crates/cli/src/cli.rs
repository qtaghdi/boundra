use crate::commands;
use crate::parsing::{parse_command, Command};

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
