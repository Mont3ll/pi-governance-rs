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
fn store_integrity_previews_by_default_and_applies_with_reviewed_fingerprint() {
    let store = tmp_store("integrity");
    assert!(Command::new(bin()).args(["--store", &store, "demo", "--json"]).status().unwrap().success());
    let records_path = format!("{store}/records.jsonl");
    let original = std::fs::read_to_string(&records_path).unwrap();
    let first = original.lines().next().unwrap();
    let corrupted = format!("{original}{first}\n");
    std::fs::write(&records_path, &corrupted).unwrap();

    let preview = Command::new(bin()).args(["--store", &store, "store-integrity", "--json"]).output().unwrap();
    assert!(preview.status.success());
    assert_eq!(std::fs::read_to_string(&records_path).unwrap(), corrupted);
    let preview_json: serde_json::Value = serde_json::from_slice(&preview.stdout).unwrap();
    assert_eq!(preview_json["dry_run"], true);
    assert_eq!(preview_json["mutation_performed"], false);
    let fingerprint = preview_json["fingerprint"].as_str().unwrap();

    let missing = Command::new(bin()).args(["--store", &store, "store-integrity", "--apply"]).output().unwrap();
    assert!(!missing.status.success());
    assert_eq!(std::fs::read_to_string(&records_path).unwrap(), corrupted);

    let applied = Command::new(bin()).args(["--store", &store, "store-integrity", "--apply", "--fingerprint", fingerprint, "--json"]).output().unwrap();
    assert!(applied.status.success());
    let applied_json: serde_json::Value = serde_json::from_slice(&applied.stdout).unwrap();
    assert_eq!(applied_json["mutation_performed"], true);
    assert!(applied_json["backup"].is_object());
    assert!(applied_json["report_path"].is_string());
    assert_eq!(std::fs::read_to_string(&records_path).unwrap().lines().count(), original.lines().count());
}

#[test]
fn migrate_previews_by_default_and_requires_apply_for_mutation() {
    let store = tmp_store("migrate-preview");
    std::fs::create_dir_all(&store).unwrap();
    let legacy = "{\"id\":\"evt_legacy\",\"severity\":\"info\",\"message\":\"legacy\",\"object_id\":null,\"created_at\":\"2026-01-01T00:00:00Z\"}\n";
    std::fs::write(format!("{store}/events.jsonl"), legacy).unwrap();

    let before_entries = std::fs::read_dir(&store).unwrap().map(|entry| entry.unwrap().file_name()).collect::<Vec<_>>();
    let preview = Command::new(bin()).args(["--store", &store, "migrate", "--json"]).output().unwrap();
    assert!(preview.status.success());
    let preview_json: serde_json::Value = serde_json::from_slice(&preview.stdout).unwrap();
    assert_eq!(preview_json["dry_run"], true);
    assert_eq!(std::fs::read_to_string(format!("{store}/events.jsonl")).unwrap(), legacy);
    let after_entries = std::fs::read_dir(&store).unwrap().map(|entry| entry.unwrap().file_name()).collect::<Vec<_>>();
    assert_eq!(after_entries, before_entries);

    let applied = Command::new(bin()).args(["--store", &store, "migrate", "--apply", "--json"]).output().unwrap();
    assert!(applied.status.success());
    let applied_json: serde_json::Value = serde_json::from_slice(&applied.stdout).unwrap();
    assert_eq!(applied_json["dry_run"], false);
    assert!(applied_json["backup"].is_object());
    assert!(std::fs::read_to_string(format!("{store}/events.jsonl")).unwrap().contains("schema_version"));
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
