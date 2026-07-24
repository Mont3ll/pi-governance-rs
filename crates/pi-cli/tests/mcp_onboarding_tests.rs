use std::fs;
use std::process::{Command, Stdio};

fn bin() -> String {
    env!("CARGO_BIN_EXE_pi").to_string()
}
fn tmp_dir(name: &str) -> String {
    let path = format!("/tmp/pi-cli-{name}-{}", std::process::id());
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();
    path
}

#[test]
fn mcp_config_new_clients_emit_expected_shapes() {
    let out = Command::new(bin())
        .args([
            "mcp-config",
            "opencode",
            "--command",
            "/tmp/pi-bin",
            "--store",
            "/tmp/pi-store",
            "--namespace",
            "interop-test",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["mcp"]["pi-governance"]["type"], "local");
    assert_eq!(json["mcp"]["pi-governance"]["command"][0], "/tmp/pi-bin");
    assert_eq!(json["mcp"]["pi-governance"]["enabled"], true);

    let out = Command::new(bin())
        .args([
            "mcp-config",
            "codex",
            "--command",
            "/tmp/pi-bin",
            "--store",
            "/tmp/pi-store",
            "--namespace",
            "interop-test",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let text = String::from_utf8(out.stdout).unwrap();
    assert!(text.contains("[mcp_servers.pi-governance]"));
    assert!(text.contains("command = \"/tmp/pi-bin\""));
    assert!(text.contains("/tmp/pi-store"));

    let out = Command::new(bin())
        .args([
            "mcp-config",
            "pi-agent",
            "--command",
            "/tmp/pi-bin",
            "--store",
            "/tmp/pi-store",
            "--namespace",
            "interop-test",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(
        json["mcpServers"]["pi-governance"]["command"],
        "/tmp/pi-bin"
    );

    for client in ["claude", "cursor", "inspector"] {
        let out = Command::new(bin())
            .args(["mcp-config", client, "--command", "/tmp/pi-bin"])
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{client}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
}

#[test]
fn mcp_install_merges_and_preserves_existing_servers() {
    let dir = tmp_dir("mcp-install");
    let pi_agent = format!("{dir}/mcp.json");
    fs::write(
        &pi_agent,
        r#"{"mcpServers":{"headroom":{"command":"headroom","args":["mcp"]}}}"#,
    )
    .unwrap();
    let before = fs::read_to_string(&pi_agent).unwrap();
    let dry = Command::new(bin())
        .args([
            "mcp-install",
            "pi-agent",
            "--config",
            &pi_agent,
            "--command",
            &bin(),
            "--store",
            "/tmp/pi-store",
            "--namespace",
            "interop-test",
            "--dry-run",
        ])
        .output()
        .unwrap();
    assert!(
        dry.status.success(),
        "{}",
        String::from_utf8_lossy(&dry.stderr)
    );
    assert_eq!(fs::read_to_string(&pi_agent).unwrap(), before);
    let out = Command::new(bin())
        .args([
            "mcp-install",
            "pi-agent",
            "--config",
            &pi_agent,
            "--command",
            &bin(),
            "--store",
            "/tmp/pi-store",
            "--namespace",
            "interop-test",
            "--yes",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let json: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&pi_agent).unwrap()).unwrap();
    assert!(json["mcpServers"]["headroom"].is_object());
    assert!(json["mcpServers"]["pi-governance"].is_object());
    assert!(fs::read_dir(&dir).unwrap().any(|e| e
        .unwrap()
        .file_name()
        .to_string_lossy()
        .contains("mcp.json.backup.")));

    let opencode = format!("{dir}/opencode.jsonc");
    fs::write(&opencode, r#"{"$schema":"https://opencode.ai/config.json","mcp":{"headroom":{"type":"local","command":["headroom","mcp"],"enabled":true}}}"#).unwrap();
    let out = Command::new(bin())
        .args([
            "mcp-install",
            "opencode",
            "--config",
            &opencode,
            "--command",
            &bin(),
            "--store",
            "/tmp/pi-store",
            "--namespace",
            "interop-test",
            "--yes",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let json: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&opencode).unwrap()).unwrap();
    assert!(json["mcp"]["headroom"].is_object());
    assert!(json["mcp"]["pi-governance"].is_object());

    let codex = format!("{dir}/config.toml");
    fs::write(&codex, "model = \"gpt\"\n\n[mcp_servers.headroom]\ncommand = \"headroom\"\nargs = [\"mcp\"]\nenabled = true\n").unwrap();
    let out = Command::new(bin())
        .args([
            "mcp-install",
            "codex",
            "--config",
            &codex,
            "--command",
            &bin(),
            "--store",
            "/tmp/pi-store",
            "--namespace",
            "interop-test",
            "--yes",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let text = fs::read_to_string(&codex).unwrap();
    assert!(text.contains("[mcp_servers.headroom]"));
    assert!(text.contains("[mcp_servers.pi-governance]"));
}

#[test]
fn mcp_install_prompts_and_rejects_invalid_config() {
    let dir = tmp_dir("mcp-install-invalid");
    let cfg = format!("{dir}/bad.json");
    fs::write(&cfg, "{ invalid").unwrap();
    let out = Command::new(bin())
        .args([
            "mcp-install",
            "pi-agent",
            "--config",
            &cfg,
            "--command",
            &bin(),
            "--yes",
        ])
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert_eq!(fs::read_to_string(&cfg).unwrap(), "{ invalid");

    let cfg2 = format!("{dir}/needs-confirm.json");
    fs::write(&cfg2, "{}").unwrap();
    let out = Command::new(bin())
        .args([
            "mcp-install",
            "pi-agent",
            "--config",
            &cfg2,
            "--command",
            &bin(),
        ])
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(!out.status.success());
}

#[test]
fn mcp_doctor_reports_failures_and_json_success() {
    let dir = tmp_dir("mcp-doctor");
    let missing = format!("{dir}/missing.json");
    let out = Command::new(bin())
        .args([
            "mcp-doctor",
            "pi-agent",
            "--config",
            &missing,
            "--store",
            &dir,
            "--namespace",
            "interop-test",
        ])
        .output()
        .unwrap();
    assert!(!out.status.success());

    let cfg = format!("{dir}/mcp.json");
    fs::write(&cfg, "{\"mcpServers\":{}}").unwrap();
    let out = Command::new(bin())
        .args([
            "mcp-doctor",
            "pi-agent",
            "--config",
            &cfg,
            "--store",
            &dir,
            "--namespace",
            "interop-test",
        ])
        .output()
        .unwrap();
    assert!(!out.status.success());

    assert!(Command::new(bin())
        .args(["--store", &dir, "demo", "--reset"])
        .status()
        .unwrap()
        .success());
    assert!(Command::new(bin())
        .args([
            "mcp-install",
            "pi-agent",
            "--config",
            &cfg,
            "--command",
            &bin(),
            "--store",
            &dir,
            "--namespace",
            "interop-test",
            "--yes"
        ])
        .status()
        .unwrap()
        .success());
    let out = Command::new(bin())
        .args([
            "mcp-doctor",
            "pi-agent",
            "--config",
            &cfg,
            "--store",
            &dir,
            "--namespace",
            "interop-test",
            "--json",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["checks"]["direct_tools_list"], true);
    assert_eq!(json["tools"]["pi.retrieve_context"], true);
    assert!(json["tools"].get("pi.inspect_record").is_some());
}
