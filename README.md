# PI Governance Rust Port

Current milestone: `0.5.1`.

This workspace is a portable Rust implementation of a PI-style governance layer for coding agents. It exposes a CLI and an MCP stdio server around a governed JSONL store.

## Workspace layout

```text
crates/
  pi-core/        Core records, patches, evidence, policy, schema constants
  pi-store/       JSONL store, file locking, backups, schema migrations
  pi-retrieval/   Deterministic context retrieval
  pi-governance/  Runtime engine: propose, apply, retrieve, doctor, migrate, revise beliefs
  pi-mcp/         MCP stdio adapter
  pi-cli/         `pi` command-line interface
```

## Build and test

```bash
cargo check --workspace
cargo test --workspace
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

## Belief revision

v0.5.0 added direct governed belief-revision commands. v0.5.1 completes the contested-claim half of the workflow. These create normal patches, so each operation is visible through `list-patches`, inspectable with `inspect-patch`, and applied with `apply` unless `--apply` is provided.

### Reinforce a record

Reinforcement adds new evidence to an active record and increases confidence by a bounded amount.

```bash
./target/debug/pi --store /home/mel/Documents/Projects/pi-governance-rs/.pi reinforce <record_id> \
  --evidence-uri test:reinforcement \
  --evidence-kind test \
  --reason "new tests support this stored claim" \
  --apply
```

### Supersede a record

Supersession marks the target record as `Superseded` and creates a replacement record that references the old record in `supersedes`.

Supersession requires manual review by policy, so use `--force` only when explicitly approved.

```bash
./target/debug/pi --store /home/mel/Documents/Projects/pi-governance-rs/.pi supersede <record_id> \
  --class requirement \
  --claim "Belief revision must support reinforcement, supersession, and tombstones." \
  --project pi-governance-rs \
  --tag belief-revision \
  --evidence-uri conversation:v0.5.0 \
  --reason "the original claim was refined after implementation" \
  --apply \
  --force
```

### Tombstone a record

Tombstoning marks an active record as `Tombstoned` while retaining it in the audit trail.

Tombstones require manual review by policy, so applying immediately requires `--force`.

```bash
./target/debug/pi --store /home/mel/Documents/Projects/pi-governance-rs/.pi tombstone <record_id> \
  --evidence-uri review:v0.5.0 \
  --evidence-kind human-review \
  --reason "record is no longer valid but must remain auditable" \
  --apply \
  --force
```


### Contest a record

Contesting marks an active record as `Contested` while preserving it in the audit trail. Contested records are excluded from active retrieval until resolved. Contesting requires evidence and manual-review force when applying immediately.

```bash
./target/debug/pi --store /home/mel/Documents/Projects/pi-governance-rs/.pi contest <record_id> \
  --evidence-uri review:v0.5.1-contest \
  --evidence-kind human-review \
  --reason "new evidence disputes this stored claim" \
  --apply \
  --force
```

### Resolve a contest

A contested record can be resolved by upholding it, tombstoning it, or superseding it with a replacement claim. Resolution requires manual review, so immediate application requires `--force`.

Uphold the record and return it to `Active`:

```bash
./target/debug/pi --store /home/mel/Documents/Projects/pi-governance-rs/.pi resolve-contest <record_id> \
  --resolution uphold \
  --evidence-uri review:v0.5.1-uphold \
  --evidence-kind human-review \
  --reason "review confirmed this claim should remain active" \
  --apply \
  --force
```

Supersede the contested record:

```bash
./target/debug/pi --store /home/mel/Documents/Projects/pi-governance-rs/.pi resolve-contest <record_id> \
  --resolution supersede \
  --class requirement \
  --claim "Belief revision must support contested claims and explicit resolutions." \
  --project pi-governance-rs \
  --tag belief-revision \
  --evidence-uri conversation:v0.5.1 \
  --reason "review found the older claim should be replaced" \
  --apply \
  --force
```

Tombstone the contested record:

```bash
./target/debug/pi --store /home/mel/Documents/Projects/pi-governance-rs/.pi resolve-contest <record_id> \
  --resolution tombstone \
  --evidence-uri review:v0.5.1-tombstone \
  --evidence-kind human-review \
  --reason "review confirmed this claim is invalid but auditable" \
  --apply \
  --force
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

v0.4.0 added schema migration support for legacy JSONL files. Older records created before v0.3.0 may be missing `schema_version`. That is expected until they are migrated.

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
pi.supersede_record
pi.tombstone_record
pi.reinforce_record
pi.contest_record
pi.resolve_contest
pi.apply_patch
pi.list_patches
pi.inspect_patch
pi.migrate_schema
pi.doctor
pi.list_records
```

## v0.5.1 changes

- Adds `pi contest`.
- Adds `pi resolve-contest`.
- Adds MCP tools `pi.contest_record` and `pi.resolve_contest`.
- Adds `RecordStatus::Contested`.
- Adds contest and resolution patch operations.
- Adds policy and engine tests for contested claims and review resolution.
- Updates package, CLI, and MCP server version to `0.5.1`.

## v0.5.0 changes

- Adds `pi reinforce`.
- Adds `pi supersede`.
- Adds `pi tombstone`.
- Adds MCP tools `pi.reinforce_record`, `pi.supersede_record`, and `pi.tombstone_record`.
- Adds patch constructors for reinforcement, supersession, and tombstoning.
- Adds governance engine methods for belief revision.
- Adds tests covering reinforcement, supersession, and tombstone flows.
- Updates package, CLI, and MCP server version to `0.5.0`.

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
