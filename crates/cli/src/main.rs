use std::env;
use std::path::Path;

use boundra_core::Violation;
use boundra_parser::collect_imports;
use boundra_rules::check_boundaries;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_help();
        std::process::exit(2);
    }

    match args[1].as_str() {
        "check-boundaries" => run_check_boundaries(&args[2..]),
        "help" | "--help" | "-h" => {
            print_help();
            std::process::exit(0);
        }
        _ => {
            eprintln!("unknown command: {}", args[1]);
            print_help();
            std::process::exit(2);
        }
    }
}

fn run_check_boundaries(args: &[String]) {
    let json = args.iter().any(|arg| arg == "--format=json");

    let imports = match collect_imports(Path::new(".")) {
        Ok(v) => v,
        Err(err) => {
            eprintln!("failed to scan project: {err}");
            std::process::exit(3);
        }
    };

    let violations = check_boundaries(&imports);

    if json {
        print_json(&violations);
    } else {
        print_text(&violations);
    }

    if violations.is_empty() {
        std::process::exit(0);
    }

    std::process::exit(1);
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

    println!("check-boundaries: FAILED ({} violation(s))", violations.len());
}

fn print_json(violations: &[Violation]) {
    let status = if violations.is_empty() { "passed" } else { "failed" };
    println!("{{");
    println!("  \"status\": \"{}\",", status);
    println!("  \"violations\": [");

    for (index, v) in violations.iter().enumerate() {
        let suffix = if index + 1 == violations.len() { "" } else { "," };
        println!("    {{");
        println!("      \"rule\": \"{}\",", v.rule);
        println!("      \"file\": \"{}\",", escape_json(&v.file));
        println!("      \"line\": {},", v.line);
        println!("      \"import\": \"{}\",", escape_json(&v.import_path));
        println!("      \"message\": \"{}\",", escape_json(&v.message));
        println!(
            "      \"suggestion\": \"{}\"",
            escape_json(&v.suggestion)
        );
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
    println!("  boundra check-boundaries [--format=json]");
}
