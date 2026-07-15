use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn bin() -> String { env!("CARGO_BIN_EXE_pi").to_string() }

fn store(name: &str) -> String {
    let nonce = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    format!("/tmp/pi-cli-rc9-{name}-{nonce}")
}

fn run(args: &[&str]) -> std::process::Output {
    Command::new(bin()).args(args).output().expect("run pi")
}

fn assert_success(args: &[&str]) -> String {
    let out = run(args);
    assert!(out.status.success(), "command failed: {:?}\nstdout={}\nstderr={}", args, String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
    String::from_utf8(out.stdout).unwrap()
}

#[test]
fn memory_worth_scores_durable_and_secret_text() {
    let durable = assert_success(&["memory-worth", "Going forward, always run cargo test before release.", "--json"]);
    let json: serde_json::Value = serde_json::from_str(&durable).unwrap();
    assert_eq!(json["decision"], "candidate");
    assert_eq!(json["suggested_layer"], "l2_playbook");
    assert_eq!(json["suggested_memory_kind"], "instruction");
    assert_eq!(json["suggested_rule_type"], "workflow");

    let secret = assert_success(&["memory-worth", "api_key=super-secret-token", "--json"]);
    let json: serde_json::Value = serde_json::from_str(&secret).unwrap();
    assert_eq!(json["decision"], "reject");
    assert!(json["warnings"].as_array().unwrap().iter().any(|w| w.as_str().unwrap().contains("secret")));
}

#[test]
fn capture_creates_l2_candidate_and_daily_session_decision() {
    let s = store("capture");
    assert_success(&["demo", "--store", &s, "--reset"]);

    let captured = assert_success(&[
        "--store", &s,
        "capture", "--text", "don't use echo >> for notes, use sed instead", "--project", "pi-governance-rs", "--json",
    ]);
    let json: serde_json::Value = serde_json::from_str(&captured).unwrap();
    assert_eq!(json["applied"], false);
    let candidate = &json["candidates"].as_array().unwrap()[0];
    assert_eq!(candidate["suggested_layer"], "l2_playbook");
    assert_eq!(candidate["trust_class"], "user_correction");
    assert!(candidate["patch_id"].as_str().unwrap().starts_with("patch_"));

    let patches = assert_success(&["--store", &s, "list-patches"]);
    assert!(patches.contains(candidate["patch_id"].as_str().unwrap()));

    assert_success(&[
        "--store", &s,
        "capture", "--target", "daily", "--text", "#decision use canonical JSONL as the source of truth", "--project", "pi-governance-rs", "--json",
    ]);
    let decisions = assert_success(&["--store", &s, "session", "decisions", "--project", "pi-governance-rs"]);
    assert!(decisions.contains("use canonical JSONL"));
}

#[test]
fn inbox_context_recall_xray_and_session_workflow() {
    let s = store("workflow");
    assert_success(&["demo", "--store", &s, "--reset"]);
    assert_success(&[
        "--store", &s,
        "capture", "--text", "Always run cargo test and release-audit before stable tagging.", "--project", "pi-governance-rs",
    ]);
    let patches = assert_success(&["--store", &s, "list-patches"]);
    let patch_id = patches.lines().find_map(|line| line.split_whitespace().find(|part| part.starts_with("patch_"))).unwrap().to_string();

    let inbox = assert_success(&["--store", &s, "inbox", "--layer", "l2"]);
    assert!(inbox.contains(&patch_id));

    assert_success(&["--store", &s, "inbox", "--apply", &patch_id]);
    let ctx = assert_success(&["--store", &s, "context", "prepare stable release", "--project", "pi-governance-rs", "--format", "markdown"]);
    assert!(ctx.contains("## Governed Memory Context"));
    assert!(ctx.contains("### L2"));
    assert!(ctx.contains("release-audit"));

    let xray = assert_success(&["--store", &s, "recall-xray", "stable release", "--project", "pi-governance-rs", "--json"]);
    let json: serde_json::Value = serde_json::from_str(&xray).unwrap();
    assert_eq!(json["query"], "stable release");
    assert!(!json["included"].as_array().unwrap().is_empty());
    assert!(json["included"].as_array().unwrap()[0].get("layer").is_some());

    assert_success(&["--store", &s, "session", "add", "--text", "We debugged MCP namespace propagation. #decision keep namespace interop-test for agent testing.", "--project", "pi-governance-rs"]);
    let search = assert_success(&["--store", &s, "session", "search", "namespace propagation", "--project", "pi-governance-rs"]);
    assert!(search.contains("namespace propagation"));
    let decisions = assert_success(&["--store", &s, "session", "decisions", "--project", "pi-governance-rs"]);
    assert!(decisions.contains("keep namespace interop-test"));
}

#[test]
fn mcp_lists_and_calls_rc9_tools_with_object_content() {
    let s = store("mcp");
    assert_success(&["demo", "--store", &s, "--reset"]);

    let request = "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"tools/list\",\"params\":{}}\n";
    let mut child = Command::new(bin())
        .args(["--store", &s, "mcp-stdio"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn().unwrap();
    use std::io::Write;
    child.stdin.as_mut().unwrap().write_all(request.as_bytes()).unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success());
    let text = String::from_utf8(out.stdout).unwrap();
    for tool in ["pi.score_memory_worth", "pi.capture_candidates", "pi.build_context", "pi.session_add", "pi.session_search", "pi.session_decisions", "pi.recall_xray", "pi.memory_graph", "pi.memory_quality", "pi.relationship_quality", "pi.recall_effectiveness", "pi.store_quality", "pi.simulate_patch", "pi.procedure_candidates", "pi.failure_analysis", "pi.recall_feedback"] {
        assert!(text.contains(tool), "missing {tool} in {text}");
    }

    let call = "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/call\",\"params\":{\"name\":\"pi.score_memory_worth\",\"arguments\":{\"observation\":\"Going forward, always run cargo test before release.\",\"project\":\"pi-governance-rs\"}}}\n";
    let mut child = Command::new(bin()).args(["--store", &s, "mcp-stdio"]).stdin(std::process::Stdio::piped()).stdout(std::process::Stdio::piped()).spawn().unwrap();
    child.stdin.as_mut().unwrap().write_all(call.as_bytes()).unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success());
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert!(json["result"]["structuredContent"].is_object());
    assert_eq!(json["result"]["structuredContent"]["decision"], "candidate");

    let call = "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"pi.memory_graph\",\"arguments\":{}}}\n";
    let mut child = Command::new(bin()).args(["--store", &s, "mcp-stdio"]).stdin(std::process::Stdio::piped()).stdout(std::process::Stdio::piped()).spawn().unwrap();
    child.stdin.as_mut().unwrap().write_all(call.as_bytes()).unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success());
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["result"]["structuredContent"]["mutation_performed"], false);
    assert_eq!(json["result"]["structuredContent"]["schema_version"], 1);
    assert!(json["result"]["structuredContent"]["nodes"].as_array().unwrap().len() <= 200);

    let call = "{\"jsonrpc\":\"2.0\",\"id\":4,\"method\":\"tools/call\",\"params\":{\"name\":\"pi.retrieve_context\",\"arguments\":{\"query\":\"release workflow\",\"format\":\"json\"}}}\n";
    let mut child = Command::new(bin()).args(["--store", &s, "mcp-stdio"]).stdin(std::process::Stdio::piped()).stdout(std::process::Stdio::piped()).spawn().unwrap();
    child.stdin.as_mut().unwrap().write_all(call.as_bytes()).unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success());
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert!(json["result"]["structuredContent"].get("records").is_none());
    assert!(json["result"]["structuredContent"]["blocks"].is_array());
}
