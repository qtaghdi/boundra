use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use boundra_core::load_project_model;
use boundra_parser::{collect_imports_with_report, ScanOptions};
use boundra_rules::{check_boundaries_with_config, BoundaryContext};

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
        analyzed_domain_count: count_analyzed_domains(
            &scan_report.scanned_files,
            &project.domain_roots,
        ),
    };
    let violations = check_boundaries_with_config(
        &scan_report.imports,
        &BoundaryContext {
            apps_path: project.config.paths.apps.clone(),
            domains_path: project.config.paths.domains.clone(),
            packages_path: project.config.paths.packages.clone(),
            domains: project.domains,
            domain_roots: project.domain_roots,
            path_aliases: project.path_aliases,
        },
        &project.config.check_boundaries,
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

/// Count manifest-backed domains that contributed at least one scanned file.
fn count_analyzed_domains(
    scanned_files: &[String],
    domain_roots: &BTreeMap<String, String>,
) -> usize {
    scanned_files
        .iter()
        .filter_map(|file| {
            let normalized_file = file.replace('\\', "/");
            domain_roots
                .iter()
                .filter(|(_, root)| {
                    let normalized_root = root.replace('\\', "/").trim_matches('/').to_string();
                    normalized_file == normalized_root
                        || normalized_file.starts_with(&format!("{normalized_root}/"))
                })
                .max_by_key(|(_, root)| root.len())
                .map(|(domain, _)| domain.as_str())
        })
        .collect::<BTreeSet<_>>()
        .len()
}

fn report_error(options: &CheckBoundariesOptions, diagnostic: &CliDiagnostic) {
    match options.format {
        OutputFormat::Text => print_error(diagnostic),
        OutputFormat::Json => print_error_json("check-boundaries", diagnostic),
    }
}
