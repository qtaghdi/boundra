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

#[test]
fn help_lists_the_complete_v1_command_surface() {
    let root = create_temp_dir("help-output");
    let output = run_boundra(&root, &["--help"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert_eq!(output.status.code(), Some(0));
    for command in [
        "add-dependency",
        "check-boundaries",
        "create-domain",
        "generate route|query|mutation",
        "graph-domains",
        "init",
    ] {
        assert!(stdout.contains(command), "help should list {command}");
    }
}

#[test]
fn init_creates_a_valid_non_destructive_workspace() {
    let root = create_temp_dir("init-workspace");
    let root_arg = root.to_string_lossy().to_string();

    let output = run_boundra(&root, &["init", "--root", &root_arg, "--name", "task-app"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert_eq!(output.status.code(), Some(0));
    assert!(stdout.contains("init: OK (task-app)"));
    assert!(root.join("boundra.config.json").is_file());
    assert!(root.join("apps").is_dir());
    assert!(root.join("domains").is_dir());

    let check = run_boundra(
        &root,
        &["check-boundaries", "--root", &root_arg, "--format", "json"],
    );
    assert_eq!(check.status.code(), Some(0));

    let repeated = run_boundra(&root, &["init", "--root", &root_arg]);
    let stderr = String::from_utf8_lossy(&repeated.stderr);
    assert_eq!(repeated.status.code(), Some(2));
    assert!(stderr.contains("[ERROR] PROJECT-003"));
}

#[test]
fn version_matches_the_cli_package_version() {
    let root = create_temp_dir("version-output");
    let output = run_boundra(&root, &["--version"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        stdout.trim(),
        format!("boundra {}", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn missing_command_prints_actionable_diagnostic() {
    let root = create_temp_dir("missing-command");
    let output = run_boundra(&root, &[]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert_eq!(output.status.code(), Some(2));
    assert!(stderr.contains("[ERROR] CLI-001"));
    assert!(stderr.contains("boundra --help"));
}

#[test]
fn usage_errors_are_json_when_json_is_requested() {
    let root = create_temp_dir("json-usage-error");
    let output = run_boundra(
        &root,
        &["check-boundaries", "--format", "json", "--unknown"],
    );
    let json = parse_json_stdout(&output);

    assert_eq!(output.status.code(), Some(2));
    assert_eq!(json["status"], "error");
    assert_eq!(json["errors"][0]["code"], "CLI-001");
    assert_eq!(json["meta"]["command"], "check-boundaries");
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
    assert!(stderr.contains("[ERROR] DOMAIN-001"));
    assert!(stderr.contains("use a kebab-case name"));
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
fn check_boundaries_rejects_app_to_domain_internal_import() {
    let root = create_fixture("app-domain-internal");
    write_domain_manifest(&root, "order", "order", &[]);
    fs::create_dir_all(root.join("apps/web/src")).expect("failed to create app source");
    fs::create_dir_all(root.join("domains/order/server/internal"))
        .expect("failed to create order internal server path");
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
        root.join("apps/web/src/checkout.ts"),
        "import { checkout } from '@domains/order/server/internal/checkout';\n",
    )
    .expect("failed to write app fixture");
    fs::write(
        root.join("domains/order/server/internal/checkout.ts"),
        "export const checkout = true;\n",
    )
    .expect("failed to write domain fixture");

    let output = run_boundra(&root, &["check-boundaries", "--format", "json"]);
    let json = parse_json_stdout(&output);

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(json["violations"][0]["rule"], "BR-005");
    assert_eq!(json["violations"][0]["file"], "apps/web/src/checkout.ts");
}

#[test]
fn check_boundaries_allows_app_to_declared_domain_public_api() {
    let root = create_fixture("app-domain-public");
    write_domain_manifest(&root, "order", "order", &[]);
    fs::create_dir_all(root.join("apps/web/src")).expect("failed to create app source");
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
        root.join("apps/web/src/main.ts"),
        "import '@domains/order/shared/public';\n",
    )
    .expect("failed to write app fixture");

    let output = run_boundra(&root, &["check-boundaries", "--format", "json"]);
    let json = parse_json_stdout(&output);

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(json["status"], "passed");
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

    let contract =
        fs::read_to_string(root.join("domains/billing/shared/contracts/create-invoice.ts"))
            .expect("failed to read generated route contract");
    assert!(contract.contains("from \"boundra\";"));
    assert!(contract.contains("defineRoute, type InferSchema"));
    assert!(contract.contains("import { z } from \"zod\";"));
    assert!(contract.contains("createInvoiceInputSchema = z.object({})"));
    assert!(contract.contains("export const createInvoiceRoute"));
    assert!(contract.contains("input: createInvoiceInputSchema"));

    let route = fs::read_to_string(root.join("domains/billing/server/routes/create-invoice.ts"))
        .expect("failed to read generated route implementation");
    assert!(route.contains("implementRoute"));
    assert!(route.contains("createInvoiceRoute"));

    let public_api = fs::read_to_string(root.join("domains/billing/shared/public.ts"))
        .expect("failed to read shared public API");
    assert!(public_api.contains("export * from \"./contracts/create-invoice\";"));

    let manifest = fs::read_to_string(root.join("domains/billing/domain.json"))
        .expect("failed to read domain manifest");
    let manifest_json: Value = serde_json::from_str(&manifest).expect("manifest should be JSON");
    assert!(manifest_json["publicApi"]["shared"]
        .as_array()
        .expect("shared public API should be an array")
        .iter()
        .any(|value| value.as_str() == Some("./shared/contracts/create-invoice.ts")));
}

#[test]
fn generate_query_and_mutation_scaffold_client_adapters() {
    let root = create_temp_dir("generate-client-adapters");

    let create_output = run_boundra(&root, &["create-domain", "billing"]);
    assert_eq!(create_output.status.code(), Some(0));

    let query_output = run_boundra(&root, &["generate", "query", "billing/list-invoices"]);
    let mutation_output = run_boundra(&root, &["generate", "mutation", "billing/pay-invoice"]);

    assert_eq!(query_output.status.code(), Some(0));
    assert_eq!(mutation_output.status.code(), Some(0));
    assert!(root
        .join("domains/billing/client/queries/list-invoices.ts")
        .exists());
    assert!(root
        .join("domains/billing/client/mutations/pay-invoice.ts")
        .exists());
    assert!(root
        .join("domains/billing/shared/contracts/list-invoices.ts")
        .exists());
    assert!(root
        .join("domains/billing/shared/contracts/pay-invoice.ts")
        .exists());

    let query_contract =
        fs::read_to_string(root.join("domains/billing/shared/contracts/list-invoices.ts"))
            .expect("failed to read generated query contract");
    let mutation_contract =
        fs::read_to_string(root.join("domains/billing/shared/contracts/pay-invoice.ts"))
            .expect("failed to read generated mutation contract");

    assert!(query_contract.contains("defineQuery, type InferSchema"));
    assert!(query_contract.contains("from \"boundra\";"));
    assert!(query_contract.contains("listInvoicesInputSchema = z.object({})"));
    assert!(query_contract.contains("export const listInvoicesQuery"));
    assert!(mutation_contract.contains("defineMutation, type InferSchema"));
    assert!(mutation_contract.contains("from \"boundra\";"));
    assert!(mutation_contract.contains("payInvoiceResultSchema = z.object({})"));
    assert!(mutation_contract.contains("export const payInvoiceMutation"));

    let query_adapter =
        fs::read_to_string(root.join("domains/billing/client/queries/list-invoices.ts"))
            .expect("failed to read generated query adapter");
    let mutation_adapter =
        fs::read_to_string(root.join("domains/billing/client/mutations/pay-invoice.ts"))
            .expect("failed to read generated mutation adapter");
    assert!(query_adapter.contains("from \"boundra\";"));
    assert!(mutation_adapter.contains("from \"boundra\";"));
    assert!(query_adapter.contains("client.query(listInvoicesQuery, input)"));
    assert!(mutation_adapter.contains("client.mutation(payInvoiceMutation, input)"));

    let public_api = fs::read_to_string(root.join("domains/billing/shared/public.ts"))
        .expect("failed to read shared public API");
    assert!(public_api.contains("export * from \"./contracts/list-invoices\";"));
    assert!(public_api.contains("export * from \"./contracts/pay-invoice\";"));

    let manifest = fs::read_to_string(root.join("domains/billing/domain.json"))
        .expect("failed to read domain manifest");
    let manifest_json: Value = serde_json::from_str(&manifest).expect("manifest should be JSON");
    let shared = manifest_json["publicApi"]["shared"]
        .as_array()
        .expect("shared public API should be an array");

    assert!(shared
        .iter()
        .any(|value| value.as_str() == Some("./shared/contracts/list-invoices.ts")));
    assert!(shared
        .iter()
        .any(|value| value.as_str() == Some("./shared/contracts/pay-invoice.ts")));
}

#[test]
fn generate_rejects_unknown_domain() {
    let root = create_temp_dir("generate-unknown-domain");

    fs::create_dir_all(root.join("domains")).expect("failed to create domains root");

    let output = run_boundra(&root, &["generate", "query", "billing/invoices"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert_eq!(output.status.code(), Some(2));
    assert!(stderr.contains("[ERROR] DOMAIN-004"));
    assert!(stderr.contains("unknown domain 'billing'"));
    assert!(stderr.contains("boundra create-domain billing"));
}

#[test]
fn generate_conflict_does_not_leave_partial_contract() {
    let root = create_temp_dir("generate-conflict");
    assert_eq!(
        run_boundra(&root, &["create-domain", "billing"])
            .status
            .code(),
        Some(0)
    );
    let adapter_path = root.join("domains/billing/client/queries/list-invoices.ts");
    fs::create_dir_all(adapter_path.parent().expect("adapter should have a parent"))
        .expect("failed to create adapter directory");
    fs::write(&adapter_path, "export {};\n").expect("failed to write conflicting adapter");

    let output = run_boundra(&root, &["generate", "query", "billing/list-invoices"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert_eq!(output.status.code(), Some(2));
    assert!(stderr.contains("[ERROR] GEN-001"));
    assert!(stderr.contains("never overwrites"));
    assert!(!root
        .join("domains/billing/shared/contracts/list-invoices.ts")
        .exists());

    let public_api = fs::read_to_string(root.join("domains/billing/shared/public.ts"))
        .expect("failed to read shared public API");
    assert_eq!(public_api, "export {};\n");
}

#[test]
fn add_dependency_updates_manifest_idempotently() {
    let root = create_temp_dir("add-dependency");
    assert_eq!(
        run_boundra(&root, &["create-domain", "billing"])
            .status
            .code(),
        Some(0)
    );
    assert_eq!(
        run_boundra(&root, &["create-domain", "order"])
            .status
            .code(),
        Some(0)
    );

    let first = run_boundra(&root, &["add-dependency", "billing/order"]);
    let second = run_boundra(&root, &["add-dependency", "billing/order"]);
    let second_stdout = String::from_utf8_lossy(&second.stdout);

    assert_eq!(first.status.code(), Some(0));
    assert_eq!(second.status.code(), Some(0));
    assert!(second_stdout.contains("already declared"));

    let manifest = fs::read_to_string(root.join("domains/billing/domain.json"))
        .expect("failed to read billing manifest");
    let manifest_json: Value = serde_json::from_str(&manifest).expect("manifest should be JSON");
    assert_eq!(manifest_json["dependsOn"], serde_json::json!(["order"]));
}

#[test]
fn add_dependency_rejects_self_dependency() {
    let root = create_temp_dir("self-dependency");
    assert_eq!(
        run_boundra(&root, &["create-domain", "billing"])
            .status
            .code(),
        Some(0)
    );

    let output = run_boundra(&root, &["add-dependency", "billing/billing"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert_eq!(output.status.code(), Some(2));
    assert!(stderr.contains("[ERROR] DEPENDENCY-001"));
    assert!(stderr.contains("cannot depend on itself"));
}

#[test]
fn check_boundaries_keeps_project_errors_machine_readable() {
    let root = create_temp_dir("json-project-error");
    fs::create_dir_all(root.join("domains")).expect("failed to create domains root");
    fs::write(root.join("boundra.config.json"), "{ invalid json")
        .expect("failed to write invalid config");

    let output = run_boundra(&root, &["check-boundaries", "--format", "json"]);
    let json = parse_json_stdout(&output);

    assert_eq!(output.status.code(), Some(2));
    assert_eq!(json["status"], "error");
    assert_eq!(json["errors"][0]["code"], "PROJECT-001");
    assert!(json["errors"][0]["suggestion"]
        .as_str()
        .is_some_and(|value| value.contains("fix the reported config")));
    assert_eq!(json["meta"]["command"], "check-boundaries");
}
