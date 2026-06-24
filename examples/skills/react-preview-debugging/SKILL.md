---
name: react-preview-debugging
description: Capture a governed workflow for debugging React preview issues with PI memory.
class: workflow
domain: software-engineering
project: generic
tags:
  - react
  - preview
  - debugging
version: 1.0.0-rc.3
confidence: medium
policy: standard
---

# React Preview Debugging

## When to Use

Use this when a React UI preview differs from expected behavior and the fix may become a durable project convention.

## Prerequisites

- The app can run locally.
- The failing route or component is known.
- Screenshots, logs, or reproduction steps are available.

## Workflow

1. Reproduce the preview issue.
2. Inspect console and network errors.
3. Check component props and routing state.
4. Make the smallest fix.
5. If the user provides a durable correction, propose it to PI.
6. Review the proposed memory before relying on it later.

## Verification

Expected:

- Preview renders the intended state.
- The fix is scoped to the issue.
- Any durable lesson is stored as governed memory, not silently captured.

## Common Failure Modes

- Storing a one-time UI preference as a global rule.
- Confusing temporary debugging notes with durable corrections.
- Not contesting stale UI conventions after a redesign.

## Related PI Usage

```bash
pi propose --class correction \
  --claim "For this app, preview debugging should verify route state before changing shared components." \
  --project my-react-app \
  --tag preview \
  --evidence-uri examples:react-preview-debugging
```
