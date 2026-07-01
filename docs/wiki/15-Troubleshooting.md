# Troubleshooting

## Client Shows Zero PI Tools

Run direct `tools/list`, verify binary path, restart client, and check client-prefixed names.

## Direct MCP Works but Client Does Not

Inspect generated config, ensure the client is reading the expected config, and restart stale server processes.

## Wrong Namespace or No Records Visible

Confirm `--namespace`, store path, and MCP config namespace. Use `namespace list` and `namespace doctor`.

## structuredContent Expected Record, Received Array

Use rc.8 or later docs/tests; structuredContent compatibility was specifically validated in rc.8.

## Wrong Binary, Store, or Config Path

Use absolute generic paths such as `/path/to/pi` and `/path/to/.pi` in configs. Run `mcp-doctor`.

## Retrieval Returns Zero Records

Use `--explain`, check namespace, status filters, project filters, `include-contested`, and `min-confidence`.

## Proposal Remains Pending

Inspect the patch, review it, then apply, reject, or defer.

## Redacted Export Does Not Replace Review

Redaction is best-effort. Review exports before sharing.


---

Related: [Wiki index](../WIKI_INDEX.md), [Deployment checklist](../DEPLOYMENT_CHECKLIST.md), [Release strategy](../RELEASE_STRATEGY.md), [Stable v1 gate](../STABLE_V1_GATE.md).

## Portable Workflow Parity

`v1.0.0` adds deterministic portable memory workflow parity: `memory-worth`, `capture`, `inbox`, `context`, `session add/search/decisions`, `recall-xray`, explicit L1/L2/L3 layers, trust class, durability, source kind, and minimal verification gates. Capture creates candidates or L3 evidence only; it does not silently apply durable L1/L2 memory. L1 is never auto-applied. L3 is session/evidence context, not authoritative memory.


## Distribution and MCP Sharing

Repository: https://github.com/Mont3ll/pi-governance-rs
License: MIT OR Apache-2.0

Install from source with `cargo build -p pi-governance-rs`, from Git with `cargo install --git https://github.com/Mont3ll/pi-governance-rs --tag v1.0.2 pi-governance-rs`, or from crates.io with `cargo install pi-governance-rs` after crates.io publishing is explicitly approved. `pi-governance-rs` is a standalone local stdio MCP server by default; it does not provide a hosted service in v1.0.0. It remains compatible with `pi-persistent-intelligence` through the shared PI memory contract.
