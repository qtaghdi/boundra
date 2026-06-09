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

fn create_fixture(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("boundra-{name}-{unique}"));

    fs::create_dir_all(root.join("domains/order/client")).expect("failed to create order client");
    fs::create_dir_all(root.join("domains/order/server")).expect("failed to create order server");
    fs::create_dir_all(root.join("domains/auth/shared")).expect("failed to create auth shared");

    root
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
