use std::fs;
use std::path::PathBuf;

use boundra_core::load_project_model;
use serde_json::Value;

use crate::output::{print_error, CliDiagnostic};
use crate::util::display_path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AddDependencyOptions {
    pub(crate) domain: String,
    pub(crate) dependency: String,
    pub(crate) root: PathBuf,
}

pub(crate) fn run(options: &AddDependencyOptions) -> i32 {
    let project = match load_project_model(&options.root) {
        Ok(project) => project,
        Err(err) => {
            print_error(
                &CliDiagnostic::new(
                    "PROJECT-001",
                    format!("failed to load project: {err}"),
                    "fix the reported config or domain manifest and retry",
                )
                .with_context("root", options.root.display().to_string()),
            );
            return 2;
        }
    };

    if options.domain == options.dependency {
        print_error(
            &CliDiagnostic::new(
                "DEPENDENCY-001",
                format!("domain '{}' cannot depend on itself", options.domain),
                "choose a different target domain",
            )
            .with_context("domain", &options.domain),
        );
        return 2;
    }
    if !project.domains.contains_key(&options.domain) {
        print_unknown_domain(&options.domain, &project.domains, "domain");
        return 2;
    }
    if !project.domains.contains_key(&options.dependency) {
        print_unknown_domain(&options.dependency, &project.domains, "dependency");
        return 2;
    }

    let manifest_path = options
        .root
        .join(&project.config.paths.domains)
        .join(&options.domain)
        .join(&project.config.domain.manifest_file);
    let changed = match append_dependency(&manifest_path, &options.dependency) {
        Ok(changed) => changed,
        Err(err) => {
            print_error(
                &CliDiagnostic::new(
                    "DEPENDENCY-002",
                    format!("failed to update domain dependency: {err}"),
                    "fix the domain manifest and run add-dependency again",
                )
                .with_context("domain", &options.domain)
                .with_context("dependency", &options.dependency),
            );
            return 3;
        }
    };

    let state = if changed { "added" } else { "already declared" };
    println!(
        "add-dependency: OK ({} -> {}, {state})",
        options.domain, options.dependency
    );
    0
}

fn print_unknown_domain(
    name: &str,
    domains: &std::collections::BTreeMap<String, boundra_core::DomainManifest>,
    role: &str,
) {
    let available = domains.keys().cloned().collect::<Vec<_>>().join(", ");
    print_error(
        &CliDiagnostic::new(
            "DOMAIN-004",
            format!("unknown {role} domain '{name}'"),
            format!("run 'boundra create-domain {name}' or choose an existing domain"),
        )
        .with_context("available", available)
        .with_context(role, name),
    );
}

fn append_dependency(manifest_path: &std::path::Path, dependency: &str) -> std::io::Result<bool> {
    let content = fs::read_to_string(manifest_path)?;
    let mut manifest: Value = serde_json::from_str(&content).map_err(|err| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("invalid JSON in {}: {err}", display_path(manifest_path)),
        )
    })?;
    let manifest_object = manifest.as_object_mut().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "domain manifest must be a JSON object",
        )
    })?;
    let dependencies = manifest_object
        .entry("dependsOn")
        .or_insert_with(|| Value::Array(Vec::new()));
    let dependencies = dependencies.as_array_mut().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "domain manifest dependsOn must be an array",
        )
    })?;

    if dependencies
        .iter()
        .any(|value| value.as_str() == Some(dependency))
    {
        return Ok(false);
    }

    dependencies.push(Value::String(dependency.to_string()));
    let output = serde_json::to_string_pretty(&manifest).expect("failed to serialize manifest");
    fs::write(manifest_path, format!("{output}\n"))?;
    Ok(true)
}
