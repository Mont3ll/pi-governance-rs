# Maintenance and Audit

Use maintenance and audit commands before releases and when diagnosing stores.

```bash
pi --store .pi maintenance scan
pi --store .pi doctor
pi --store .pi namespace doctor
pi --store .pi policy doctor
pi smoke-test
pi release-audit
```

Maintenance findings include `pending_patches`, `old_pending_patches`, `deferred_patches`, `rejected_patches`, `contested_records`, `superseded_records`, `tombstoned_records`, `records_without_evidence`, `low_confidence_records`, `records_with_empty_claims`, `possible_duplicate_claims`, `namespace_summary`, and `policy_summary`.


---

Related: [Wiki index](../WIKI_INDEX.md), [Deployment checklist](../DEPLOYMENT_CHECKLIST.md), [Release strategy](../RELEASE_STRATEGY.md), [Stable v1 gate](../STABLE_V1_GATE.md).

## Portable Workflow Parity

`v1.0.0` adds deterministic portable memory workflow parity: `memory-worth`, `capture`, `inbox`, `context`, `session add/search/decisions`, `recall-xray`, explicit L1/L2/L3 layers, trust class, durability, source kind, and minimal verification gates. Capture creates candidates or L3 evidence only; it does not silently apply durable L1/L2 memory. L1 is never auto-applied. L3 is session/evidence context, not authoritative memory.


## Distribution and MCP Sharing

Repository: https://github.com/Mont3ll/pi-governance-rs
License: MIT OR Apache-2.0

Install from source with `cargo build -p pi-cli`, from Git with `cargo install --git https://github.com/Mont3ll/pi-governance-rs --tag v1.0.0 pi-cli`, or from crates.io with `cargo install pi-cli` after crates.io publishing is explicitly approved. `pi-governance-rs` is a standalone local stdio MCP server by default; it does not provide a hosted service in v1.0.0. It remains compatible with `pi-persistent-intelligence` through the shared PI memory contract.
