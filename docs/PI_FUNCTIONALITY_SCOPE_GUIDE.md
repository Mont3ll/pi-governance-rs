# PI Functionality Scope Guide

Current coordinated release: `v1.1.0`.

## Included

- Local JSONL governed memory with namespace-aware stable identities
- Reviewable proposal, reinforcement, contest, supersession, tombstone, and resolution workflows
- Deterministic, lexical, and hybrid retrieval modes
- MCP stdio server and CLI with explicit store identity diagnostics
- Portable JS/Rust import, export, redaction metadata, and report-only reconciliation
- Preview-first schema migration, store-integrity repair, and privacy purge with backups and reviewed fingerprints
- Read-only graph, memory quality, relationship quality, recall effectiveness, store quality, patch simulation, procedure-candidate, and failure-analysis reports
- Disabled-by-default bounded recall telemetry using query hashes rather than raw queries

## Explicit boundaries

- No hosted service or remote memory backend
- No automatic authority selection between independent peer stores
- No automatic application of durable L1/L2 memory
- No automatic skill writing or vault mutation
- No embeddings or vector database requirement

## Future work

Future releases may add optional backends or additional client integrations only when they preserve the same review, provenance, privacy, and local-first boundaries.
