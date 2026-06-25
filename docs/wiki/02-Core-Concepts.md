# Core Concepts

- **Store:** local JSONL source of truth, usually `.pi`.
- **Record:** governed memory claim with class, claim, confidence, evidence, namespace, project fields, and status.
- **Patch:** proposed mutation. Patch statuses include proposed/pending, applied, rejected, and deferred states as exposed by CLI/MCP.
- **Evidence:** reference that explains why the claim or patch exists.
- **Namespace:** isolation scope for records, patches, retrieval, MCP clients, and tests.
- **Policy profile:** standard, strict, or permissive governance behavior for review requirements.
- **Record statuses:** active, contested, superseded, and tombstoned. Tombstones preserve history rather than hard-deleting claims.
- **Retriever modes:** deterministic, lexical, and hybrid. rc.8 uses no embeddings and requires no vector database.
- **Maintenance findings:** scan results such as pending patches, contested records, low-confidence records, missing evidence, duplicates, namespace summary, and policy summary.
- **Redaction metadata:** metadata attached to redacted exports so reviewers know redaction was best-effort.
- **MCP structuredContent:** object-shaped responses intended for client compatibility.


---

Related: [Wiki index](../WIKI_INDEX.md), [Deployment checklist](../DEPLOYMENT_CHECKLIST.md), [Release strategy](../RELEASE_STRATEGY.md), [Stable v1 gate](../STABLE_V1_GATE.md).
