# PI Governance Rust Port

Current milestone: `0.4.0`.

This workspace is a portable Rust implementation of a PI-style governance layer for coding agents. It exposes a CLI and an MCP stdio server around a governed JSONL store.

## Workspace layout

```text
crates/
  pi-core/        Core records, patches, evidence, policy, schema constants
  pi-store/       JSONL store, file locking, backups, schema migrations
  pi-retrieval/   Deterministic context retrieval
  pi-governance/  Runtime engine: propose, apply, retrieve, doctor, migrate
  pi-mcp/         MCP stdio adapter
  pi-cli/         `pi` command-line interface
```

## Build

```bash
cargo check --workspace
cargo build -p pi-cli
```

## Initialize a local store

```bash
./target/debug/pi --store /home/mel/Documents/Projects/pi-governance-rs/.pi init
```

The store is local runtime data and should not be committed.

## Propose a record

```bash
./target/debug/pi --store /home/mel/Documents/Projects/pi-governance-rs/.pi propose \
  --class preference \
  --claim "User prefers exact React preview fidelity over reinterpretation." \
  --project figma-landing \
  --tag react \
  --tag fidelity \
  --evidence-uri conversation:2026-06-15 \
  --apply
```

## Retrieve context

```bash
./target/debug/pi --store /home/mel/Documents/Projects/pi-governance-rs/.pi retrieve \
  "React preview fidelity requirements" \
  --project figma-landing \
  --budget 900
```

## Patch visibility

```bash
./target/debug/pi --store /home/mel/Documents/Projects/pi-governance-rs/.pi list-patches
```

```bash
./target/debug/pi --store /home/mel/Documents/Projects/pi-governance-rs/.pi inspect-patch <patch_id>
```

Apply a pending patch:

```bash
./target/debug/pi --store /home/mel/Documents/Projects/pi-governance-rs/.pi apply <patch_id>
```

Do not include angle brackets when using a real patch id.

## Doctor

Human-readable:

```bash
./target/debug/pi --store /home/mel/Documents/Projects/pi-governance-rs/.pi doctor
```

JSON:

```bash
./target/debug/pi --store /home/mel/Documents/Projects/pi-governance-rs/.pi doctor --json
```

`doctor` reports:

```text
store path
lock path
current schema version
schema migration_needed
record/patch/event counts
schema audit per JSONL file
warnings/errors
```

## Migrations

v0.4.0 adds schema migration support for legacy JSONL files. Older records created before v0.3.0 may be missing `schema_version`. That is expected until they are migrated.

Preview migration changes without rewriting files:

```bash
./target/debug/pi --store /home/mel/Documents/Projects/pi-governance-rs/.pi migrate --dry-run
```

Run a migration with a backup:

```bash
./target/debug/pi --store /home/mel/Documents/Projects/pi-governance-rs/.pi migrate --backup
```

Get machine-readable output:

```bash
./target/debug/pi --store /home/mel/Documents/Projects/pi-governance-rs/.pi migrate --dry-run --json
```

Backups are written under:

```text
.pi/backups/
```

The migration currently adds or corrects `schema_version` on:

```text
records.jsonl root records
records.jsonl evidence refs
patches.jsonl root patches
patches.jsonl patch evidence refs
patches.jsonl proposed_record
patches.jsonl proposed_record evidence refs
events.jsonl root events
```

Invalid JSONL lines are preserved during migration and reported instead of being deleted.

## MCP stdio

Build first:

```bash
cargo build -p pi-cli
```

Run the MCP server directly:

```bash
./target/debug/pi --store /home/mel/Documents/Projects/pi-governance-rs/.pi mcp-stdio
```

A blank terminal is expected. The server is waiting for JSON-RPC messages on stdin.

MCP tools exposed:

```text
pi.retrieve_context
pi.propose_record
pi.apply_patch
pi.list_patches
pi.inspect_patch
pi.migrate_schema
pi.doctor
pi.list_records
```

## v0.4.0 changes

- Adds `pi migrate`.
- Adds `pi migrate --dry-run`.
- Adds `pi migrate --backup`.
- Adds `pi doctor --json` documentation and migration-needed reporting.
- Adds `pi.migrate_schema` MCP tool.
- Adds schema migration reports with per-file counts.
- Adds timestamped store backups under `.pi/backups/`.
- Adds test scaffolding for policy, store migrations, governance engine flows, and MCP server construction.
- Updates package, CLI, and MCP server version to `0.4.0`.

## Git safety

The `.gitignore` intentionally excludes:

```text
.pi/
target/
local MCP configs
agent/editor local folders
secrets and env files
archives and temporary output
```

`Cargo.lock` should be committed because this workspace contains a CLI binary.
