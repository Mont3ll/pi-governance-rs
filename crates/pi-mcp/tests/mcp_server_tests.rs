use pi_governance::GovernanceEngine;
use pi_mcp::McpStdioServer;
use pi_store::JsonlStore;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_store_dir(test_name: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be after unix epoch")
        .as_nanos();

    std::env::temp_dir().join(format!(
        "pi-mcp-{test_name}-{}-{nonce}",
        std::process::id()
    ))
}

#[test]
fn stdio_server_can_be_constructed() {
    let root = temp_store_dir("construct");
    let engine = GovernanceEngine::new(JsonlStore::new(root));
    let _server = McpStdioServer::new(engine);
}
