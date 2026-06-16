use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use boundra_core::{load_project_model, DomainManifest};
use serde::Serialize;

use crate::util::display_path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GraphDomainsOptions {
    pub(crate) format: GraphFormat,
    pub(crate) output: Option<PathBuf>,
    pub(crate) root: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GraphFormat {
    Mermaid,
    Dot,
    Json,
}

#[derive(Debug, Serialize)]
struct GraphDomainsOutput<'a> {
    domains: Vec<GraphDomainOutput<'a>>,
    edges: Vec<GraphEdgeOutput<'a>>,
    meta: GraphOutputMeta<'a>,
}

#[derive(Debug, Serialize)]
struct GraphDomainOutput<'a> {
    name: &'a str,
    depends_on: &'a [String],
}

#[derive(Debug, Serialize)]
struct GraphEdgeOutput<'a> {
    from: &'a str,
    to: &'a str,
}

#[derive(Debug, Serialize)]
struct GraphOutputMeta<'a> {
    command: &'a str,
    format: &'a str,
    domain_count: usize,
    edge_count: usize,
}

pub(crate) fn run(options: &GraphDomainsOptions) -> i32 {
    let project = match load_project_model(&options.root) {
        Ok(project) => project,
        Err(err) => {
            eprintln!("failed to load project: {err}");
            return 2;
        }
    };

    // domain.json의 dependsOn만 사용해 도메인 의존 그래프를 만든다.
    // 코드 import를 다시 스캔하지 않기 때문에 "선언된 의존성"을 보여주는 명령이다.
    let output = match options.format {
        GraphFormat::Mermaid => render_mermaid_graph(&project.domains),
        GraphFormat::Dot => render_dot_graph(&project.domains),
        GraphFormat::Json => render_json_graph(&project.domains),
    };

    if let Some(path) = &options.output {
        if let Some(parent) = path.parent() {
            if let Err(err) = fs::create_dir_all(parent) {
                eprintln!("failed to create graph output directory: {err}");
                return 3;
            }
        }
        if let Err(err) = fs::write(path, output) {
            eprintln!("failed to write graph output: {err}");
            return 3;
        }
        println!("graph-domains: OK ({})", display_path(path));
        return 0;
    }

    println!("{output}");
    0
}

fn render_mermaid_graph(domains: &BTreeMap<String, DomainManifest>) -> String {
    let mut lines = vec!["graph TD".to_string()];
    for (name, manifest) in domains {
        lines.push(format!("  {name}"));
        for dependency in &manifest.depends_on {
            lines.push(format!("  {name} --> {dependency}"));
        }
    }

    lines.join("\n")
}

fn render_dot_graph(domains: &BTreeMap<String, DomainManifest>) -> String {
    let mut lines = vec!["digraph domains {".to_string()];
    for (name, manifest) in domains {
        lines.push(format!("  \"{name}\";"));
        for dependency in &manifest.depends_on {
            lines.push(format!("  \"{name}\" -> \"{dependency}\";"));
        }
    }
    lines.push("}".to_string());

    lines.join("\n")
}

fn render_json_graph(domains: &BTreeMap<String, DomainManifest>) -> String {
    // JSON 출력은 CI/AI가 파싱하기 쉬우도록 nodes(domains), edges, meta를 분리한다.
    let edges = domains
        .values()
        .flat_map(|manifest| {
            manifest
                .depends_on
                .iter()
                .map(|dependency| GraphEdgeOutput {
                    from: manifest.name.as_str(),
                    to: dependency.as_str(),
                })
        })
        .collect::<Vec<_>>();
    let output = GraphDomainsOutput {
        domains: domains
            .values()
            .map(|manifest| GraphDomainOutput {
                name: &manifest.name,
                depends_on: &manifest.depends_on,
            })
            .collect(),
        meta: GraphOutputMeta {
            command: "graph-domains",
            format: graph_format_name(GraphFormat::Json),
            domain_count: domains.len(),
            edge_count: edges.len(),
        },
        edges,
    };

    serde_json::to_string_pretty(&output).expect("failed to serialize graph JSON")
}

pub(crate) fn graph_format_name(format: GraphFormat) -> &'static str {
    match format {
        GraphFormat::Mermaid => "mermaid",
        GraphFormat::Dot => "dot",
        GraphFormat::Json => "json",
    }
}
