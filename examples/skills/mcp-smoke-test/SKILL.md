---
name: mcp-smoke-test
description: Validate PI MCP stdio behavior before release or adapter changes.
class: workflow
domain: software-engineering
project: pi-governance-rs
tags:
  - mcp
  - smoke-test
  - adapter
version: 1.0.0-rc.3
confidence: high
policy: standard
---

# MCP Smoke Test

## When to Use

Use this after changing MCP configuration, stdio behavior, or tool schemas.

## Prerequisites

- `cargo build -p pi-governance-rs` has completed.
- A PI store exists or a temporary store is selected.

## Workflow

1. Generate client config with `pi mcp-config claude`.
2. Run `tools/list` through `pi mcp-stdio`.
3. Confirm core tools are present.
4. Call `pi.smoke_test` with JSON arguments.
5. If a call fails, inspect `tools/list` before debugging tool calls.

## Verification

Expected:

- MCP config includes `mcp-stdio` and `--store`.
- `tools/list` includes the previous core tools.
- `pi.smoke_test` returns `isError: false` and result `pass`.

## Common Failure Modes

- The binary path points to an old build.
- JSON-RPC payload is malformed.
- Tool names changed accidentally.

## Related PI Usage

```bash
pi propose --class workflow \
  --claim "Before changing the MCP adapter, verify tools/list and pi.smoke_test over stdio." \
  --project pi-governance-rs \
  --tag mcp \
  --evidence-uri examples:mcp-smoke-test
```
