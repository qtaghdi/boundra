use std::collections::BTreeMap;

use boundra_core::Violation;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OutputFormat {
    Text,
    Json,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CliDiagnostic {
    pub(crate) code: String,
    pub(crate) message: String,
    pub(crate) context: BTreeMap<String, String>,
    pub(crate) suggestion: String,
}

impl CliDiagnostic {
    pub(crate) fn new(
        code: impl Into<String>,
        message: impl Into<String>,
        suggestion: impl Into<String>,
    ) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            context: BTreeMap::new(),
            suggestion: suggestion.into(),
        }
    }

    pub(crate) fn with_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context.insert(key.into(), value.into());
        self
    }
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
struct ErrorOutput<'a> {
    status: &'static str,
    errors: [&'a CliDiagnostic; 1],
    meta: ErrorMeta<'a>,
}

#[derive(Debug, Serialize)]
struct ErrorMeta<'a> {
    command: &'a str,
}

pub(crate) fn print_error(diagnostic: &CliDiagnostic) {
    eprintln!("[ERROR] {}", diagnostic.code);
    eprintln!("message: {}", diagnostic.message);
    for (key, value) in &diagnostic.context {
        eprintln!("{key}: {value}");
    }
    eprintln!("suggestion: {}", diagnostic.suggestion);
}

pub(crate) fn print_error_json(command: &str, diagnostic: &CliDiagnostic) {
    let output = ErrorOutput {
        status: "error",
        errors: [diagnostic],
        meta: ErrorMeta { command },
    };
    println!(
        "{}",
        serde_json::to_string_pretty(&output).expect("failed to serialize error output")
    );
}

pub(crate) fn print_text(violations: &[Violation]) {
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

pub(crate) fn print_json(violations: &[Violation]) {
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
