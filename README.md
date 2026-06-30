# PI Governance

Local-first governed memory for AI agents.

- **Current:** `v1.0.0-rc.9`
- **Status:** stable-release candidate
- **Runtime:** Rust CLI + MCP stdio server
- **Store:** local JSONL source of truth

## Why PI Exists

Agents should not silently rewrite what they believe. Durable memory changes should be proposed, reviewed, audited, revised, contested, and reversible. PI Governance gives local AI-agent memory a governance layer: every durable belief starts as a patch, carries evidence, and can be inspected before it changes the source of truth.

## What PI Is

PI is a local-first governed memory runtime for coding agents and MCP clients. It provides:

- patch-before-mutation memory updates
- evidence references for claims
- namespace isolation for projects, clients, or workflows
- policy profiles for stricter or more permissive review
- MCP integration over stdio
- audit, smoke-test, maintenance, schema, and release tooling
- local deterministic, lexical, and hybrid retrieval over the JSONL store

## What PI Is Not

PI is not a:

- vector database
- GraphRAG engine
- codebase indexer
- agent framework
- hosted memory service
- secret vault
- DLP system
- dashboard product

## Current Release Candidate

`v1.0.0-rc.9` is the current stable-release candidate. It adds portable PI memory workflow parity on top of the rc.8 governance-runtime hardening:

- deterministic `memory-worth` scoring
- `capture` for correction/preference candidates and manual memory-write equivalents
- `inbox` candidate workflow over the governed review queue
- `context` scoped injection output for non-PI agents
- local `session add/search/decisions` for append-only L3 evidence
- `recall-xray` inclusion/exclusion diagnostics
- explicit L1/L2/L3 layers, memory kind, rule type, trust class, durability, and source kind
- MCP tools for score/capture/context/session/recall workflows
- rc.8 capabilities: MCP `inspect_record`, review actions, maintenance scan, local retrieval, redacted export metadata, schemas, and OpenCode/Codex/PI agent interoperability

Stable `v1.0.0` has not shipped yet. Capture creates candidates or L3 evidence; it does not silently apply durable L1/L2 memory.

## Quick Start

```bash
git clone <repository-url> pi-governance-rs
cd pi-governance-rs
cargo build -p pi-cli
./target/debug/pi --version
./target/debug/pi demo --store /tmp/pi-demo-store --reset
./target/debug/pi --store /tmp/pi-demo-store review
./target/debug/pi --store /tmp/pi-demo-store retrieve "release workflow" --retriever hybrid --explain
./target/debug/pi --store /tmp/pi-demo-store doctor
```

Expected version for this release candidate:

```text
pi 1.0.0-rc.9
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

# Maintenance, audit, and export
pi --store .pi maintenance scan
pi --store .pi doctor
pi --store .pi export --redacted --output pi-export.redacted.json

# MCP onboarding
pi mcp-config opencode --command /path/to/pi --store /path/to/.pi --namespace default
pi mcp-install opencode --command /path/to/pi --store /path/to/.pi --namespace default --dry-run
pi mcp-install opencode --command /path/to/pi --store /path/to/.pi --namespace default --yes
pi mcp-doctor opencode --command /path/to/pi --store /path/to/.pi --namespace default
```

### Command Matrix

The rc.9 CLI includes `init`, `doctor`, `migrate`, `config`, `policy`, `namespace`, `propose`, `review`, `inbox`, `capture`, `memory-worth`, `context`, `session`, `recall-xray`, `demo`, `agent-instructions`, `apply`, `reinforce`, `supersede`, `tombstone`, `contest`, `resolve-contest`, `retrieve`, `export`, `import`, `list`, `inspect-record`, `list-patches`, `inspect-patch`, `mcp-stdio`, `mcp-config`, `mcp-install`, `mcp-doctor`, `smoke-test`, `release-audit`, and `changelog`.

See [docs/wiki/04-CLI-Guide.md](docs/wiki/04-CLI-Guide.md) for the full command guide.

## MCP Setup

PI runs as an MCP stdio server and can generate client configuration for OpenCode, Codex CLI, and PI agent/shared MCP setups.

```bash
pi mcp-config opencode --command /path/to/pi --store /path/to/.pi --namespace default
pi mcp-install opencode --command /path/to/pi --store /path/to/.pi --namespace default --dry-run
pi mcp-install opencode --command /path/to/pi --store /path/to/.pi --namespace default --yes
pi mcp-doctor opencode --command /path/to/pi --store /path/to/.pi --namespace default
```

Restart the MCP client after installation. Clients may expose prefixed tool names such as `pi.retrieve_context`, `pi-governance_pi_retrieve_context`, `pi_governance_pi.retrieve_context`, or `mcp__pi_governance__.pi_retrieve_context`.

## Core Concepts

- **Store:** local JSONL source of truth, normally `.pi`.
- **Record:** governed memory claim with class, confidence, evidence, namespace, and status.
- **Patch:** proposed durable mutation awaiting review or application.
- **Evidence:** reference explaining why a claim or change exists.
- **Namespace:** isolation boundary for projects or clients.
- **Policy profile:** review strictness and validation behavior.
- **MCP server:** stdio interface exposing PI tools to clients.
- **Maintenance scan:** local health scan for stale patches, contested records, weak evidence, duplicates, and summaries.
- **Retriever mode:** deterministic, lexical, or hybrid local retrieval.

## Safety Model

PI uses patch-before-mutation. Durable memory is not silently rewritten: proposed changes can be applied, rejected, deferred, contested, resolved, superseded, or tombstoned. Destructive hard delete is avoided in favor of auditable tombstones. Namespace isolation keeps stores and clients scoped. Redacted export is best-effort and must be reviewed before sharing; PI is not a secret scanner or DLP system.

## Interoperability Status

| Client | rc.8 Status |
| --- | --- |
| OpenCode | pass |
| Codex CLI | pass |
| PI agent | pass |

Tested capabilities include `retrieve_context`, `propose_record`, `list_patches`, `inspect_patch`, `inspect_record`, `list_records`, `maintenance_scan`, `doctor`, `smoke_test`, namespace propagation, and structuredContent compatibility.

## Documentation

Start with [docs/WIKI_INDEX.md](docs/WIKI_INDEX.md). Key pages:

- [Installation](docs/wiki/03-Installation.md)
- [CLI Guide](docs/wiki/04-CLI-Guide.md)
- [MCP Setup](docs/wiki/05-MCP-Setup.md)
- [Agent Interoperability](docs/wiki/06-Agent-Interoperability.md)
- [Release and Deployment](docs/wiki/13-Release-And-Deployment.md)
- [QA and Test Matrix](docs/wiki/14-QA-And-Test-Matrix.md)
- [Deployment Checklist](docs/DEPLOYMENT_CHECKLIST.md)
- [Release Strategy](docs/RELEASE_STRATEGY.md)
- [Stable v1 Gate](docs/STABLE_V1_GATE.md)

## Deferred / Not in rc.9

Moved into rc.9: deterministic correction capture, manual `memory_write` equivalent through `pi capture`, memory-worth scoring, inbox workflow, scoped context injection, session search/decisions, minimal trust/durability/source gates, recall X-ray, L1/L2/L3 layers, and minimal reinforcement event compatibility.

Still deferred:

- deep reinforcement maintenance weighting
- LLM consolidation
- qmd semantic search
- vault integration / Obsidian mutation
- background job queue
- memory graph or timeline
- procedure candidates and skill draft artifacts
- meta-consolidation automation
- dashboard/TUI
- hosted MCP endpoint
- connectors
- vector backend
- graph backend
- team RBAC/SSO
- cloud sync

## Release Strategy

`v1.0.0-rc.9` is the stable-release candidate. No new features should be added before stable unless a blocker appears. Stable release requires the final checklist in [docs/STABLE_V1_GATE.md](docs/STABLE_V1_GATE.md) and [docs/DEPLOYMENT_CHECKLIST.md](docs/DEPLOYMENT_CHECKLIST.md).

## License

No license file is currently present in this repository. A license file should be added before public release.
