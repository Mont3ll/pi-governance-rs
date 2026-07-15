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
    for args in [vec!["--store", &root, "graph", "--json"], vec!["--store", &root, "quality", "memory", "--json"], vec!["--store", &root, "quality", "relationship", "--json"]] {
        let value: serde_json::Value = serde_json::from_str(&success(&args)).unwrap();
        assert_eq!(value["mutation_performed"], false);
        assert_eq!(value["schema_version"], 1);
    }
}
