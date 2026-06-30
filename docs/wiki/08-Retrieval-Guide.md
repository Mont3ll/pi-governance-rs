# Retrieval Guide

PI rc.8 supports deterministic, lexical, and hybrid local retrievers. No embeddings are used in rc.8. No vector database is required. Retrieval remains local and deterministic.

```bash
pi --store .pi retrieve "release workflow" --retriever deterministic --explain
pi --store .pi retrieve "release workflow" --retriever lexical --explain
pi --store .pi retrieve "release workflow" --retriever hybrid --explain
```

`--explain` reports diagnostics such as matched terms, matched fields, score components, empty-result explanations, and budget packing. Use namespace, project, status, `include-contested`, and `min-confidence` filters to narrow context.


---

Related: [Wiki index](../WIKI_INDEX.md), [Deployment checklist](../DEPLOYMENT_CHECKLIST.md), [Release strategy](../RELEASE_STRATEGY.md), [Stable v1 gate](../STABLE_V1_GATE.md).

## Portable Workflow Parity

`v1.0.0` adds deterministic portable memory workflow parity: `memory-worth`, `capture`, `inbox`, `context`, `session add/search/decisions`, `recall-xray`, explicit L1/L2/L3 layers, trust class, durability, source kind, and minimal verification gates. Capture creates candidates or L3 evidence only; it does not silently apply durable L1/L2 memory. L1 is never auto-applied. L3 is session/evidence context, not authoritative memory.
