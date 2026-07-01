# MCP Setup

PI supports MCP stdio setup for OpenCode, Codex CLI, and PI agent/shared MCP. If existing client support is present for Claude, Cursor, or Inspector-style testing, use the generated config as the source of truth and run `mcp-doctor` before relying on a client.

## OpenCode Example

```bash
pi mcp-config opencode --command /path/to/pi --store /path/to/.pi --namespace default
pi mcp-install opencode --command /path/to/pi --store /path/to/.pi --namespace default
pi mcp-install opencode --command /path/to/pi --store /path/to/.pi --namespace default --yes
pi mcp-doctor opencode --command /path/to/pi --store /path/to/.pi --namespace default
```

## Codex CLI

```bash
pi mcp-config codex --command /path/to/pi --store /path/to/.pi --namespace default
pi mcp-doctor codex --command /path/to/pi --store /path/to/.pi --namespace default
```

## PI Agent / Shared MCP

```bash
pi mcp-config pi-agent --command /path/to/pi --store /path/to/.pi --namespace default
pi mcp-doctor pi-agent --command /path/to/pi --store /path/to/.pi --namespace default
```

Restart the client after installation. Client-prefixed tool names may look like `pi.retrieve_context`, `pi-governance_pi_retrieve_context`, `pi_governance_pi.retrieve_context`, or `mcp__pi_governance__.pi_retrieve_context`.


---

Related: [Wiki index](../WIKI_INDEX.md), [Deployment checklist](../DEPLOYMENT_CHECKLIST.md), [Release strategy](../RELEASE_STRATEGY.md), [Stable v1 gate](../STABLE_V1_GATE.md).

## Portable Workflow Parity

`v1.0.0` adds deterministic portable memory workflow parity: `memory-worth`, `capture`, `inbox`, `context`, `session add/search/decisions`, `recall-xray`, explicit L1/L2/L3 layers, trust class, durability, source kind, and minimal verification gates. Capture creates candidates or L3 evidence only; it does not silently apply durable L1/L2 memory. L1 is never auto-applied. L3 is session/evidence context, not authoritative memory.


## Distribution and MCP Sharing

Repository: https://github.com/Mont3ll/pi-governance-rs
License: MIT OR Apache-2.0

Install from source with `cargo build -p pi-governance-rs`, from Git with `cargo install --git https://github.com/Mont3ll/pi-governance-rs --tag v1.0.2 pi-governance-rs`, or from crates.io with `cargo install pi-governance-rs` after crates.io publishing is explicitly approved. `pi-governance-rs` is a standalone local stdio MCP server by default; it does not provide a hosted service in v1.0.0. It remains compatible with `pi-persistent-intelligence` through the shared PI memory contract.
