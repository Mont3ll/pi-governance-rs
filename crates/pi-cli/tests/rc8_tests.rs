use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};

fn bin() -> String { env!("CARGO_BIN_EXE_pi").to_string() }
fn tmp_store(name: &str) -> String { let p = format!("/tmp/pi-cli-{name}-{}", std::process::id()); let _=fs::remove_dir_all(&p); p }

fn mcp(store: &str, ns: &str, req: serde_json::Value) -> serde_json::Value {
    let mut child = Command::new(bin()).args(["--store", store, "--namespace", ns, "mcp-stdio"]).stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
    writeln!(child.stdin.as_mut().unwrap(), "{}", req).unwrap();
    drop(child.stdin.take());
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    serde_json::from_slice(&out.stdout).unwrap()
}

fn propose(store: &str, claim: &str) -> String {
    let out = Command::new(bin()).args(["--store", store, "propose", "--class", "workflow", "--claim", claim, "--project", "pi-governance-rs", "--tag", "rc8", "--evidence-uri", "qa:rc8"]).output().unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    v["patch_id"].as_str().unwrap().to_string()
}

#[test]
fn mcp_inspect_record_and_maintenance_scan_work() {
    let store = tmp_store("rc8-mcp");
    assert!(Command::new(bin()).args(["demo", "--store", &store, "--reset"]).status().unwrap().success());
    let list = Command::new(bin()).args(["--store", &store, "list"]).output().unwrap();
    let text = String::from_utf8(list.stdout).unwrap();
    let rec = text.lines().find(|l| l.contains("status=Active")).unwrap().split(']').next().unwrap().trim_start_matches("- [").to_string();

    let tools = mcp(&store, "default", serde_json::json!({"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}));
    let names = tools["result"]["tools"].as_array().unwrap().iter().filter_map(|t| t["name"].as_str()).collect::<Vec<_>>();
    assert!(names.contains(&"pi.inspect_record"));
    assert!(names.contains(&"pi.maintenance_scan"));

    let inspected = mcp(&store, "default", serde_json::json!({"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"pi.inspect_record","arguments":{"record_id":rec}}}));
    let sc = &inspected["result"]["structuredContent"];
    assert!(sc.is_object());
    assert!(sc["record"].is_object());
    assert!(sc["related_patches"].is_array());
    assert!(sc["audit"].is_object());

    let scan = mcp(&store, "default", serde_json::json!({"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"pi.maintenance_scan","arguments":{}}}));
    assert!(scan["result"]["structuredContent"]["summary"].is_object());
    assert!(scan["result"]["structuredContent"]["findings"].is_array());

    let missing = mcp(&store, "default", serde_json::json!({"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"pi.inspect_record","arguments":{"record_id":"rec_missing"}}}));
    assert_eq!(missing["result"]["isError"], true);
}

#[test]
fn review_actions_reject_defer_apply_and_scan_are_json() {
    let store = tmp_store("rc8-review");
    assert!(Command::new(bin()).args(["init", "--store", &store]).status().unwrap().success());

    let reject_id = propose(&store, "rc8 reject action memory.");
    let out = Command::new(bin()).args(["--store", &store, "review", "--reject", &reject_id, "--reason", "QA reject", "--json"]).output().unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["status"], "rejected");
    assert!(!Command::new(bin()).args(["--store", &store, "apply", &reject_id]).status().unwrap().success());

    let defer_id = propose(&store, "rc8 defer action memory.");
    let out = Command::new(bin()).args(["--store", &store, "review", "--defer", &defer_id, "--reason", "QA defer", "--json"]).output().unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["status"], "deferred");

    let apply_id = propose(&store, "rc8 apply action memory.");
    assert!(Command::new(bin()).args(["--store", &store, "review", "--apply", &apply_id]).status().unwrap().success());
    let list = Command::new(bin()).args(["--store", &store, "list"]).output().unwrap();
    assert!(String::from_utf8(list.stdout).unwrap().contains("rc8 apply action memory"));

    let scan = Command::new(bin()).args(["--store", &store, "maintenance", "scan", "--json"]).output().unwrap();
    assert!(scan.status.success());
    let scan_json: serde_json::Value = serde_json::from_slice(&scan.stdout).unwrap();
    assert!(scan_json["summary"].is_object());
}

#[test]
fn retriever_modes_and_redacted_export() {
    let store = tmp_store("rc8-retrieval");
    assert!(Command::new(bin()).args(["demo", "--store", &store, "--reset"]).status().unwrap().success());
    for mode in ["deterministic", "lexical", "hybrid"] {
        let out = Command::new(bin()).args(["--store", &store, "retrieve", "release workflow", "--retriever", mode, "--explain"]).output().unwrap();
        assert!(out.status.success(), "{mode}: {}", String::from_utf8_lossy(&out.stderr));
        let text = String::from_utf8(out.stdout).unwrap();
        assert!(text.contains(&format!("Retriever: `{mode}`")), "{text}");
    }
    let empty = Command::new(bin()).args(["--store", &store, "retrieve", "xyzabc unusual no match", "--retriever", "hybrid", "--explain"]).output().unwrap();
    assert!(empty.status.success());
    assert!(String::from_utf8(empty.stdout).unwrap().contains("empty reason"));

    let mcp_ret = mcp(&store, "default", serde_json::json!({"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"pi.retrieve_context","arguments":{"query":"release workflow","retriever":"hybrid","explain":true}}}));
    assert_eq!(mcp_ret["result"]["structuredContent"]["retriever"], "hybrid");

    let export = Command::new(bin()).args(["--store", &store, "export", "--redacted"]).output().unwrap();
    assert!(export.status.success());
    let v: serde_json::Value = serde_json::from_slice(&export.stdout).unwrap();
    assert_eq!(v["redaction"]["enabled"], true);
}
