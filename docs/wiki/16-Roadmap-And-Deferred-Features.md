# Roadmap and Deferred Features

## Post-stable Quality Improvements

- capture candidates / `pi capture --stdin`
- persistent FTS/BM25 index if needed
- memory capsules
- relationship edges
- maintenance auto-suggestions as governed patches
- deeper pi-persistent-intelligence schema integration

## Product Expansion

- dashboard/TUI
- hosted MCP endpoint
- connectors
- vector backend
- graph backend
- team RBAC/SSO
- central audit logs
- cloud sync

## Already Moved Into rc.8

- MCP `inspect_record`
- review queue actions
- maintenance scan
- local lexical/hybrid retrieval hardening
- redacted export hardening
- schema documentation


---

Related: [Wiki index](../WIKI_INDEX.md), [Deployment checklist](../DEPLOYMENT_CHECKLIST.md), [Release strategy](../RELEASE_STRATEGY.md), [Stable v1 gate](../STABLE_V1_GATE.md).

## Moved Into v1.0.0

- automatic correction capture as deterministic candidate capture
- manual memory_write equivalent through `pi capture`
- memory-worth scoring
- inbox candidate workflow
- scoped context injection
- session search
- session decisions
- minimal trust classes/durability/source kind
- minimal verification gates
- recall x-ray
- explicit L1/L2/L3 layers
- minimal reinforcement event compatibility

## Still Deferred After v1.0.0

- deep reinforcement maintenance weighting
- LLM consolidation
- qmd semantic search
- vault integration
- background job queue
- memory graph
- memory timeline
- procedure candidates
- skill draft artifacts
- meta-consolidation automation
- dashboard/TUI
- hosted MCP
- connectors
- vector backend
- graph backend
- team RBAC/SSO
- cloud sync


## Distribution and MCP Sharing

Repository: https://github.com/Mont3ll/pi-governance-rs
License: MIT OR Apache-2.0

Install from source with `cargo build -p pi-governance-rs`, from Git with `cargo install --git https://github.com/Mont3ll/pi-governance-rs --tag v1.0.1 pi-governance-rs`, or from crates.io with `cargo install pi-governance-rs` after crates.io publishing is explicitly approved. `pi-governance-rs` is a standalone local stdio MCP server by default; it does not provide a hosted service in v1.0.0. It remains compatible with `pi-persistent-intelligence` through the shared PI memory contract.
