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

## Portable Workflow Parity

`v1.0.0` adds deterministic portable memory workflow parity: `memory-worth`, `capture`, `inbox`, `context`, `session add/search/decisions`, `recall-xray`, explicit L1/L2/L3 layers, trust class, durability, source kind, and minimal verification gates. Capture creates candidates or L3 evidence only; it does not silently apply durable L1/L2 memory. L1 is never auto-applied. L3 is session/evidence context, not authoritative memory.
