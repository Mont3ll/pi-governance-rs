# PI Governance

**Current version:** `v1.0.0`

PI Governance is local-first governed memory for AI agents: a Rust CLI and MCP stdio server that stores durable memory in local JSONL files and requires changes to pass through reviewable patches. It is for maintainers and agent users who want auditable, namespace-scoped agent memory rather than silent belief rewrites.

## Who Should Use It

Use PI if you run local AI coding agents, want memory changes with evidence, need an MCP-compatible governed memory server, or are preparing/testing the stable `v1.0.0` release path.

## Quick Links

- [Installation](03-Installation.md)
- [MCP setup](05-MCP-Setup.md)
- [CLI guide](04-CLI-Guide.md)
- [QA matrix](14-QA-And-Test-Matrix.md)
- [Release checklist](13-Release-And-Deployment.md)

## Portable Workflow Parity

`v1.0.0` adds deterministic portable memory workflow parity: `memory-worth`, `capture`, `inbox`, `context`, `session add/search/decisions`, `recall-xray`, explicit L1/L2/L3 layers, trust class, durability, source kind, and minimal verification gates. Capture creates candidates or L3 evidence only; it does not silently apply durable L1/L2 memory. L1 is never auto-applied. L3 is session/evidence context, not authoritative memory.


## Distribution and MCP Sharing

Repository: https://github.com/Mont3ll/pi-governance-rs
License: MIT OR Apache-2.0

Install from source with `cargo build -p pi-governance-rs`, from Git with `cargo install --git https://github.com/Mont3ll/pi-governance-rs --tag v1.1.0 pi-governance-rs`, or from crates.io with `cargo install pi-governance-rs`. `pi-governance-rs` is a standalone local stdio MCP server by default; it does not provide a hosted service. It remains compatible with `pi-persistent-intelligence` through the shared PI memory contract.
