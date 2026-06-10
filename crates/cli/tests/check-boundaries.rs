use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

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
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert_eq!(output.status.code(), Some(0));
    assert!(stdout.contains("\"status\": \"passed\""));
    assert!(stdout.contains("\"violations\": ["));
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
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert_eq!(output.status.code(), Some(1));
    assert!(stdout.contains("\"status\": \"failed\""));
    assert!(stdout.contains("\"rule\": \"BR-004\""));
}

#[test]
fn check_boundaries_rejects_invalid_format() {
    let root = create_fixture("invalid-format");

    let output = run_boundra(&root, &["check-boundaries", "--format", "yaml"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert_eq!(output.status.code(), Some(2));
    assert!(stderr.contains("invalid --format value: yaml"));
}
