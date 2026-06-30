---
name: rust-release-check
description: Validate a Rust PI release before tagging.
class: workflow
domain: software-engineering
project: pi-governance-rs
tags:
  - rust
  - release
  - cargo
  - audit
version: 1.0.0-rc.3
confidence: high
policy: standard
---

# Rust Release Check

## When to Use

Use this before tagging a PI Rust release.

## Prerequisites

- Working tree is clean.
- Previous version tag exists.
- Target version has been bumped.

## Workflow

1. Run `cargo check --workspace`.
2. Run `cargo test --workspace`.
3. Run `cargo build -p pi-governance-rs`.
4. Run `./target/debug/pi --version`.
5. Run `./target/debug/pi smoke-test`.
6. Run `./target/debug/pi release-audit`.

## Verification

Expected:

- Cargo check passes.
- Cargo test passes.
- Version command returns the target version.
- Smoke test passes.
- Release audit passes.

## Common Failure Modes

- Version was bumped in one crate but not another.
- Changelog does not contain the new release.
- JSON diagnostic output is invalid.
- Working tree includes generated artifacts.

## Related PI Usage

Store this as governed workflow memory using:

```bash
pi propose --class workflow \
  --claim "Before tagging a Rust PI release, run cargo check, cargo test, build, version, smoke-test, and release-audit." \
  --project pi-governance-rs \
  --tag release \
  --evidence-uri examples:rust-release-check \
  --apply
```
