# Governance Workflows

## Standard Patch Flow

`propose → inspect → review → apply` keeps durable writes explicit. Inspect the patch, verify evidence, then apply only if the claim belongs in memory.

## Reject and Defer

`propose → reject` records that a claim should not be applied. `propose → defer` keeps the patch out of durable memory until more information exists.

## Revision Workflows

- `reinforce`: add evidence and confidence to an active record.
- `supersede`: replace an active record with a newer claim.
- `tombstone`: retire a record without hard delete.
- `contest`: mark a record disputed with evidence.
- `resolve-contest`: uphold, tombstone, or supersede a contested record.

## Policy Profiles

Strict policy requires more manual review; standard policy is the normal default; permissive policy reduces friction for low-risk local workflows. Destructive changes are avoided because auditability is more important than making memory disappear.


---

Related: [Wiki index](../WIKI_INDEX.md), [Deployment checklist](../DEPLOYMENT_CHECKLIST.md), [Release strategy](../RELEASE_STRATEGY.md), [Stable v1 gate](../STABLE_V1_GATE.md).
