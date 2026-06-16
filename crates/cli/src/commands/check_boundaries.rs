use std::path::PathBuf;

use boundra_core::load_project_model;
use boundra_parser::{collect_imports_with_options, ScanOptions};
use boundra_rules::{check_boundaries_with_context, BoundaryContext};

use crate::output::{print_json, print_text, OutputFormat};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CheckBoundariesOptions {
    pub(crate) format: OutputFormat,
    pub(crate) root: PathBuf,
}

pub(crate) fn run(options: &CheckBoundariesOptions) -> i32 {
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
