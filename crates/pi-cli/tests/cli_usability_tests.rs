use std::process::Command;

fn bin() -> String {
    env!("CARGO_BIN_EXE_pi").to_string()
}

fn tmp_store(name: &str) -> String {
    let path = format!("/tmp/pi-cli-{name}-{}", std::process::id());
    let _ = std::fs::remove_dir_all(&path);
    path
}

#[test]
fn review_handles_empty_inbox_and_json() {
    let store = tmp_store("empty-review");
    assert!(Command::new(bin()).args(["--store", &store, "init"]).status().unwrap().success());

    let out = Command::new(bin()).args(["--store", &store, "review"]).output().unwrap();
    assert!(out.status.success());
    let text = String::from_utf8(out.stdout).unwrap();
    assert!(text.contains("Pending patches: 0"));

    let out = Command::new(bin()).args(["--store", &store, "review", "--json"]).output().unwrap();
    assert!(out.status.success());
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["pending_count"], 0);
}

#[test]
fn review_lists_inspects_and_applies_patch() {
    let store = tmp_store("review-apply");
    assert!(Command::new(bin()).args(["--store", &store, "init"]).status().unwrap().success());
    let out = Command::new(bin()).args([
        "--store", &store, "propose", "--class", "workflow", "--claim", "Review test workflow memory.",
        "--project", "pi-governance-rs", "--evidence-uri", "test:review",
    ]).output().unwrap();
    assert!(out.status.success());
    let proposed: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let patch_id = proposed["patch_id"].as_str().unwrap();

    let out = Command::new(bin()).args(["--store", &store, "review", "--json"]).output().unwrap();
    assert!(out.status.success());
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["pending_count"], 1);

    let out = Command::new(bin()).args(["--store", &store, "review", patch_id, "--json"]).output().unwrap();
    assert!(out.status.success());
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["id"], patch_id);

    let out = Command::new(bin()).args(["--store", &store, "review", patch_id, "--apply"]).output().unwrap();
    assert!(out.status.success());
    assert!(String::from_utf8(out.stdout).unwrap().contains("Applied: true"));
}

#[test]
fn demo_creates_store_and_review_retrieve_work() {
    let store = tmp_store("demo");
    let out = Command::new(bin()).args(["--store", &store, "demo", "--json"]).output().unwrap();
    assert!(out.status.success());
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert!(json["records"].as_u64().unwrap() >= 7);
    assert!(json["pending_patches"].as_u64().unwrap() >= 1);

    assert!(Command::new(bin()).args(["--store", &store, "review"]).status().unwrap().success());
    assert!(Command::new(bin()).args(["--store", &store, "retrieve", "release workflow"]).status().unwrap().success());
    assert!(Command::new(bin()).args(["--store", &store, "doctor"]).status().unwrap().success());
}

#[test]
fn agent_instructions_json_is_valid() {
    let out = Command::new(bin()).args(["agent-instructions", "--json"]).output().unwrap();
    assert!(out.status.success());
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert!(json["instructions"].as_array().unwrap().len() >= 4);
}

#[test]
fn inspect_record_finds_json_and_filters_namespace() {
    let store = tmp_store("inspect-record");
    assert!(Command::new(bin()).args(["--store", &store, "init"]).status().unwrap().success());
    let out = Command::new(bin()).args([
        "--store", &store, "propose", "--class", "requirement", "--claim", "Inspect record test memory.",
        "--project", "pi-governance-rs", "--tag", "inspect", "--evidence-uri", "test:inspect", "--apply",
    ]).output().unwrap();
    assert!(out.status.success());
    let proposed: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let record_id = proposed["record_id"].as_str().unwrap();

    let out = Command::new(bin()).args(["--store", &store, "inspect-record", record_id]).output().unwrap();
    assert!(out.status.success());
    let text = String::from_utf8(out.stdout).unwrap();
    assert!(text.contains("Status:"));
    assert!(text.contains("Class:"));
    assert!(text.contains("Inspect record test memory."));
    assert!(text.contains("test:inspect"));

    let out = Command::new(bin()).args(["--store", &store, "inspect-record", record_id, "--json"]).output().unwrap();
    assert!(out.status.success());
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["record"]["id"], record_id);
    assert_eq!(json["record"]["namespace"], "default");

    let out = Command::new(bin()).args(["--store", &store, "--namespace", "other", "inspect-record", record_id]).output().unwrap();
    assert!(!out.status.success());
}

#[test]
fn inspect_record_missing_json_is_error() {
    let store = tmp_store("inspect-missing");
    assert!(Command::new(bin()).args(["--store", &store, "init"]).status().unwrap().success());
    let out = Command::new(bin()).args(["--store", &store, "inspect-record", "rec_missing", "--json"]).output().unwrap();
    assert!(!out.status.success());
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["error"], "record_not_found");
}
