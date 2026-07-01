# Changelog

## v1.0.2

Packaging-only release preparing crates.io distribution.

Changed:
- Renamed public Cargo package identity from pi-cli to pi-governance-rs.
- Namespaced internal crates under pi-governance-*.
- Kept installed binary name as pi.
- Updated installation and publishing docs.

No runtime governance behavior changed.
No MCP tools changed.
No CLI commands changed.

- v1.0.0 — stable portable PI memory governance release
  - Adds public repository, dual-license, packaging, install, MCP sharing, GitHub release planning, and MCP Registry preparation documentation.
  - Promotes v1.0.0-rc.9 to stable.
  - Provides standalone CLI/MCP PI memory governance for MCP-capable agents.
  - Includes capture, memory-worth scoring, inbox, L1/L2/L3 layers, trust/durability/source metadata, verification gates, context builder, session search/decisions, recall-xray, maintenance scan, import/export, and redacted export metadata.
  - Validated against PI agent and Codex CLI live MCP clients; OpenCode install/doctor passed, while the rc.9 live client run was incomplete due environmental/client-run limitation and did not demonstrate a PI Governance MCP failure.
  - Documents compatibility with pi-persistent-intelligence through the shared PI memory contract.
  - Adds no hosted service, database, vector store, graph backend, dashboard, or cloud sync.
- v1.0.0-rc.9 — portable PI memory workflow parity
  - Adds explicit L1/L2/L3 memory layers, memory kind, rule type, trust class, durability, and source kind metadata.
  - Adds deterministic `memory-worth` scoring.
  - Adds `capture` for correction/preference candidates and manual daily/long-term memory-write equivalents.
  - Adds `inbox`, `context`, `session add/search/decisions`, and `recall-xray` CLI workflows.
  - Adds MCP `pi.score_memory_worth`, `pi.capture_candidates`, `pi.build_context`, `pi.session_add`, `pi.session_search`, `pi.session_decisions`, and `pi.recall_xray`.
  - Keeps durable L1/L2 memory patch-governed; capture never silently applies durable memory.
- v1.0.0-rc.8 — release-quality governance, review ergonomics, and retrieval hardening
  - Adds MCP `pi.inspect_record` parity.
  - Adds review queue actions: `review --apply`, `review --reject`, and `review --defer`.
  - Adds read-only `maintenance scan` and MCP `pi.maintenance_scan`.
  - Adds local deterministic lexical/hybrid retrieval modes without embeddings or external indexes.
  - Adds redacted export metadata.
  - Adds schema documentation.
  - Adds no governance semantic changes.
- v1.0.0-rc.7 — MCP response compatibility and namespace correctness
  - Fixes object-shaped `structuredContent` for list-style MCP tools.
  - Fixes/defaults MCP namespace propagation from `pi --namespace ... mcp-stdio`.
  - Documents client-prefixed MCP tool names used by some clients.
  - Adds regression coverage.
  - Adds no governance semantic changes.
- v1.0.0-rc.6 — MCP client onboarding and troubleshooting
  - Adds `pi mcp-config opencode`, `pi mcp-config codex`, and `pi mcp-config pi-agent`.
  - Adds `pi mcp-install opencode`, `pi mcp-install codex`, and `pi mcp-install pi-agent`.
  - Adds `pi mcp-doctor opencode`, `pi mcp-doctor codex`, and `pi mcp-doctor pi-agent`.
  - Adds MCP troubleshooting docs.
  - Updates public testing docs.
  - Updates README and product guide.
  - Adds no governance semantic changes.
- v1.0.0-rc.5 — public-testing readiness and record inspection
  - Adds `pi inspect-record`.
  - Adds `pi inspect-record --json`.
  - Adds public testing guide.
  - Adds non-goals documentation.
  - Adds GitHub issue templates.
  - Updates README for public testing.
  - Updates product guide for rc.5.
  - Adds no governance semantic changes.
- v1.0.0-rc.4 — final docs consistency pass
  - Cleans up duplicated README sections.
  - Cleans up stale release-candidate version references.
  - Verifies first-10-minutes commands and command matrix documentation.
  - Verifies MCP documentation against available tools.
  - Adds standalone product guide HTML under docs/.
  - Verifies release-audit and smoke-test output.
  - Adds no governance semantic changes.
- v1.0.0-rc.3 — OSS usability, governed skills, and coding-agent integration
  - Adds `pi review` for pending governed memory patches.
  - Adds `pi demo` for a safe local demo store.
  - Adds governed skill/workflow examples.
  - Adds agent instruction guidance.
  - Adds memory-poisoning documentation and security documentation.
  - Adds codebase-memory-mcp complement documentation.
  - Adds pi-persistent-intelligence compatibility notes.
  - Rewrites the README quickstart for first-10-minutes adoption.
  - Adds no governance semantic changes.
- v1.0.0-rc.2 — release-candidate soak and compatibility pass
  - Verifies fresh-user clean clone installation.
  - Verifies README and release documentation examples.
  - Verifies MCP client configuration and MCP smoke flows.
  - Verifies clean-store import/export portability.
  - Verifies namespace and policy behavior after fresh init.
  - Verifies JSON diagnostics and release-audit output.
  - Adds no new governance semantics.
- v1.0.0-rc.1 — first release candidate
  - Freezes the public CLI command-name surface for the release candidate.
  - Freezes MCP tool names for the release candidate.
  - Documents fresh clone verification and archive content verification.
  - Documents release checklist verification.
  - Adds no new governance semantics.
- v0.10.1 audit and release-candidate cleanup
- v0.10.0 release hardening and adapter polish
- v0.9.0 policy profiles and operating modes
- v0.8.0 namespace isolation
- v0.7.0 deterministic retrieval improvements
- v0.6.0 portable import/export
- v0.5.1 contested belief revision workflow
- v0.5.0 governed belief revision operations
- v0.4.0 schema migrations and tests
- v0.3.0 store locking and schema versioning
- v0.2.0 patch visibility and safer apply flow
- v0.1.0 initial Rust PI port

## Release Documentation Links

- [Wiki index](docs/WIKI_INDEX.md)
- [Deployment checklist](docs/DEPLOYMENT_CHECKLIST.md)
- [Release strategy](docs/RELEASE_STRATEGY.md)
- [Stable v1 gate](docs/STABLE_V1_GATE.md)
- [Release and deployment wiki](docs/wiki/13-Release-And-Deployment.md)
- [QA and test matrix](docs/wiki/14-QA-And-Test-Matrix.md)
