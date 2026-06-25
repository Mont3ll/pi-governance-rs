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
