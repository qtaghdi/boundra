use std::collections::BTreeSet;
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
        analyzed_domain_count: count_analyzed_domains(
            &scan_report.scanned_files,
            &project.config.paths.domains,
            project.domains.keys().map(String::as_str),
        ),
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

/// Count manifest-backed domains that contributed at least one scanned file.
fn count_analyzed_domains<'a>(
    scanned_files: &[String],
    domains_path: &str,
    known_domains: impl IntoIterator<Item = &'a str>,
) -> usize {
    let known_domains = known_domains.into_iter().collect::<BTreeSet<_>>();
    let normalized_root = domains_path
        .replace('\\', "/")
        .trim_matches('/')
        .trim_start_matches("./")
        .to_string();
    let normalized_root = if normalized_root == "." {
        String::new()
    } else {
        normalized_root
    };
    let prefix = (!normalized_root.is_empty()).then(|| format!("{normalized_root}/"));

    scanned_files
        .iter()
        .filter_map(|file| {
            let relative = match &prefix {
                Some(prefix) => file.strip_prefix(prefix)?,
                None => file.as_str(),
            };
            let domain = relative.split('/').next()?;
            known_domains.contains(domain).then_some(domain)
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
