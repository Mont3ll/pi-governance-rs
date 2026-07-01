# Agent Interoperability

## rc.8 Tested Clients

| Client | Status |
| --- | --- |
| OpenCode | install/doctor pass; live rc.9 client run incomplete due environmental/client-run limitation |
| Codex CLI | pass |
| PI agent | pass |

## Tested Capabilities

`retrieve_context`, `propose_record`, `list_patches`, `inspect_patch`, `inspect_record`, `list_records`, `maintenance_scan`, `doctor`, `smoke_test`, review action discovery, namespace propagation, and structuredContent object compatibility.

## rc.8 Interoperability Prompt

Use the release-candidate interoperability prompt from the project release-preparation notes for full client validation. The prompt should verify direct MCP `tools/list`, tool calls, namespace propagation, structured content, and client-visible prefixed tool names.


---

Related: [Wiki index](../WIKI_INDEX.md), [Deployment checklist](../DEPLOYMENT_CHECKLIST.md), [Release strategy](../RELEASE_STRATEGY.md), [Stable v1 gate](../STABLE_V1_GATE.md).

## Portable Workflow Parity

`v1.0.0` adds deterministic portable memory workflow parity: `memory-worth`, `capture`, `inbox`, `context`, `session add/search/decisions`, `recall-xray`, explicit L1/L2/L3 layers, trust class, durability, source kind, and minimal verification gates. Capture creates candidates or L3 evidence only; it does not silently apply durable L1/L2 memory. L1 is never auto-applied. L3 is session/evidence context, not authoritative memory.


## Distribution and MCP Sharing

Repository: https://github.com/Mont3ll/pi-governance-rs
License: MIT OR Apache-2.0

Install from source with `cargo build -p pi-governance-rs`, from Git with `cargo install --git https://github.com/Mont3ll/pi-governance-rs --tag v1.0.2 pi-governance-rs`, or from crates.io with `cargo install pi-governance-rs` after crates.io publishing is explicitly approved. `pi-governance-rs` is a standalone local stdio MCP server by default; it does not provide a hosted service in v1.0.0. It remains compatible with `pi-persistent-intelligence` through the shared PI memory contract.
