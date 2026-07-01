# Core Concepts

- **Store:** local JSONL source of truth, usually `.pi`.
- **Record:** governed memory claim with class, claim, confidence, evidence, namespace, project fields, and status.
- **Patch:** proposed mutation. Patch statuses include proposed/pending, applied, rejected, and deferred states as exposed by CLI/MCP.
- **Evidence:** reference that explains why the claim or patch exists.
- **Namespace:** isolation scope for records, patches, retrieval, MCP clients, and tests.
- **Policy profile:** standard, strict, or permissive governance behavior for review requirements.
- **Record statuses:** active, contested, superseded, and tombstoned. Tombstones preserve history rather than hard-deleting claims.
- **Retriever modes:** deterministic, lexical, and hybrid. an earlier release candidate uses no embeddings and requires no vector database.
- **Maintenance findings:** scan results such as pending patches, contested records, low-confidence records, missing evidence, duplicates, namespace summary, and policy summary.
- **Redaction metadata:** metadata attached to redacted exports so reviewers know redaction was best-effort.
- **MCP structured content:** object-shaped responses intended for client compatibility.


---

Related: [Wiki index](../WIKI_INDEX.md), [Deployment checklist](../DEPLOYMENT_CHECKLIST.md), [Release strategy](../RELEASE_STRATEGY.md), [Stable v1 gate](../STABLE_V1_GATE.md).

## Portable Workflow Parity

`v1.0.0` adds deterministic portable memory workflow parity: `memory-worth`, `capture`, `inbox`, `context`, `session add/search/decisions`, `recall-xray`, explicit L1/L2/L3 layers, trust class, durability, source kind, and minimal verification gates. Capture creates candidates or L3 evidence only; it does not silently apply durable L1/L2 memory. L1 is never auto-applied. L3 is session/evidence context, not authoritative memory.


## Distribution and MCP Sharing

Repository: https://github.com/Mont3ll/pi-governance-rs
License: MIT OR Apache-2.0

Install from source with `cargo build -p pi-governance-rs`, from Git with `cargo install --git https://github.com/Mont3ll/pi-governance-rs --tag v1.0.2 pi-governance-rs`, or from crates.io with `cargo install pi-governance-rs` after crates.io publishing is explicitly approved. `pi-governance-rs` is a standalone local stdio MCP server by default; it does not provide a hosted service in v1.0.0. It remains compatible with `pi-persistent-intelligence` through the shared PI memory contract.
