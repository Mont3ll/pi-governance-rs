# Non-Goals

PI is not:

- a vector database
- a graph database
- a codebase indexer
- a full agent framework
- a hosted cloud memory service
- a secrets manager
- a truth oracle
- a replacement for human review
- a replacement for codebase-memory MCP servers
- a replacement for skill libraries

## Complement Model

Use codebase intelligence tools to understand repository structure. Use skill libraries to guide task procedures. Use PI to govern durable memory, decisions, corrections, workflows, and stale-memory revisions.

## Deferred Until After Public Testing

- capture/extraction
- FTS/BM25 retrieval
- vector retrieval
- graph/capsule memory
- dashboard
- connectors
- hosted service
- pi-persistent-intelligence integration package

These are intentionally out of scope for the v1.0.0 release-candidate line.

## an earlier release candidate Deferred Feature Classification

Release-quality improvements now included in an earlier release candidate: MCP record inspection, review queue actions, minimal maintenance scan, local lexical/hybrid retrieval hardening, redacted export hardening, and schema documentation.

Post-stable quality improvements: `pi capture --stdin`, FTS/BM25 persistent index if still needed, memory capsules, relationship edges, maintenance auto-suggestions as governed patches, and expanded machine-readable schemas.

Product expansion remains deferred: dashboard/TUI, hosted MCP endpoint, connectors, vector backend, graph backend, team RBAC/SSO, central audit logs, and cloud sync.

## Release Documentation Links

- [Wiki index](docs/WIKI_INDEX.md)
- [Deployment checklist](docs/DEPLOYMENT_CHECKLIST.md)
- [Release strategy](docs/RELEASE_STRATEGY.md)
- [Stable v1 gate](docs/STABLE_V1_GATE.md)
- [Release and deployment wiki](docs/wiki/13-Release-And-Deployment.md)
- [QA and test matrix](docs/wiki/14-QA-And-Test-Matrix.md)
