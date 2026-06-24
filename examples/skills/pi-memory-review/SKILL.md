---
name: pi-memory-review
description: Review governed PI memory patches before release or major coding work.
class: workflow
domain: software-engineering
project: pi-governance-rs
tags:
  - memory
  - governance
  - review
version: 1.0.0-rc.3
confidence: high
policy: standard
---

# PI Memory Review

## When to Use

Use this before release work, after a long coding session, or when an agent has proposed durable memory changes.

## Prerequisites

- A PI store is selected.
- Pending patches may exist.

## Workflow

1. Run `pi review`.
2. Inspect each pending patch with `pi review <patch_id>`.
3. Apply allowed patches with `pi review <patch_id> --apply`.
4. Apply manual-review patches only when appropriate using `--force`.
5. If a memory is stale, contest or supersede it rather than silently ignoring it.

## Verification

Expected:

- Pending memory changes are understood before application.
- High-impact memories receive manual review.
- Stale memories are contestable and auditable.

## Common Failure Modes

- Applying strict-policy patches without reading the reason.
- Letting old release instructions stay active.
- Treating contested memory as safe by default.

## Related PI Usage

```bash
pi review
pi review <patch_id>
pi review <patch_id> --apply --force
```
