use pi_governance_engine::GovernanceEngine;
use pi_governance_mcp::{registered_tool_names, McpStdioServer};
use pi_governance_store::JsonlStore;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_store_dir(test_name: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be after unix epoch")
        .as_nanos();

    std::env::temp_dir().join(format!("pi-mcp-{test_name}-{}-{nonce}", std::process::id()))
}

#[test]
fn stdio_server_can_be_constructed() {
    let root = temp_store_dir("construct");
    let engine = GovernanceEngine::new(JsonlStore::new(root));
    let _server = McpStdioServer::new(engine);
}

#[test]
fn initialize_exposes_distinct_resolved_and_configured_store_identities() {
    let existing = temp_store_dir("identity-existing");
    std::fs::create_dir_all(&existing).unwrap();
    let missing = temp_store_dir("identity-missing");
    let existing_server = McpStdioServer::new_with_namespace(
        GovernanceEngine::new(JsonlStore::new(&existing)),
        "persistent-intelligence".into(),
    );
    let missing_server = McpStdioServer::new_with_namespace(
        GovernanceEngine::new(JsonlStore::new(&missing)),
        "persistent-intelligence".into(),
    );

    let resolved = existing_server.initialize_result(serde_json::json!({}));
    let configured = missing_server.initialize_result(serde_json::json!({}));

    assert_eq!(resolved["piStoreIdentity"]["resolved"], true);
    assert_eq!(
        resolved["piStoreIdentity"]["namespace"],
        "persistent-intelligence"
    );
    assert_eq!(configured["piStoreIdentity"]["resolved"], false);
    assert_ne!(
        resolved["piStoreIdentity"]["store"],
        configured["piStoreIdentity"]["store"]
    );
    std::fs::remove_dir_all(existing).unwrap();
}

#[test]
fn registered_tool_names_come_from_the_canonical_registry() {
    let names = registered_tool_names();
    assert!(names.iter().any(|name| name == "pi.retrieve_context"));
    assert!(names.iter().any(|name| name == "pi.recall_xray"));
    assert!(!names.iter().any(|name| name.trim().is_empty()));
}

#[test]
fn observability_tools_are_registered() {
    let names = registered_tool_names();
    for expected in [
        "pi.memory_graph",
        "pi.memory_quality",
        "pi.relationship_quality",
        "pi.recall_effectiveness",
        "pi.store_quality",
        "pi.simulate_patch",
        "pi.procedure_candidates",
        "pi.failure_analysis",
        "pi.recall_feedback",
    ] {
        assert!(
            names.iter().any(|name| name == expected),
            "missing {expected}"
        );
    }
}

#[test]
fn retrieve_context_schema_includes_deterministic_retrieval_fields() {
    let root = temp_store_dir("schema");
    let engine = GovernanceEngine::new(JsonlStore::new(root));
    let server = McpStdioServer::new(engine);
    let tools = server.tool_definitions();
    let retrieve = tools
        .as_array()
        .unwrap()
        .iter()
        .find(|tool| tool["name"] == "pi.retrieve_context")
        .expect("retrieve tool exists");
    let props = &retrieve["inputSchema"]["properties"];
    assert!(props["explain"].is_object());
    assert!(props["classes"].is_object());
    assert!(props["include_global"].is_object());
    assert!(props["include_contested"].is_object());
    assert!(props["min_confidence"].is_object());
}
