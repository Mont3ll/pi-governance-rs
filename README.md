# PI Governance

Local-first governed memory for AI agents.

PI Governance gives you a `pi` command and a local stdio MCP server for capturing, reviewing, retrieving, and sharing durable memory across AI tools. Your memory store stays on your machine by default.

## Why PI Governance

AI agents become more useful when they can remember project rules, decisions, preferences, corrections, and workflow notes. PI Governance keeps that memory explicit and reviewable:

- local JSONL stores you control
- reviewable changes before durable memory is updated
- evidence, scope, tags, confidence, and namespace metadata
- compact context retrieval for agent tasks
- MCP tools for clients that can run local stdio servers

## Install

Install from crates.io:

```bash
cargo install pi-governance-rs
pi --version
```

The installed command is `pi`.

Build from source:

```bash
git clone https://github.com/Mont3ll/pi-governance-rs.git
cd pi-governance-rs
cargo build -p pi-governance-rs
./target/debug/pi --version
```

## Quick Start

Create a demo store and inspect it:

```bash
pi demo --store .pi --reset
pi --store .pi doctor
pi --store .pi review
pi --store .pi retrieve "release workflow" --explain
```

Propose memory, review pending patches, and apply approved changes:

```bash
pi --store .pi propose workflow "Run tests before publishing." --tag release --evidence conversation:today
pi --store .pi review
pi --store .pi review --apply <patch-id>
```

## MCP Setup

PI Governance can run as a local stdio MCP server:

```bash
pi mcp-config codex --command "$(which pi)" --store /path/to/.pi --namespace default
pi mcp-doctor codex --command "$(which pi)" --store /path/to/.pi --namespace default
```

Use the printed configuration with your MCP-capable client. The server command is the local `pi` binary with `mcp-stdio` arguments.

## Core Workflow

1. Capture or propose a memory candidate.
2. Review the patch before it changes durable memory.
3. Apply, reject, or defer the patch.
4. Retrieve scoped context for the current agent task.
5. Export, import, or inspect the store when needed.

Common commands include:

```text
init
doctor
propose
capture
memory-worth
inbox
review
context
session
recall-xray
graph
quality memory
quality relationship
retrieve
export
import
inspect-record
list-patches
inspect-patch
mcp-config
mcp-doctor
mcp-stdio
```

## Works with MCP Clients

PI Governance runs as a local stdio MCP server. Any MCP-capable client that can launch a local command can connect to the `pi` binary.

Common setups include:

- Codex CLI
- OpenCode
- PI agent
- Claude Desktop or other MCP clients that support local stdio servers

## Relationship to pi-persistent-intelligence

`pi-governance-rs` and `pi-persistent-intelligence` are standalone implementations of the shared PI memory model.

Use `pi-governance-rs` when you want governed memory through CLI or MCP across multiple agents.

Use `pi-persistent-intelligence` when you want the lightweight native memory extension inside PI agent.

They can interoperate through the shared memory contract and compatible import/export formats.

## Security and Privacy

PI Governance is local-first by default. Stores live on disk at the path you choose, and MCP clients connect by launching your local `pi` command.

The governance model reduces memory-poisoning risk by making durable writes explicit, reviewable, and evidence-backed. Review imported stores and pending patches before applying them.

Avoid storing secrets, credentials, private keys, passwords, or high-risk personal data as durable memory.

## Documentation

- [Installation](docs/INSTALL.md)
- [MCP server sharing](docs/MCP_SERVER_SHARING.md)
- [pi-persistent-intelligence compatibility](docs/PI_PERSISTENT_INTELLIGENCE_COMPATIBILITY.md)
- [Wiki index](docs/WIKI_INDEX.md)
- [Security policy](SECURITY.md)
- [Changelog](CHANGELOG.md)

## License

Licensed under either MIT or Apache-2.0, at your option.
