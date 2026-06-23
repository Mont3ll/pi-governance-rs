# pi-governance-rs

Current milestone: `0.6.0`.

A Rust port of the PI governance layer for coding agents. The runtime exposes governed memory operations through a CLI and an MCP stdio server while keeping mutations patch-governed, inspectable, and auditable.

## Workspace layout

```text
crates/
  pi-core/         core schemas, policy rules, record/patch types
  pi-store/        JSONL store, locking, migrations, import/export
  pi-retrieval/    deterministic retrieval and context rendering
  pi-governance/   runtime engine over store/policy/retrieval
  pi-mcp/          MCP stdio adapter
  pi-cli/          command-line binary
```

## Core commands

```bash
cargo build -p pi-cli
./target/debug/pi --version
./target/debug/pi --store .pi doctor
```

Initialize a store:

```bash
./target/debug/pi --store .pi init
```

Propose and apply a governed record:

```bash
./target/debug/pi --store .pi propose \
  --class requirement \
  --claim "Governed memory updates must go through patches." \
  --project pi-governance-rs \
  --tag governance \
  --evidence-uri conversation:example \
  --apply
```

Retrieve context:

```bash
./target/debug/pi --store .pi retrieve \
  "governed memory update requirements" \
  --project pi-governance-rs \
  --budget 900
```

## Patch visibility

```bash
./target/debug/pi --store .pi list-patches
./target/debug/pi --store .pi inspect-patch <patch_id>
./target/debug/pi --store .pi apply <patch_id>
```

## Schema migrations

Dry run:

```bash
./target/debug/pi --store .pi migrate --dry-run
```

Rewrite with backup:

```bash
./target/debug/pi --store .pi migrate --backup
```

JSON output:

```bash
./target/debug/pi --store .pi migrate --dry-run --json
```

## Belief revision

Reinforce a record:

```bash
./target/debug/pi --store .pi reinforce <record_id> \
  --evidence-uri test:reinforcement \
  --evidence-kind test \
  --reason "new validation supports this stored claim" \
  --apply
```

Supersede a record:

```bash
./target/debug/pi --store .pi supersede <record_id> \
  --class requirement \
  --claim "Updated governed claim." \
  --project pi-governance-rs \
  --tag belief-revision \
  --evidence-uri conversation:supersede \
  --reason "the previous claim was refined" \
  --apply \
  --force
```

Tombstone a record:

```bash
./target/debug/pi --store .pi tombstone <record_id> \
  --evidence-uri review:tombstone \
  --evidence-kind human-review \
  --reason "record is no longer valid but must remain auditable" \
  --apply \
  --force
```

Contest and resolve a record:

```bash
./target/debug/pi --store .pi contest <record_id> \
  --evidence-uri review:contest \
  --evidence-kind human-review \
  --reason "new evidence disputes this stored claim" \
  --apply \
  --force

./target/debug/pi --store .pi resolve-contest <record_id> \
  --resolution uphold \
  --evidence-uri review:uphold \
  --evidence-kind human-review \
  --reason "review confirmed this claim should remain active" \
  --apply \
  --force
```

## Portable import/export

v0.6.0 adds portable JSON bundles for moving governed memory between stores, machines, or coding agents.

Export all records, patches, and events:

```bash
./target/debug/pi --store .pi export --output /tmp/pi-export.json
```

Export only a project-relevant slice:

```bash
./target/debug/pi --store .pi export \
  --project pi-governance-rs \
  --output /tmp/pi-project-export.json
```

Export a redacted bundle:

```bash
./target/debug/pi --store .pi export \
  --project pi-governance-rs \
  --redacted \
  --output /tmp/pi-redacted-export.json
```

Dry-run an import:

```bash
./target/debug/pi --store /tmp/pi-import-store import /tmp/pi-export.json --dry-run --json
```

Import with backup:

```bash
./target/debug/pi --store /tmp/pi-import-store import /tmp/pi-export.json --backup
```

Import is merge-only: duplicate record, patch, and event IDs are skipped rather than overwritten.

## MCP stdio mode

Run the MCP server directly:

```bash
./target/debug/pi --store /absolute/path/to/.pi mcp-stdio
```

Manual tools/list smoke test:

```bash
printf '%s\n' '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}' \
| ./target/debug/pi --store /absolute/path/to/.pi mcp-stdio
```

MCP tools include:

```text
pi.retrieve_context
pi.propose_record
pi.supersede_record
pi.tombstone_record
pi.reinforce_record
pi.contest_record
pi.resolve_contest
pi.apply_patch
pi.list_patches
pi.inspect_patch
pi.migrate_schema
pi.export_store
pi.import_store
pi.doctor
pi.list_records
```

## v0.6.0 changes

- Adds `StoreExportBundle`, `StoreExportOptions`, `StoreImportOptions`, and `StoreImportReport`.
- Adds `pi export` and `pi import` CLI commands.
- Adds MCP tools `pi.export_store` and `pi.import_store`.
- Adds merge-only import semantics that skip duplicate IDs instead of overwriting.
- Adds optional backup before actual imports.
- Adds project-filtered and redacted exports.
- Adds store and engine tests for export/import behavior.
- Updates package, CLI, and MCP server version to `0.6.0`.

## Validation

```bash
cargo check --workspace
cargo test --workspace
cargo build -p pi-cli
./target/debug/pi --version
```

Expected:

```text
pi 0.6.0
```
