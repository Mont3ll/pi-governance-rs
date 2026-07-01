# Release Notes

## Current release: v1.0.3

`v1.0.3` is the current user-facing crates.io release.

Install:

```bash
cargo install pi-governance-rs
pi --version
```

The installed command is `pi`.

## What PI Governance provides

PI Governance is a local-first governed memory CLI and MCP stdio server for AI agents.

It includes:

- local JSONL memory stores
- capture and memory-worth scoring
- inbox and review workflows
- patch-based durable memory changes
- L1/L2/L3 memory layers
- trust, durability, source, evidence, and namespace metadata
- scoped context output for agent tasks
- session add/search/decisions
- recall-xray
- import/export
- local stdio MCP tools

## Safety model

- Capture creates candidates or session evidence; it does not silently apply durable memory.
- L1 identity memory requires review.
- Session data is context/evidence, not authoritative durable memory.
- Imported stores and patches should be reviewed before use.
- Secrets, credentials, private keys, passwords, and high-risk personal data should not be stored as durable memory.

## Installation options

Install from crates.io:

```bash
cargo install pi-governance-rs
```

Install from Git:

```bash
cargo install --git https://github.com/Mont3ll/pi-governance-rs --tag v1.0.3 pi-governance-rs
```

Build from source:

```bash
git clone https://github.com/Mont3ll/pi-governance-rs.git
cd pi-governance-rs
cargo build -p pi-governance-rs
./target/debug/pi --version
```

## MCP usage

PI Governance runs as a local stdio MCP server. MCP clients connect by launching your local `pi` command.

```bash
pi mcp-config codex --command "$(which pi)" --store /path/to/.pi --namespace default
pi mcp-doctor codex --command "$(which pi)" --store /path/to/.pi --namespace default
```

## Compatibility

`pi-governance-rs` can be used by itself through CLI or MCP.

It can also interoperate with `pi-persistent-intelligence` through the shared PI memory contract and compatible import/export formats.
