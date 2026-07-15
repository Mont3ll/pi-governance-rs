# pi-persistent-intelligence Compatibility

`pi-governance-rs` and `pi-persistent-intelligence` are standalone implementations of the shared PI governed-memory model.

Use `pi-governance-rs` for explicit CLI or MCP memory workflows across agent clients. Use `pi-persistent-intelligence` for native PI-agent hooks and interactive extension commands. Neither project requires the other.

## Shared model

Both implementations support scoped records, confidence and evidence metadata, review before durable changes, lifecycle revision, and portable export/import.

## Operational inspection mapping

| Capability | Rust CLI | Rust MCP |
|---|---|---|
| Retrieve context | `pi retrieve` / `pi context` | `pi.retrieve_context` / `pi.build_context` |
| Review pending memory | `pi review` / `pi inbox` | `pi.list_patches`, `pi.inspect_patch`, `pi.list_inbox` |
| Explain recall | `pi recall-xray` | `pi.recall_xray` |
| Computed memory graph | `pi graph` | `pi.memory_graph` |
| Per-record quality | `pi quality memory` | `pi.memory_quality` |
| Relationship quality | `pi quality relationship` | `pi.relationship_quality` |
| Recall effectiveness | `pi quality recall` | `pi.recall_effectiveness` |
| Store quality | `pi quality store` | `pi.store_quality` |

Graph and quality outputs are deterministic, report-only views over canonical JSONL state. They do not create a graph database, persist derived edges, or mutate governed memory. Quality scores are versioned review heuristics; inspect their signals and reasons rather than treating scores as objective truth.

## Recall telemetry

Recall telemetry is disabled by default. When enabled, it is local, bounded by `max_events`, stores a SHA-256 query hash rather than raw query text, and is excluded from ordinary store export. Use `pi config set-recall-telemetry on --max-events 10000` to enable it.

## Intentional differences

Rust does not reproduce PI-extension lifecycle hooks, automatic context injection, TUI browsers, or automatic session shutdown behavior. Agent clients call Rust tools explicitly. Rust graph output is bounded and uses non-secret evidence-reference identifiers rather than exposing evidence URIs in graph node IDs.

## Interoperability and safety

Use export/import to move reviewed memory between stores. Review imported records and patches before relying on them. Keep identity-level claims, corrections, and high-impact rules under explicit review.
