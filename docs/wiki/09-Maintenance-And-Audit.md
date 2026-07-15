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

## Operational quality inspection

Use `pi graph`, `pi quality memory`, and `pi quality relationship` to inspect current memory structure and review signals. These reports are bounded and read-only. A low score is a prompt for review, not an automatic mutation or deletion decision.

Equivalent MCP tools are `pi.memory_graph`, `pi.memory_quality`, `pi.relationship_quality`, `pi.recall_effectiveness`, and `pi.store_quality`. Recall telemetry is disabled by default, bounded when enabled, and excluded from ordinary exports. Recall X-ray stores aggregate exclusion reasons; `recall-feedback` records explicit successful, corrected, or ignored outcomes. Use `pi simulate-patch <patch-id>` to inspect predicted governance and quality effects before applying a proposed patch. Use `pi procedure-candidates` and `pi failure-analysis` for review-only workflow and failure reports; neither command creates memory, patches, inquiries, or skills.

## Portable Workflow Parity

`v1.0.0` adds deterministic portable memory workflow parity: `memory-worth`, `capture`, `inbox`, `context`, `session add/search/decisions`, `recall-xray`, explicit L1/L2/L3 layers, trust class, durability, source kind, and minimal verification gates. Capture creates candidates or L3 evidence only; it does not silently apply durable L1/L2 memory. L1 is never auto-applied. L3 is session/evidence context, not authoritative memory.


## Distribution and MCP Sharing

Repository: https://github.com/Mont3ll/pi-governance-rs
License: MIT OR Apache-2.0

Install from source with `cargo build -p pi-governance-rs`, from Git with `cargo install --git https://github.com/Mont3ll/pi-governance-rs --tag v1.1.0 pi-governance-rs`, or from crates.io with `cargo install pi-governance-rs`. `pi-governance-rs` is a standalone local stdio MCP server by default; it does not provide a hosted service. It remains compatible with `pi-persistent-intelligence` through the shared PI memory contract.
