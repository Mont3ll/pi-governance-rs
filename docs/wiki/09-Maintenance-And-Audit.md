# Maintenance and Audit

Use maintenance and audit commands before releases and when diagnosing stores.

```bash
pi --store .pi maintenance scan
pi --store .pi doctor
pi --store .pi namespace doctor
pi --store .pi policy doctor
pi smoke-test
pi release-audit
```

Maintenance findings include `pending_patches`, `old_pending_patches`, `deferred_patches`, `rejected_patches`, `contested_records`, `superseded_records`, `tombstoned_records`, `records_without_evidence`, `low_confidence_records`, `records_with_empty_claims`, `possible_duplicate_claims`, `namespace_summary`, and `policy_summary`.


---

Related: [Wiki index](../WIKI_INDEX.md), [Deployment checklist](../DEPLOYMENT_CHECKLIST.md), [Release strategy](../RELEASE_STRATEGY.md), [Stable v1 gate](../STABLE_V1_GATE.md).
