# Retrieval Guide

PI an earlier release candidate supports deterministic, lexical, and hybrid local retrievers. No embeddings are used in an earlier release candidate. No vector database is required. Retrieval remains local and deterministic.

```bash
pi --store .pi retrieve "release workflow" --retriever deterministic --explain
pi --store .pi retrieve "release workflow" --retriever lexical --explain
pi --store .pi retrieve "release workflow" --retriever hybrid --explain
```

`--explain` reports diagnostics such as matched terms, matched fields, score components, empty-result explanations, and budget packing. Use namespace, project, status, `include-contested`, and `min-confidence` filters to narrow context.


---

Related: [Wiki index](../WIKI_INDEX.md), [Deployment checklist](../DEPLOYMENT_CHECKLIST.md), [Release strategy](../RELEASE_STRATEGY.md), [Stable v1 gate](../STABLE_V1_GATE.md).

## Portable Workflow Parity

`v1.0.0` adds deterministic portable memory workflow parity: `memory-worth`, `capture`, `inbox`, `context`, `session add/search/decisions`, `recall-xray`, explicit L1/L2/L3 layers, trust class, durability, source kind, and minimal verification gates. Capture creates candidates or L3 evidence only; it does not silently apply durable L1/L2 memory. L1 is never auto-applied. L3 is session/evidence context, not authoritative memory.


## Distribution and MCP Sharing

Repository: https://github.com/Mont3ll/pi-governance-rs
License: MIT OR Apache-2.0

Install from source with `cargo build -p pi-governance-rs`, from Git with `cargo install --git https://github.com/Mont3ll/pi-governance-rs --tag v1.1.0 pi-governance-rs`, or from crates.io with `cargo install pi-governance-rs`. `pi-governance-rs` is a standalone local stdio MCP server by default; it does not provide a hosted service. It remains compatible with `pi-persistent-intelligence` through the shared PI memory contract.
