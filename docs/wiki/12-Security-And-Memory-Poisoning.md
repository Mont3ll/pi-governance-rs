# Security and Memory Poisoning

Do not store secrets in PI. Memory poisoning risks include false claims, stale corrections, malicious instructions, weak evidence, and namespace confusion. PI mitigates these with patch-before-mutation, manual review, contest and resolve workflows, tombstones instead of hard delete, namespace isolation, and auditable history. Redacted export is limited and best-effort.

See [SECURITY.md](../../SECURITY.md) and [docs/MEMORY_POISONING.md](../MEMORY_POISONING.md).


---

Related: [Wiki index](../WIKI_INDEX.md), [Deployment checklist](../DEPLOYMENT_CHECKLIST.md), [Release strategy](../RELEASE_STRATEGY.md), [Stable v1 gate](../STABLE_V1_GATE.md).
