# CLI Guide

Commands below use `pi --store .pi` examples; replace `.pi` with your local store.

## `init`

Purpose: init command for governed memory operation, inspection, setup, or validation.

Example:

```bash
pi --store .pi init
```

Expected output summary: human-readable or JSON output describing the requested operation, errors, or next review step.

Notes/cautions: review output before applying durable changes; use namespaces intentionally; redacted output is best-effort.

## `demo`

Purpose: demo command for governed memory operation, inspection, setup, or validation.

Example:

```bash
pi --store .pi demo
```

Expected output summary: human-readable or JSON output describing the requested operation, errors, or next review step.

Notes/cautions: review output before applying durable changes; use namespaces intentionally; redacted output is best-effort.

## `propose`

Purpose: propose command for governed memory operation, inspection, setup, or validation.

Example:

```bash
pi --store .pi propose --claim "Release validation requires tests." --evidence "release-checklist"
```

Expected output summary: human-readable or JSON output describing the requested operation, errors, or next review step.

Notes/cautions: review output before applying durable changes; use namespaces intentionally; redacted output is best-effort.

## `review`

Purpose: review command for governed memory operation, inspection, setup, or validation.

Example:

```bash
pi --store .pi review
```

Expected output summary: human-readable or JSON output describing the requested operation, errors, or next review step.

Notes/cautions: review output before applying durable changes; use namespaces intentionally; redacted output is best-effort.

## `review --apply`

Purpose: review   apply command for governed memory operation, inspection, setup, or validation.

Example:

```bash
pi --store .pi review <patch-id> --apply
```

Expected output summary: human-readable or JSON output describing the requested operation, errors, or next review step.

Notes/cautions: review output before applying durable changes; use namespaces intentionally; redacted output is best-effort.

## `review --reject`

Purpose: review   reject command for governed memory operation, inspection, setup, or validation.

Example:

```bash
pi --store .pi review <patch-id> --reject
```

Expected output summary: human-readable or JSON output describing the requested operation, errors, or next review step.

Notes/cautions: review output before applying durable changes; use namespaces intentionally; redacted output is best-effort.

## `review --defer`

Purpose: review   defer command for governed memory operation, inspection, setup, or validation.

Example:

```bash
pi --store .pi review <patch-id> --defer
```

Expected output summary: human-readable or JSON output describing the requested operation, errors, or next review step.

Notes/cautions: review output before applying durable changes; use namespaces intentionally; redacted output is best-effort.

## `apply`

Purpose: apply command for governed memory operation, inspection, setup, or validation.

Example:

```bash
pi --store .pi apply <patch-id>
```

Expected output summary: human-readable or JSON output describing the requested operation, errors, or next review step.

Notes/cautions: review output before applying durable changes; use namespaces intentionally; redacted output is best-effort.

## `list-patches`

Purpose: list patches command for governed memory operation, inspection, setup, or validation.

Example:

```bash
pi --store .pi list-patches
```

Expected output summary: human-readable or JSON output describing the requested operation, errors, or next review step.

Notes/cautions: review output before applying durable changes; use namespaces intentionally; redacted output is best-effort.

## `inspect-patch`

Purpose: inspect patch command for governed memory operation, inspection, setup, or validation.

Example:

```bash
pi --store .pi inspect-patch <patch-id>
```

Expected output summary: human-readable or JSON output describing the requested operation, errors, or next review step.

Notes/cautions: review output before applying durable changes; use namespaces intentionally; redacted output is best-effort.

## `list`

Purpose: list command for governed memory operation, inspection, setup, or validation.

Example:

```bash
pi --store .pi list
```

Expected output summary: human-readable or JSON output describing the requested operation, errors, or next review step.

Notes/cautions: review output before applying durable changes; use namespaces intentionally; redacted output is best-effort.

## `inspect-record`

Purpose: inspect record command for governed memory operation, inspection, setup, or validation.

Example:

```bash
pi --store .pi inspect-record <record-id>
```

Expected output summary: human-readable or JSON output describing the requested operation, errors, or next review step.

Notes/cautions: review output before applying durable changes; use namespaces intentionally; redacted output is best-effort.

## `retrieve`

Purpose: retrieve command for governed memory operation, inspection, setup, or validation.

Example:

```bash
pi --store .pi retrieve "release workflow" --retriever hybrid --explain
```

Expected output summary: human-readable or JSON output describing the requested operation, errors, or next review step.

Notes/cautions: review output before applying durable changes; use namespaces intentionally; redacted output is best-effort.

## `maintenance scan`

Purpose: maintenance scan command for governed memory operation, inspection, setup, or validation.

Example:

```bash
pi --store .pi maintenance scan
```

Expected output summary: human-readable or JSON output describing the requested operation, errors, or next review step.

Notes/cautions: review output before applying durable changes; use namespaces intentionally; redacted output is best-effort.

## `doctor`

Purpose: doctor command for governed memory operation, inspection, setup, or validation.

Example:

```bash
pi --store .pi doctor
```

Expected output summary: human-readable or JSON output describing the requested operation, errors, or next review step.

Notes/cautions: review output before applying durable changes; use namespaces intentionally; redacted output is best-effort.

## `reconcile`

Purpose: compare this independent canonical store with a peer `pi-governance` snapshot without importing or synchronizing data.

```bash
pi --store .pi --namespace default reconcile peer-bundle.json --json
pi --store .pi --namespace default reconcile peer-bundle.json --project my-project --json
```

The report covers records, patches, evidence, inquiries, sessions, reinforcement, generic events, and tombstones. It shows source-only, destination-only, matching, divergent, duplicate, and conflicting IDs. There is intentionally no apply option.

## `privacy-purge`

Permanently redacts secret-bearing record content while preserving the stable record ID, namespace, a tombstoned shell, and an external byte-for-byte backup. Preview is read-only; apply requires both the reviewed fingerprint and `--force`.

```bash
pi --store .pi --namespace default privacy-purge <record-id> --reason "privacy cleanup" --json
pi --store .pi --namespace default privacy-purge <record-id> --reason "privacy cleanup" --apply --force --fingerprint <preview-fingerprint> --json
```

The stored reason is represented by its SHA-256 digest. Previous claim, evidence, tags, and supersession links are excluded from the rewritten record and audit report.

## `store-integrity`

Purpose: preview duplicate stable-ID and self-supersession repair. The preview is read-only and emits the fingerprint required for apply.

```bash
pi --store .pi store-integrity --json
pi --store .pi store-integrity --apply --fingerprint <fingerprint-from-preview> --json
```

Apply refuses a stale fingerprint, creates a mandatory backup, atomically rewrites `records.jsonl`, and writes an audit report.

## `migrate`

Purpose: migrate legacy JSONL schema versions. The command is read-only unless `--apply` is supplied.

```bash
pi --store .pi migrate --json
pi --store .pi migrate --apply --json
```

Applied migrations always create a timestamped backup. Review preview output before applying durable changes.

## `namespace list`

Purpose: namespace list command for governed memory operation, inspection, setup, or validation.

Example:

```bash
pi --store .pi namespace list
```

Expected output summary: human-readable or JSON output describing the requested operation, errors, or next review step.

Notes/cautions: review output before applying durable changes; use namespaces intentionally; redacted output is best-effort.

## `namespace doctor`

Purpose: namespace doctor command for governed memory operation, inspection, setup, or validation.

Example:

```bash
pi --store .pi namespace doctor
```

Expected output summary: human-readable or JSON output describing the requested operation, errors, or next review step.

Notes/cautions: review output before applying durable changes; use namespaces intentionally; redacted output is best-effort.

## `config show`

Purpose: config show command for governed memory operation, inspection, setup, or validation.

Example:

```bash
pi --store .pi config show
```

Expected output summary: human-readable or JSON output describing the requested operation, errors, or next review step.

Notes/cautions: review output before applying durable changes; use namespaces intentionally; redacted output is best-effort.

## `config set-policy`

Purpose: config set policy command for governed memory operation, inspection, setup, or validation.

Example:

```bash
pi --store .pi config set-policy
```

Expected output summary: human-readable or JSON output describing the requested operation, errors, or next review step.

Notes/cautions: review output before applying durable changes; use namespaces intentionally; redacted output is best-effort.

## `policy doctor`

Purpose: policy doctor command for governed memory operation, inspection, setup, or validation.

Example:

```bash
pi --store .pi policy doctor
```

Expected output summary: human-readable or JSON output describing the requested operation, errors, or next review step.

Notes/cautions: review output before applying durable changes; use namespaces intentionally; redacted output is best-effort.

## `policy explain`

Purpose: policy explain command for governed memory operation, inspection, setup, or validation.

Example:

```bash
pi --store .pi policy explain
```

Expected output summary: human-readable or JSON output describing the requested operation, errors, or next review step.

Notes/cautions: review output before applying durable changes; use namespaces intentionally; redacted output is best-effort.

## `export`

Purpose: export command for governed memory operation, inspection, setup, or validation.

Example:

```bash
pi --store .pi export --redacted --output pi-export.redacted.json
```

Expected output summary: human-readable or JSON output describing the requested operation, errors, or next review step.

Notes/cautions: review output before applying durable changes; use namespaces intentionally; redacted output is best-effort.

## `import`

Purpose: import command for governed memory operation, inspection, setup, or validation.

Example:

```bash
pi --store .pi import pi-export.redacted.json
```

Expected output summary: human-readable or JSON output describing the requested operation, errors, or next review step.

Notes/cautions: review output before applying durable changes; use namespaces intentionally; redacted output is best-effort.

## `mcp-config`

Purpose: mcp config command for governed memory operation, inspection, setup, or validation.

Example:

```bash
pi mcp-config opencode --command /path/to/pi --store /path/to/.pi --namespace default
```

Expected output summary: human-readable or JSON output describing the requested operation, errors, or next review step.

Notes/cautions: review output before applying durable changes; use namespaces intentionally; redacted output is best-effort.

## `mcp-install`

Purpose: mcp install command for governed memory operation, inspection, setup, or validation.

Example:

```bash
pi mcp-install opencode --command /path/to/pi --store /path/to/.pi --namespace default
```

Expected output summary: human-readable or JSON output describing the requested operation, errors, or next review step.

Notes/cautions: review output before applying durable changes; use namespaces intentionally; redacted output is best-effort.

## `mcp-doctor`

Purpose: mcp doctor command for governed memory operation, inspection, setup, or validation.

Example:

```bash
pi mcp-doctor opencode --command /path/to/pi --store /path/to/.pi --namespace default
```

Expected output summary: human-readable or JSON output describing the requested operation, errors, or next review step.

Notes/cautions: review output before applying durable changes; use namespaces intentionally; redacted output is best-effort.

## `smoke-test`

Purpose: smoke test command for governed memory operation, inspection, setup, or validation.

Example:

```bash
pi --store .pi smoke-test
```

Expected output summary: human-readable or JSON output describing the requested operation, errors, or next review step.

Notes/cautions: review output before applying durable changes; use namespaces intentionally; redacted output is best-effort.

## `release-audit`

Purpose: release audit command for governed memory operation, inspection, setup, or validation.

Example:

```bash
pi --store .pi release-audit
```

Expected output summary: human-readable or JSON output describing the requested operation, errors, or next review step.

Notes/cautions: review output before applying durable changes; use namespaces intentionally; redacted output is best-effort.

## `changelog`

Purpose: changelog command for governed memory operation, inspection, setup, or validation.

Example:

```bash
pi --store .pi changelog
```

Expected output summary: human-readable or JSON output describing the requested operation, errors, or next review step.

Notes/cautions: review output before applying durable changes; use namespaces intentionally; redacted output is best-effort.

## `agent-instructions`

Purpose: agent instructions command for governed memory operation, inspection, setup, or validation.

Example:

```bash
pi --store .pi agent-instructions
```

Expected output summary: human-readable or JSON output describing the requested operation, errors, or next review step.

Notes/cautions: review output before applying durable changes; use namespaces intentionally; redacted output is best-effort.


---

Related: [Wiki index](../WIKI_INDEX.md), [Deployment checklist](../DEPLOYMENT_CHECKLIST.md), [Release strategy](../RELEASE_STRATEGY.md), [Stable v1 gate](../STABLE_V1_GATE.md).

## Graph and quality reports

```console
pi --store .pi graph --json
pi --store .pi quality memory --json
pi --store .pi quality relationship --json
pi --store .pi quality recall --json
pi --store .pi quality store --json
pi --store .pi config set-recall-telemetry on --max-events 1000
pi --store .pi recall-feedback corrected <record-id>
pi --store .pi simulate-patch <patch-id> --json
pi --store .pi procedure-candidates --min-source-records 2 --json
pi --store .pi failure-analysis --stale-days 30 --json
```

These commands compute read-only, namespace-scoped reports from canonical JSONL data. `graph` accepts `--max-nodes` and `--max-edges`. Quality scores are versioned heuristics accompanied by concrete signals; they do not mutate records or persist derived graph state.

## Portable Workflow Parity

`v1.0.0` adds deterministic portable memory workflow parity: `memory-worth`, `capture`, `inbox`, `context`, `session add/search/decisions`, `recall-xray`, explicit L1/L2/L3 layers, trust class, durability, source kind, and minimal verification gates. Capture creates candidates or L3 evidence only; it does not silently apply durable L1/L2 memory. L1 is never auto-applied. L3 is session/evidence context, not authoritative memory.


## Distribution and MCP Sharing

Repository: https://github.com/Mont3ll/pi-governance-rs
License: MIT OR Apache-2.0

Install from source with `cargo build -p pi-governance-rs`, from Git with `cargo install --git https://github.com/Mont3ll/pi-governance-rs --tag v1.1.0 pi-governance-rs`, or from crates.io with `cargo install pi-governance-rs`. `pi-governance-rs` is a standalone local stdio MCP server by default; it does not provide a hosted service. It remains compatible with `pi-persistent-intelligence` through the shared PI memory contract.
