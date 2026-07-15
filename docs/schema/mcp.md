# MCP schema

PI MCP uses JSON-RPC over stdio. `tools/list` is the canonical tool registry; release audit validates required tools against that registry.

Tool results include text content and machine-readable `structuredContent`. Graph and quality reports include `schema_version`, `generated_at`, namespace scope, recommendations or warnings where applicable, and `mutation_performed: false`.

Operational inspection tools:

- `pi.memory_graph` accepts optional `namespace`, `max_nodes`, and `max_edges`.
- `pi.memory_quality` accepts optional `namespace`.
- `pi.relationship_quality` accepts optional `namespace`, `max_nodes`, and `max_edges`.
- `pi.recall_effectiveness` accepts optional `namespace`.
- `pi.store_quality` accepts optional `namespace`.
- `pi.simulate_patch` requires `patch_id` and returns predicted state and quality deltas with `mutation_performed: false`.
- `pi.procedure_candidates` accepts optional `namespace` and `min_source_records`.
- `pi.failure_analysis` accepts optional `namespace` and `stale_days`.

The graph is a bounded computed report. MCP defaults return at most 200 graph nodes, 400 graph edges, 100 memory/relationship items, and 50 recall items; clients may request larger bounded limits. Non-explain retrieval omits full ranked-record metadata and returns compact context blocks. Evidence node identifiers do not contain evidence URIs. Quality metric versions are separate from the persisted store schema version.
