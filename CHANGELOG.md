# Changelog

## Unreleased

Added:
- Bounded read-only memory graph reports through CLI and MCP.
- Per-record memory quality and relationship quality reports with versioned heuristic signals.
- Accurate recall budget omission counts.
- Release-audit validation against the actual MCP tool registry.
- Disabled-by-default, bounded recall telemetry that stores query hashes rather than raw query text.
- Recall-effectiveness and aggregate store-quality reports through CLI and MCP.
- Read-only patch simulation with predicted state and quality deltas.

## v1.0.3

The public crates.io package is available as `pi-governance-rs`.

Install:

```bash
cargo install pi-governance-rs
```

Changed:
- Published the user-facing binary package as `pi-governance-rs`.
- Kept the installed command as `pi`.
- Updated public documentation for installation, MCP setup, usage, compatibility, and local-first security.
- Fixed packaged documentation used by the binary crate.

Existing CLI commands and MCP tools continue to work.

## v1.0.2

Published the internal Rust crates used by the `pi-governance-rs` binary package.

Changed:
- Published the supporting `pi-governance-*` library crates.
- Fixed packaged documentation used by the MCP crate.

## v1.0.1

Prepared the crates.io package identity.

Changed:
- Renamed the public Cargo package from `pi-cli` to `pi-governance-rs`.
- Namespaced supporting crates under `pi-governance-*`.
- Kept the installed command as `pi`.

## v1.0.0

First stable public release.

Included:
- local-first governed memory store
- CLI and local stdio MCP server
- explicit L1/L2/L3 memory layers
- memory-worth scoring
- capture and inbox workflows
- patch-based durable memory changes
- scoped context retrieval
- session add/search/decisions
- recall-xray
- import/export
- maintenance scan
- compatibility notes for `pi-persistent-intelligence`

## Earlier milestones

- v0.10.x: release hardening and adapter polish
- v0.9.x: policy profiles and operating modes
- v0.8.x: namespace isolation
- v0.7.x: deterministic retrieval improvements
- v0.6.x: portable import/export
- v0.5.x: belief revision, reinforcement, tombstones, and contests
- v0.4.x: schema migrations
- v0.3.x: store locking and schema versioning
- v0.2.x: patch visibility and safer apply flow
- v0.1.x: initial Rust PI implementation
