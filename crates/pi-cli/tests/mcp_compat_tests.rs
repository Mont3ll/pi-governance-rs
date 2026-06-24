use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};

fn bin() -> String { env!("CARGO_BIN_EXE_pi").to_string() }
fn tmp_store(name: &str) -> String {
    let path = format!("/tmp/pi-cli-{name}-{}", std::process::id());
    let _ = fs::remove_dir_all(&path);
    path
}

fn mcp_call(store: &str, namespace: &str, request: serde_json::Value) -> serde_json::Value {
    let mut child = Command::new(bin())
        .args(["--store", store, "--namespace", namespace, "mcp-stdio"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    writeln!(child.stdin.as_mut().unwrap(), "{}", request).unwrap();
    drop(child.stdin.take());
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    serde_json::from_slice(&out.stdout).unwrap()
}

#[test]
fn list_tools_exposes_expected_tools() {
    let store = tmp_store("mcp-tools-list");
    let response = mcp_call(&store, "interop-test", serde_json::json!({"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}));
    let tools = response["result"]["tools"].as_array().unwrap();
    let names: Vec<_> = tools.iter().filter_map(|t| t["name"].as_str()).collect();
    assert!(names.contains(&"pi.retrieve_context"));
    assert!(names.contains(&"pi.list_patches"));
    assert!(names.contains(&"pi.list_records"));
}

#[test]
fn list_tools_return_object_shaped_structured_content() {
    let store = tmp_store("mcp-list-shapes");
    assert!(Command::new(bin()).args(["demo", "--store", &store, "--reset"]).status().unwrap().success());

    let patches = mcp_call(&store, "interop-test", serde_json::json!({
        "jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"pi.list_patches","arguments":{"limit":20}}
    }));
    let patch_sc = &patches["result"]["structuredContent"];
    assert!(patch_sc.is_object(), "structuredContent must be object: {patch_sc}");
    assert!(patch_sc["patches"].is_array(), "patches must be array: {patch_sc}");
    assert!(patch_sc["count"].is_number(), "count must be number: {patch_sc}");

    let records = mcp_call(&store, "interop-test", serde_json::json!({
        "jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"pi.list_records","arguments":{"limit":20}}
    }));
    let record_sc = &records["result"]["structuredContent"];
    assert!(record_sc.is_object(), "structuredContent must be object: {record_sc}");
    assert!(record_sc["records"].is_array(), "records must be array: {record_sc}");
    assert!(record_sc["count"].is_number(), "count must be number: {record_sc}");
}

#[test]
fn mcp_default_namespace_is_server_namespace() {
    let store = tmp_store("mcp-namespace-default");
    assert!(Command::new(bin()).args(["demo", "--store", &store, "--reset"]).status().unwrap().success());

    let propose = mcp_call(&store, "interop-test", serde_json::json!({
        "jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"pi.propose_record","arguments":{
            "class":"workflow","project":"pi-governance-rs","tag":"rc7-test","evidence_uri":"interop:rc7","claim":"rc7 namespace propagation test memory."
        }}
    }));
    assert!(propose.get("error").is_none(), "propose failed: {propose}");

    let interop_review = Command::new(bin()).args(["--store", &store, "--namespace", "interop-test", "review"]).output().unwrap();
    assert!(interop_review.status.success());
    let interop_text = String::from_utf8(interop_review.stdout).unwrap();
    assert!(interop_text.contains("rc7 namespace propagation test memory"), "{interop_text}");

    let default_patches = mcp_call(&store, "default", serde_json::json!({
        "jsonrpc":"2.0","id":7,"method":"tools/call","params":{"name":"pi.list_patches","arguments":{"limit":20}}
    }));
    let default_patch_text = default_patches["result"]["content"][0]["text"].as_str().unwrap();
    assert!(!default_patch_text.contains("rc7 namespace propagation test memory"), "{default_patch_text}");

    let retrieve = mcp_call(&store, "interop-test", serde_json::json!({
        "jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"pi.retrieve_context","arguments":{"query":"rc7 namespace propagation", "include_contested": true}}
    }));
    assert_eq!(retrieve["result"]["structuredContent"]["namespace"], "interop-test");

    let doctor = mcp_call(&store, "interop-test", serde_json::json!({
        "jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"pi.doctor","arguments":{}}
    }));
    assert_eq!(doctor["result"]["structuredContent"]["current_namespace"], "interop-test");
}
