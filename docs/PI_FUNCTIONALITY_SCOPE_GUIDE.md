# PI Functionality Scope Guide

Current stable release: v1.0.0. Stable v1.0.0 has shipped.

## Release-quality improvements now included in an earlier release candidate

- MCP record inspection
- Review queue actions
- Minimal maintenance scan
- Local lexical/hybrid retrieval hardening
- Redacted export hardening
- Schema documentation

## Post-stable quality improvements

- `pi capture --stdin`
- FTS/BM25 persistent index if still needed
- Memory capsules
- Relationship edges
- Maintenance auto-suggestions as governed patches
- Machine-readable schemas if not completed

## Product expansion

- Dashboard / TUI
- Hosted MCP endpoint
- Connectors
- Vector backend
- Graph backend
- Team RBAC / SSO
- Central audit logs
- Cloud sync

## Release Documentation Links

- [Wiki index](docs/WIKI_INDEX.md)
- [Deployment checklist](docs/DEPLOYMENT_CHECKLIST.md)
- [Release strategy](docs/RELEASE_STRATEGY.md)
- [Stable v1 gate](docs/STABLE_V1_GATE.md)
- [Release and deployment wiki](docs/wiki/13-Release-And-Deployment.md)
- [QA and test matrix](docs/wiki/14-QA-And-Test-Matrix.md)

## Portable Workflow Parity

`v1.0.0` adds deterministic portable memory workflow parity: `memory-worth`, `capture`, `inbox`, `context`, `session add/search/decisions`, `recall-xray`, explicit L1/L2/L3 layers, trust class, durability, source kind, and minimal verification gates. Capture creates candidates or L3 evidence only; it does not silently apply durable L1/L2 memory. L1 is never auto-applied. L3 is session/evidence context, not authoritative memory.
