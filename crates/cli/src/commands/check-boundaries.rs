use std::path::PathBuf;

use boundra_core::load_project_model;
use boundra_parser::{collect_imports_with_report, ScanOptions};
use boundra_rules::{check_boundaries_with_context, BoundaryContext};

use crate::output::{
    print_error, print_error_json, print_json, print_text, BoundaryScanCoverage, CliDiagnostic,
    OutputFormat,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CheckBoundariesOptions {
    pub(crate) format: OutputFormat,
    pub(crate) root: PathBuf,
}

pub(crate) fn run(options: &CheckBoundariesOptions) -> i32 {
    let project = match load_project_model(&options.root) {
        Ok(project) => project,
        Err(err) => {
            report_error(
                options,
                &CliDiagnostic::new(
                    "PROJECT-001",
                    format!("failed to load project: {err}"),
                    "fix the reported config or domain manifest and run the command again",
                )
                .with_context("root", options.root.display().to_string()),
            );
            return 2;
        }
    };
    let scan_options = ScanOptions {
        include_extensions: project.config.check_boundaries.include_extensions.clone(),
        ignore: project.config.check_boundaries.ignore.clone(),
    };
    let scan_report = match collect_imports_with_report(&options.root, &scan_options) {
        Ok(v) => v,
        Err(err) => {
            report_error(
                options,
                &CliDiagnostic::new(
                    "PROJECT-002",
                    format!("failed to scan project: {err}"),
                    "check file permissions and configured scan paths, then retry",
                )
                .with_context("root", options.root.display().to_string()),
            );
            return 3;
        }
    };

    let coverage = BoundaryScanCoverage {
        scanned_file_count: scan_report.scanned_file_count,
        analyzed_domain_count: project.domains.len(),
    };
    let violations = check_boundaries_with_context(
        &scan_report.imports,
        &BoundaryContext {
            apps_path: project.config.paths.apps.clone(),
            domains_path: project.config.paths.domains.clone(),
            packages_path: project.config.paths.packages.clone(),
            domains: project.domains,
            path_aliases: project.path_aliases,
        },
    );

    match options.format {
        OutputFormat::Text => print_text(&violations, coverage),
        OutputFormat::Json => print_json(&violations, coverage),
    }

    if violations.is_empty() {
        0
    } else {
        1
    }
}

fn report_error(options: &CheckBoundariesOptions, diagnostic: &CliDiagnostic) {
    match options.format {
        OutputFormat::Text => print_error(diagnostic),
        OutputFormat::Json => print_error_json("check-boundaries", diagnostic),
    }
}
