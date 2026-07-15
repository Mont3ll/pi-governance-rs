# MCP schema

PI MCP uses JSON-RPC over stdio. `tools/list` is the canonical tool registry; release audit validates required tools against that registry.

Tool results include text content and machine-readable `structuredContent`. Graph and quality reports include `schema_version`, `generated_at`, namespace scope, recommendations or warnings where applicable, and `mutation_performed: false`.

Operational inspection tools:

- `pi.memory_graph` accepts optional `namespace`, `max_nodes`, and `max_edges`.
- `pi.memory_quality` accepts optional `namespace`.
- `pi.relationship_quality` accepts optional `namespace`, `max_nodes`, and `max_edges`.
- `pi.recall_effectiveness` accepts optional `namespace`.
- `pi.store_quality` accepts optional `namespace`.

The graph is a bounded computed report. Evidence node identifiers do not contain evidence URIs. Quality metric versions are separate from the persisted store schema version.
