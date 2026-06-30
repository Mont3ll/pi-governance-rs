# PI Governance

Local-first governed memory for AI agents.

- **Current:** `v1.0.0`
- **Status:** stable public release
- **Runtime:** Rust CLI + MCP stdio server
- **Purpose:** standalone portable PI memory governance for any MCP-capable agent
- **Store:** local JSONL source of truth

## What PI Governance Is

PI Governance is a standalone CLI + MCP governed memory runtime. It gives local AI-agent memory a governance layer: every durable belief starts as a patch, carries evidence, can be inspected, and can be applied, rejected, deferred, contested, superseded, or tombstoned.

It implements the shared PI memory contract with namespaces, projects, layers, memory kinds, rule types, trust classes, durability, source kinds, verification metadata, patch statuses, and record statuses.

## Who It Is For

- users of Codex
- users of Claude
- users of OpenCode
- users of Cursor
- users of PI agent
- users of multiple agentic coding harnesses
- users who want global memory governance without pi-agent

## Relationship to pi-persistent-intelligence

Both projects implement or map to the shared PI memory contract.

- `pi-governance-rs` is the standalone portable Rust CLI/MCP runtime.
- `pi-persistent-intelligence` is the standalone lightweight pi-agent-native extension.
- Either project can be used alone.
- Both can interoperate through export/import and shared schemas.
- `pi-persistent-intelligence` remains a supported standalone project and does not require Rust.
- `pi-governance-rs` does not require `pi-persistent-intelligence`.

Users who use multiple agents can use `pi-governance-rs` as the global governed memory runtime. Users who want pi-agent-native capture, curation, and recall UX can use `pi-persistent-intelligence`. Users may use both together through compatible PI memory bundles.

## Installation

### From source

```bash
git clone https://github.com/Mont3ll/pi-governance-rs
cd pi-governance-rs
cargo build -p pi-cli
./target/debug/pi --version
```

### From Git

```bash
cargo install --git https://github.com/Mont3ll/pi-governance-rs --tag v1.0.0 pi-cli
pi --version
```

### From crates.io

```bash
cargo install pi-cli
pi --version
```

Note: crates.io publishing may still be pending until explicitly published.

## Quick Start

```bash
./target/debug/pi --version
./target/debug/pi demo --store /tmp/pi-demo-store --reset
./target/debug/pi --store /tmp/pi-demo-store memory-worth "Always run release-audit before tagging."
./target/debug/pi --store /tmp/pi-demo-store capture --text "do not skip release-audit before tagging"
./target/debug/pi --store /tmp/pi-demo-store inbox
./target/debug/pi --store /tmp/pi-demo-store context "stable release"
./target/debug/pi --store /tmp/pi-demo-store recall-xray "stable release"
./target/debug/pi mcp-config opencode --command /path/to/pi --store /path/to/.pi --namespace default
./target/debug/pi mcp-install opencode --command /path/to/pi --store /path/to/.pi --namespace default --dry-run
./target/debug/pi mcp-doctor opencode --command /path/to/pi --store /path/to/.pi --namespace default
```

Expected version:

```text
pi 1.0.0
```

## CLI Usage

```bash
# Initialize, score, capture, and propose governed memory
pi --store .pi init
pi --store .pi memory-worth "Always run cargo test before tagging."
pi --store .pi capture --text "don't skip release-audit before tagging" --project pi-governance-rs
pi --store .pi inbox
pi --store .pi propose --class workflow --claim "Release validation requires tests." --evidence-uri "release-checklist"

# Review and apply/reject/defer patches
pi --store .pi review
pi --store .pi review <patch-id> --apply
pi --store .pi review <patch-id> --reject --reason "not accurate"
pi --store .pi review <patch-id> --defer --reason "needs more evidence"
pi --store .pi apply <patch-id>

# Inspect patches and records
pi --store .pi list-patches
pi --store .pi inspect-patch <patch-id>
pi --store .pi list
pi --store .pi inspect-record <record-id>

# Retrieve and inject local context
pi --store .pi retrieve "release workflow" --retriever hybrid --explain
pi --store .pi context "prepare stable release" --project pi-governance-rs --format markdown
pi --store .pi recall-xray "stable release" --project pi-governance-rs --json

# Append/search L3 session evidence
pi --store .pi session add --text "#decision keep JSONL as source of truth" --project pi-governance-rs
pi --store .pi session search "JSONL" --project pi-governance-rs
pi --store .pi session decisions --project pi-governance-rs

# Maintenance, audit, and export/import
pi --store .pi maintenance scan
pi --store .pi doctor
pi --store .pi export --redacted --output pi-export.redacted.json
pi --store .pi import pi-export.redacted.json --dry-run
```

### Command Matrix

The stable CLI includes `init`, `doctor`, `migrate`, `config`, `policy`, `namespace`, `propose`, `review`, `inbox`, `capture`, `memory-worth`, `context`, `session`, `recall-xray`, `demo`, `agent-instructions`, `apply`, `reinforce`, `supersede`, `tombstone`, `contest`, `resolve-contest`, `retrieve`, `export`, `import`, `list`, `inspect-record`, `list-patches`, `inspect-patch`, `mcp-stdio`, `mcp-config`, `mcp-install`, `mcp-doctor`, `smoke-test`, `release-audit`, and `changelog`.

See [docs/wiki/04-CLI-Guide.md](docs/wiki/04-CLI-Guide.md) for the full command guide.

## MCP Setup

PI Governance owns the global MCP server because it exists to make governed PI memory available to Codex, Claude, OpenCode, Cursor, PI agent, and other MCP-capable harnesses.

pi-governance-rs is a local-first MCP stdio server. The MCP client launches the pi binary as a subprocess. This keeps governed memory on the user's machine by default. No hosted MCP service is provided by default.

```bash
pi mcp-config opencode --command /path/to/pi --store /path/to/.pi --namespace default
pi mcp-install opencode --command /path/to/pi --store /path/to/.pi --namespace default --dry-run
pi mcp-install opencode --command /path/to/pi --store /path/to/.pi --namespace default --yes
pi mcp-doctor opencode --command /path/to/pi --store /path/to/.pi --namespace default
```

Restart the MCP client after installation. Clients may expose prefixed tool names such as `pi.retrieve_context`, `pi-governance_pi_retrieve_context`, `pi_governance_pi.retrieve_context`, or `mcp__pi_governance__.pi_retrieve_context`.

## Shared PI Memory Contract

PI Governance implements the shared PI memory contract:

| Field | Meaning |
| --- | --- |
| `namespace` | isolation boundary for clients, projects, or workflows |
| `project` | optional project scope |
| `layer` | `l1_identity`, `l2_playbook`, or `l3_session` |
| `memory_kind` | `fact`, `event`, `instruction`, or `task` |
| `rule_type` | workflow/preference/correction/architecture-style classification |
| `trust_class` | source trust boundary |
| `durability` | expected lifetime/scope of the memory |
| `source_kind` | capture/import/source origin |
| `verification` | deterministic checks and review requirements |
| patch statuses | `proposed`, `applied`, `rejected`, `deferred` |
| record statuses | `active`, `contested`, `superseded`, `tombstoned`, `deleted` |

Durable L1/L2 memory remains patch-governed. Capture creates candidates or L3/session evidence only. L1 is never auto-applied.

## Public Functionality

- capture
- memory-worth scoring
- inbox workflow
- L1/L2/L3 governed memory layers
- trust/durability/source metadata
- verification gates
- context builder
- session add/search/decisions
- recall-xray
- review queue actions
- maintenance scan
- local deterministic, lexical, and hybrid retrieval
- import/export
- redacted export
- MCP tools
- smoke-test and release-audit

## Interoperability Status

| Client | v1.0.0 status |
| --- | --- |
| PI agent | pass |
| Codex CLI | pass |
| OpenCode | install/doctor passed; live rc.9 client run incomplete due environmental/client-run limitation |

The OpenCode limitation did not demonstrate a PI Governance MCP failure. rc.8 previously passed OpenCode live interoperability. Current validation found no structuredContent regression, namespace regression, MCP schema regression, or direct MCP validation failure.

Tested capabilities include `score_memory_worth`, `capture_candidates`, `build_context`, `session_add`, `session_search`, `session_decisions`, `recall_xray`, `retrieve_context`, `propose_record`, `list_patches`, `inspect_patch`, `inspect_record`, `list_records`, `maintenance_scan`, `doctor`, `smoke_test`, namespace propagation, and structuredContent compatibility.

## Documentation

Start with [docs/WIKI_INDEX.md](docs/WIKI_INDEX.md). Key pages:

- [Install and packaging guide](docs/INSTALL.md)
- [Packaging](docs/PACKAGING.md)
- [MCP Server Sharing](docs/MCP_SERVER_SHARING.md)
- [MCP Registry Prep](docs/MCP_REGISTRY_PREP.md)
- [GitHub Release Plan](docs/GITHUB_RELEASE_PLAN.md)
- [Publishing Checklist](docs/PUBLISHING_CHECKLIST.md)
- [Installation](docs/wiki/03-Installation.md)
- [CLI Guide](docs/wiki/04-CLI-Guide.md)
- [MCP Setup](docs/wiki/05-MCP-Setup.md)
- [Agent Interoperability](docs/wiki/06-Agent-Interoperability.md)
- [Export, Import, and Redaction](docs/wiki/10-Export-Import-And-Redaction.md)
- [Schema Reference](docs/wiki/11-Schema-Reference.md)
- [Release and Deployment](docs/wiki/13-Release-And-Deployment.md)
- [QA and Test Matrix](docs/wiki/14-QA-And-Test-Matrix.md)
- [Deployment Checklist](docs/DEPLOYMENT_CHECKLIST.md)
- [Release Strategy](docs/RELEASE_STRATEGY.md)
- [Stable v1 Gate](docs/STABLE_V1_GATE.md)
- [pi-persistent-intelligence Compatibility](docs/PI_PERSISTENT_INTELLIGENCE_COMPATIBILITY.md)

## Non-goals

PI Governance does not add a hosted service, database, vector store, graph backend, dashboard, connector, cloud sync, or hosted MCP endpoint.

## Release Strategy

`v1.0.0` is the stable public release. Future changes should preserve the public CLI and MCP tool surfaces or provide compatibility aliases.

## License

Licensed under either of Apache License, Version 2.0 or MIT License at your option. See [LICENSE](LICENSE), [LICENSE-APACHE](LICENSE-APACHE), and [LICENSE-MIT](LICENSE-MIT).
