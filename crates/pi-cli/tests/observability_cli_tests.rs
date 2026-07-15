use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
fn bin() -> String { env!("CARGO_BIN_EXE_pi").to_string() }
fn store() -> String { format!("/tmp/pi-cli-observability-{}-{}", std::process::id(), SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()) }
fn success(args: &[&str]) -> String { let out = Command::new(bin()).args(args).output().unwrap(); assert!(out.status.success(), "stderr={}", String::from_utf8_lossy(&out.stderr)); String::from_utf8(out.stdout).unwrap() }
#[test]
fn graph_rejects_zero_limits() {
    let root = store();
    let out = Command::new(bin()).args(["--store", &root, "graph", "--max-nodes", "0", "--json"]).output().unwrap();
    assert!(!out.status.success());
}

#[test]
fn graph_and_quality_commands_return_versioned_read_only_json() {
    let root = store();
    success(&["--store", &root, "demo", "--reset"]);
    for args in [vec!["--store", &root, "graph", "--json"], vec!["--store", &root, "quality", "memory", "--json"], vec!["--store", &root, "quality", "relationship", "--json"], vec!["--store", &root, "quality", "recall", "--json"], vec!["--store", &root, "quality", "store", "--json"]] {
        let value: serde_json::Value = serde_json::from_str(&success(&args)).unwrap();
        assert_eq!(value["mutation_performed"], false);
        assert_eq!(value["schema_version"], 1);
    }
}

#[test]
fn simulate_patch_is_read_only() {
    let root = store();
    success(&["--store", &root, "demo", "--reset"]);
    let patches: serde_json::Value = serde_json::from_str(&success(&["--store", &root, "list-patches", "--json"])).unwrap();
    let patch_id = patches.as_array().unwrap().iter().find(|patch| patch["latest_status"] == "proposed").unwrap()["patch_id"].as_str().unwrap();
    let value: serde_json::Value = serde_json::from_str(&success(&["--store", &root, "simulate-patch", patch_id, "--json"])).unwrap();
    assert_eq!(value["mutation_performed"], false);
    assert_eq!(value["predicted_patch_status"], "applied");
}

#[test]
fn enabled_telemetry_records_only_query_hash_and_respects_retention() {
    let root = store();
    success(&["--store", &root, "demo", "--reset"]);
    success(&["--store", &root, "config", "set-recall-telemetry", "true", "--max-events", "1"]);
    success(&["--store", &root, "retrieve", "private raw query alpha", "--format", "json"]);
    success(&["--store", &root, "recall-xray", "private raw query beta", "--json"]);
    let telemetry = std::fs::read_to_string(format!("{root}/recall-events.jsonl")).unwrap();
    assert_eq!(telemetry.lines().count(), 1);
    assert!(!telemetry.contains("private raw query"));
    assert!(telemetry.contains("query_hash"));
}
