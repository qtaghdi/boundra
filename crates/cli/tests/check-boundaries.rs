use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::Value;

fn run_boundra(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_boundra"))
        .args(args)
        .current_dir(root)
        .output()
        .expect("failed to run boundra CLI")
}

fn create_temp_dir(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();

    let root = std::env::temp_dir().join(format!("boundra-{name}-{unique}"));
    fs::create_dir_all(&root).expect("failed to create temp dir");
    root
}

fn create_fixture(name: &str) -> PathBuf {
    let root = create_temp_dir(name);

    fs::create_dir_all(root.join("domains/order/client")).expect("failed to create order client");
    fs::create_dir_all(root.join("domains/order/server")).expect("failed to create order server");
    fs::create_dir_all(root.join("domains/auth/shared")).expect("failed to create auth shared");

    root
}

fn parse_json_stdout(output: &Output) -> Value {
    serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON")
}

fn write_domain_manifest(root: &Path, domain: &str, name: &str, depends_on: &[&str]) {
    let depends_on = depends_on
        .iter()
        .map(|dependency| format!("\"{dependency}\""))
        .collect::<Vec<_>>()
        .join(", ");
    let domain_root = root.join(format!("domains/{domain}"));

    fs::create_dir_all(domain_root.join("shared")).expect("failed to create shared layer");
    fs::write(domain_root.join("shared/public.ts"), "export {};\n")
        .expect("failed to write public API");
    fs::write(
        domain_root.join("domain.json"),
        format!(
            r#"{{
  "name": "{name}",
  "publicApi": {{
    "client": [],
    "server": [],
    "shared": ["./shared/public.ts"]
  }},
  "dependsOn": [{depends_on}]
}}
"#
        ),
    )
    .expect("failed to write domain manifest");
}

#[test]
fn check_boundaries_defaults_to_text_output() {
    let root = create_fixture("text-output");
    fs::write(
        root.join("domains/order/client/use-order.ts"),
        "import { checkout } from '../server/checkout';\n",
    )
    .expect("failed to write fixture file");
    fs::write(
        root.join("domains/order/server/checkout.ts"),
        "export {};\n",
    )
    .expect("failed to write fixture file");

    let output = run_boundra(&root, &["check-boundaries"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert_eq!(output.status.code(), Some(1));
    assert!(stdout.contains("[BOUNDARY_VIOLATION] BR-001"));
    assert!(stdout.contains("check-boundaries: FAILED (1 violation(s))"));
}

#[test]
fn check_boundaries_accepts_space_separated_json_format() {
    let root = create_fixture("json-output");
    fs::write(
        root.join("domains/order/server/checkout.ts"),
        "import { LoginRequest } from '../../auth/shared/public';\n",
    )
    .expect("failed to write fixture file");
    fs::write(
        root.join("domains/auth/shared/public.ts"),
        "export type LoginRequest = { email: string };\n",
    )
    .expect("failed to write fixture file");

    let output = run_boundra(&root, &["check-boundaries", "--format", "json"]);
    let json = parse_json_stdout(&output);

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(json["status"], "passed");
    assert_eq!(json["violations"].as_array().map(Vec::len), Some(0));
    assert_eq!(json["meta"]["command"], "check-boundaries");
    assert_eq!(json["meta"]["violation_count"], 0);
}

#[test]
fn check_boundaries_accepts_root_option() {
    let root = create_fixture("root-option");
    let cwd = create_temp_dir("outside-root");
    let root_arg = root.to_string_lossy().to_string();

    fs::write(
        root.join("domains/order/client/use-order.ts"),
        "import { checkout } from '../server/checkout';\n",
    )
    .expect("failed to write fixture file");
    fs::write(
        root.join("domains/order/server/checkout.ts"),
        "export {};\n",
    )
    .expect("failed to write fixture file");

    let output = run_boundra(&cwd, &["check-boundaries", "--root", &root_arg]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert_eq!(output.status.code(), Some(1));
    assert!(stdout.contains("[BOUNDARY_VIOLATION] BR-001"));
    assert!(stdout.contains("file: domains/order/client/use-order.ts"));
}

#[test]
fn check_boundaries_detects_multiline_import_violation() {
    let root = create_fixture("multiline-import");

    fs::write(
        root.join("domains/order/client/use-order.ts"),
        r#"import {
  checkout
} from '../server/checkout';
"#,
    )
    .expect("failed to write fixture file");
    fs::write(
        root.join("domains/order/server/checkout.ts"),
        "export {};\n",
    )
    .expect("failed to write fixture file");

    let output = run_boundra(&root, &["check-boundaries", "--format", "json"]);
    let json = parse_json_stdout(&output);

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(json["violations"][0]["rule"], "BR-001");
    assert_eq!(json["violations"][0]["line"], 1);
}

#[test]
fn check_boundaries_detects_dynamic_import_violation() {
    let root = create_fixture("dynamic-import");

    fs::write(
        root.join("domains/order/client/use-order.ts"),
        r#"export async function loadCheckout() {
  return import('../server/checkout');
}
"#,
    )
    .expect("failed to write fixture file");
    fs::write(
        root.join("domains/order/server/checkout.ts"),
        "export {};\n",
    )
    .expect("failed to write fixture file");

    let output = run_boundra(&root, &["check-boundaries", "--format", "json"]);
    let json = parse_json_stdout(&output);

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(json["violations"][0]["rule"], "BR-001");
    assert_eq!(json["violations"][0]["line"], 2);
}

#[test]
fn create_domain_scaffolds_domain_structure() {
    let root = create_temp_dir("create-domain");

    let output = run_boundra(&root, &["create-domain", "billing"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert_eq!(output.status.code(), Some(0));
    assert!(stdout.contains("create-domain: OK (billing)"));
    assert!(root.join("domains/billing/client/public.ts").exists());
    assert!(root.join("domains/billing/server/public.ts").exists());
    assert!(root.join("domains/billing/shared/public.ts").exists());
    assert!(root.join("domains/billing/mcp").is_dir());
    assert!(root.join("domains/billing/tests").is_dir());
    assert!(root.join("domains/billing/domain.json").exists());
}

#[test]
fn create_domain_rejects_non_kebab_case_name() {
    let root = create_temp_dir("create-domain-invalid-name");

    let output = run_boundra(&root, &["create-domain", "Billing"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert_eq!(output.status.code(), Some(2));
    assert!(stderr.contains("domain names must use kebab-case"));
}

#[test]
fn check_boundaries_uses_manifest_public_api() {
    let root = create_temp_dir("manifest-public-api");

    let create_output = run_boundra(&root, &["create-domain", "billing"]);
    assert_eq!(create_output.status.code(), Some(0));

    fs::create_dir_all(root.join("domains/order/server")).expect("failed to create order server");
    fs::write(
        root.join("domains/order/server/checkout.ts"),
        "import { billing } from '../../billing/server/public';\n",
    )
    .expect("failed to write fixture file");

    let output = run_boundra(&root, &["check-boundaries"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert_eq!(output.status.code(), Some(0));
    assert!(stdout.contains("check-boundaries: OK (no violations)"));
}

#[test]
fn check_boundaries_rejects_manifest_name_mismatch() {
    let root = create_temp_dir("manifest-name-mismatch");
    write_domain_manifest(&root, "billing", "payments", &[]);

    let output = run_boundra(&root, &["check-boundaries"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert_eq!(output.status.code(), Some(2));
    assert!(stderr.contains("must match folder name"));
}

#[test]
fn check_boundaries_rejects_invalid_config_json() {
    let root = create_temp_dir("invalid-config-json");

    fs::create_dir_all(root.join("domains")).expect("failed to create domains root");
    fs::write(root.join("boundra.config.json"), "{ invalid json").expect("failed to write config");

    let output = run_boundra(&root, &["check-boundaries"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert_eq!(output.status.code(), Some(2));
    assert!(stderr.contains("invalid JSON in"));
    assert!(stderr.contains("boundra.config.json"));
}

#[test]
fn check_boundaries_rejects_unknown_domain_dependency() {
    let root = create_temp_dir("unknown-dependency");
    write_domain_manifest(&root, "billing", "billing", &["accounting"]);

    let output = run_boundra(&root, &["check-boundaries"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert_eq!(output.status.code(), Some(2));
    assert!(stderr.contains("depends on unknown domain"));
}

#[test]
fn check_boundaries_rejects_missing_public_api_file() {
    let root = create_temp_dir("missing-public-api");
    let domain_root = root.join("domains/billing");

    fs::create_dir_all(&domain_root).expect("failed to create domain root");
    fs::write(
        domain_root.join("domain.json"),
        r#"{
  "name": "billing",
  "publicApi": {
    "client": [],
    "server": ["./server/public.ts"],
    "shared": []
  },
  "dependsOn": []
}
"#,
    )
    .expect("failed to write domain manifest");

    let output = run_boundra(&root, &["check-boundaries"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert_eq!(output.status.code(), Some(2));
    assert!(stderr.contains("public API path does not exist"));
}

#[test]
fn check_boundaries_applies_config_ignore_paths() {
    let root = create_fixture("ignore-paths");

    fs::write(
        root.join("boundra.config.json"),
        r#"{
  "paths": {
    "domains": "domains"
  },
  "checkBoundaries": {
    "includeExtensions": ["ts", "tsx", "js", "jsx"],
    "ignore": ["domains/order/client/**"]
  }
}
"#,
    )
    .expect("failed to write config");
    fs::write(
        root.join("domains/order/client/use-order.ts"),
        "import { checkout } from '../server/checkout';\n",
    )
    .expect("failed to write ignored fixture file");
    fs::write(
        root.join("domains/order/server/checkout.ts"),
        "export {};\n",
    )
    .expect("failed to write server fixture file");

    let output = run_boundra(&root, &["check-boundaries"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert_eq!(output.status.code(), Some(0));
    assert!(stdout.contains("check-boundaries: OK (no violations)"));
}

#[test]
fn check_boundaries_accepts_equals_json_format() {
    let root = create_fixture("equals-json-output");
    fs::write(
        root.join("domains/order/server/checkout.ts"),
        "import { stock } from '../../auth/server/internal';\n",
    )
    .expect("failed to write fixture file");
    fs::write(
        root.join("domains/auth/shared/public.ts"),
        "export type LoginRequest = { email: string };\n",
    )
    .expect("failed to write fixture file");

    let output = run_boundra(&root, &["check-boundaries", "--format=json"]);
    let json = parse_json_stdout(&output);

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(json["status"], "failed");
    assert_eq!(json["violations"][0]["rule"], "BR-004");
    assert_eq!(json["meta"]["command"], "check-boundaries");
    assert_eq!(json["meta"]["violation_count"], 1);
}

#[test]
fn check_boundaries_rejects_invalid_format() {
    let root = create_fixture("invalid-format");

    let output = run_boundra(&root, &["check-boundaries", "--format", "yaml"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert_eq!(output.status.code(), Some(2));
    assert!(stderr.contains("invalid --format value: yaml"));
}

#[test]
fn check_boundaries_resolves_tsconfig_path_aliases() {
    let root = create_fixture("path-aliases");

    fs::write(
        root.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "paths": {
      "@domains/*": ["domains/*"]
    }
  }
}
"#,
    )
    .expect("failed to write tsconfig");
    fs::write(
        root.join("domains/order/client/use-order.ts"),
        "import { checkout } from '@domains/order/server/checkout';\n",
    )
    .expect("failed to write fixture file");
    fs::write(
        root.join("domains/order/server/checkout.ts"),
        "export {};\n",
    )
    .expect("failed to write fixture file");

    let output = run_boundra(&root, &["check-boundaries", "--format", "json"]);
    let json = parse_json_stdout(&output);

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(json["violations"][0]["rule"], "BR-001");
}

#[test]
fn graph_domains_outputs_json_dependency_graph() {
    let root = create_temp_dir("graph-json");
    write_domain_manifest(&root, "billing", "billing", &[]);
    write_domain_manifest(&root, "order", "order", &["billing"]);

    let output = run_boundra(&root, &["graph-domains", "--format", "json"]);
    let json = parse_json_stdout(&output);

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(json["meta"]["command"], "graph-domains");
    assert_eq!(json["meta"]["domain_count"], 2);
    assert_eq!(json["meta"]["edge_count"], 1);
    assert_eq!(json["edges"][0]["from"], "order");
    assert_eq!(json["edges"][0]["to"], "billing");
}

#[test]
fn graph_domains_writes_output_file() {
    let root = create_temp_dir("graph-output");
    let output_path = root.join("artifacts/domain-graph.md");
    let output_arg = output_path.to_string_lossy().to_string();

    write_domain_manifest(&root, "billing", "billing", &[]);
    write_domain_manifest(&root, "order", "order", &["billing"]);

    let output = run_boundra(
        &root,
        &[
            "graph-domains",
            "--format",
            "mermaid",
            "--output",
            &output_arg,
        ],
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let graph = fs::read_to_string(output_path).expect("failed to read graph output");

    assert_eq!(output.status.code(), Some(0));
    assert!(stdout.contains("graph-domains: OK"));
    assert!(graph.contains("graph TD"));
    assert!(graph.contains("order --> billing"));
}

#[test]
fn generate_route_scaffolds_contract_and_server_route() {
    let root = create_temp_dir("generate-route");

    let create_output = run_boundra(&root, &["create-domain", "billing"]);
    assert_eq!(create_output.status.code(), Some(0));

    let output = run_boundra(&root, &["generate", "route", "billing/create-invoice"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert_eq!(output.status.code(), Some(0));
    assert!(stdout.contains("generate route: OK (billing/create-invoice)"));
    assert!(root
        .join("domains/billing/shared/contracts/create-invoice.ts")
        .exists());
    assert!(root
        .join("domains/billing/server/routes/create-invoice.ts")
        .exists());
}

#[test]
fn generate_query_and_mutation_scaffold_client_hooks() {
    let root = create_temp_dir("generate-client-hooks");

    let create_output = run_boundra(&root, &["create-domain", "billing"]);
    assert_eq!(create_output.status.code(), Some(0));

    let query_output = run_boundra(&root, &["generate", "query", "billing/list-invoices"]);
    let mutation_output = run_boundra(&root, &["generate", "mutation", "billing/pay-invoice"]);

    assert_eq!(query_output.status.code(), Some(0));
    assert_eq!(mutation_output.status.code(), Some(0));
    assert!(root
        .join("domains/billing/client/queries/use-list-invoices.ts")
        .exists());
    assert!(root
        .join("domains/billing/client/mutations/use-pay-invoice.ts")
        .exists());
    assert!(root
        .join("domains/billing/shared/contracts/list-invoices.ts")
        .exists());
    assert!(root
        .join("domains/billing/shared/contracts/pay-invoice.ts")
        .exists());
}

#[test]
fn generate_rejects_unknown_domain() {
    let root = create_temp_dir("generate-unknown-domain");

    fs::create_dir_all(root.join("domains")).expect("failed to create domains root");

    let output = run_boundra(&root, &["generate", "query", "billing/invoices"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert_eq!(output.status.code(), Some(2));
    assert!(stderr.contains("unknown domain: billing"));
}
