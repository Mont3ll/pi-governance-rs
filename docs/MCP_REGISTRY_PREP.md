# MCP Registry Preparation

Do not submit to the MCP Registry without explicit approval.

```yaml
name: pi-governance-rs
description: Local-first governed memory CLI and MCP stdio server for AI agents.
runtime: local stdio
language: Rust
install: cargo install pi-governance-rs
binary: pi
server command: pi --store /path/to/.pi --namespace default mcp-stdio
categories:
  - memory
  - developer-tools
  - ai-agents
  - governance
security: local-first, no network by default, no hosted service
license: MIT OR Apache-2.0
repository: https://github.com/Mont3ll/pi-governance-rs
```

## Major MCP tools

- `pi.retrieve_context`
- `pi.propose_record`
- `pi.supersede_record`
- `pi.tombstone_record`
- `pi.reinforce_record`
- `pi.contest_record`
- `pi.resolve_contest`
- `pi.apply_patch`
- `pi.list_patches`
- `pi.inspect_patch`
- `pi.inspect_record`
- `pi.list_records`
- `pi.maintenance_scan`
- `pi.score_memory_worth`
- `pi.capture_candidates`
- `pi.build_context`
- `pi.session_add`
- `pi.session_search`
- `pi.session_decisions`
- `pi.recall_xray`
- `pi.doctor`
- `pi.smoke_test`

## Security notes

The server is local stdio. The MCP client launches the `pi` binary as a subprocess. It does not expose a hosted HTTP endpoint by default. Stores remain local unless the user exports, imports, copies, or syncs them.


Release lineage: v1.0.0 was the first GitHub/source release. v1.0.1 prepared package identity but crates.io publication was partial. v1.0.3 completes crates.io publishing and is the public crates.io install target.
