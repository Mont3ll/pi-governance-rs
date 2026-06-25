# Export, Import, and Redaction

```bash
pi --store .pi export --output pi-export.json
pi --store .pi export --redacted --output pi-export.redacted.json
pi --store .pi export --all-namespaces --redacted --output pi-export.all.redacted.json
pi --store .pi import pi-export.json --dry-run
pi --store .pi import pi-export.json --backup
```

Redacted export includes metadata indicating redaction was requested. PI is not a secret scanner or DLP system. Do not store secrets in PI. Redacted export is best-effort and must be reviewed before sharing.


---

Related: [Wiki index](../WIKI_INDEX.md), [Deployment checklist](../DEPLOYMENT_CHECKLIST.md), [Release strategy](../RELEASE_STRATEGY.md), [Stable v1 gate](../STABLE_V1_GATE.md).
