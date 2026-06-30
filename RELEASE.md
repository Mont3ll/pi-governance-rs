# Release Notes

`v1.0.0` is the stable public release.

## v1.0.0 portable PI memory workflow parity

`v1.0.0` is the current stable public release. It adds the missing portable workflow layer from the original PI memory workflow while keeping durable L1/L2 memory patch-governed.

Included in v1.0.0:

- explicit L1/L2/L3 memory layers
- optional `memory_kind` and `rule_type`
- trust class, durability, and source kind signals
- deterministic `memory-worth` scoring
- deterministic `capture` candidate extraction from text/stdin/files
- manual `memory_write` equivalent through `pi capture --target daily|long-term`
- `inbox` candidate review workflow
- scoped `context` output for non-PI agents
- local `session add`, `session search`, and `session decisions`
- `recall-xray` inclusion/exclusion diagnostics
- MCP tools for score/capture/context/session/recall workflows
- minimal verification gates for low-trust sources, L1 records, and secret-like content

Safety notes:

- Capture creates candidates or L3 session evidence; it does not silently apply durable L1/L2 records.
- L1 identity memory always requires review.
- L3 session/event data is append-only evidence context, not authoritative durable memory.
- Repository text, generated content, third-party documentation, and codebase analysis cannot bypass manual review.

## v1.0.0-rc.8 release-quality governance hardening

rc.8 added MCP record inspection parity, review queue actions, read-only maintenance scan, local deterministic lexical/hybrid retrieval modes, redacted export metadata, and schema documentation.

## Still Deferred

Deep reinforcement maintenance weighting, LLM consolidation, qmd semantic search, vault integration, background job queues, memory graph/timeline, procedure candidates, skill draft artifacts, meta-consolidation automation, dashboard/TUI, hosted MCP, connectors, vector backend, graph backend, team RBAC/SSO, and cloud sync remain deferred.


## Packaging and distribution

Repository: https://github.com/Mont3ll/pi-governance-rs

License: MIT OR Apache-2.0. See `LICENSE`, `LICENSE-APACHE`, and `LICENSE-MIT`.

Install from Git:

```bash
cargo install --git https://github.com/Mont3ll/pi-governance-rs --tag v1.0.0 pi-cli
```

Install from crates.io after explicit publication approval:

```bash
cargo install pi-cli
```

The MCP server is local stdio by default; no hosted MCP service is included in v1.0.0.
