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

## v0.8.0 namespace isolation

PI supports logical namespace isolation while continuing to use the same JSONL store files. Existing records without namespace metadata deserialize into the `default` namespace, and existing CLI/MCP calls without namespace arguments continue to use `default`.

Namespaces are logical filters, not separate physical stores. v0.8.0 does not split files or introduce a database; record IDs remain globally unique in the current implementation, so duplicate imported IDs are skipped even if namespaces differ.

CLI examples:

```bash
pi --namespace pi-governance-rs propose --class requirement --claim "Namespace isolation prevents cross-project memory leakage." --evidence-uri conversation:v0.8.0 --apply

pi --namespace pi-governance-rs retrieve "namespace isolation" --project pi-governance-rs --explain

pi namespace list
pi namespace doctor

pi --namespace pi-governance-rs export --output /tmp/pi-governance-export.json

pi --namespace sandbox import /tmp/pi-governance-export.json --dry-run
```

MCP tools accept an optional `namespace` string argument, defaulting to `default`, including `pi.retrieve_context`, `pi.propose_record`, belief-revision tools, export/import, list, and doctor-style tools. Namespace inspection is available through `pi.list_namespaces` and `pi.namespace_doctor`.

Export/import behavior:

- `pi export` exports the current namespace.
- `pi --namespace x export` exports namespace `x`.
- `pi export --all-namespaces` exports all namespaces.
- `pi import bundle.json` imports records into the current namespace, rewriting bundle record namespaces.
- `pi import bundle.json --preserve-namespaces` keeps namespaces from the bundle.

## v0.9.0 policy profiles and operating modes

PI stores policy configuration at `.pi/config.json`. If the file is absent, PI uses an in-memory default config with `default_policy: standard` and no namespace overrides. Policies are resolved by namespace override, then default policy, then `standard`.

Profiles:

- `permissive`: ordinary proposals and reinforcement are allowed; supersede is allowed with a warning; identity rules, tombstones, contests, and destructive contest resolutions still require manual review.
- `standard`: preserves the existing governance behavior.
- `strict`: all mutation operations require manual review unless explicitly applied with `--force`; hard validation failures remain rejects.

Manual-review decisions can still be applied with `--force`. Rejects remain rejects in every profile.

CLI examples:

```bash
pi config show
pi config set-policy default strict
pi config set-policy sandbox permissive

pi --namespace sandbox propose \
  --class requirement \
  --claim "Sandbox can use permissive policy for local experimentation." \
  --evidence-uri smoke:v0.9.0 \
  --apply

pi policy doctor
pi policy explain supersede
```

MCP tools include `pi.config_show`, `pi.config_set_policy`, `pi.policy_doctor`, and `pi.policy_explain`. Mutation responses include the effective `policy_profile` where practical.

## v0.10.0 release hardening and adapter polish

Build locally with `cargo build -p pi-cli`, then run `./target/debug/pi --version`.
Quickstart: `pi init`, `pi propose --class requirement --claim "..." --evidence-uri conversation:... --apply`, then `pi retrieve "..."`.

### MCP setup

Generate adapter snippets with:

```bash
pi mcp-config claude
pi mcp-config cursor
pi mcp-config inspector --command /absolute/path/to/pi --store /absolute/path/to/.pi
```

### Command matrix

`init`, `doctor`, `migrate`, `config`, `policy`, `namespace`, `propose`, `apply`, `reinforce`, `supersede`, `tombstone`, `contest`, `resolve-contest`, `retrieve`, `export`, `import`, `list`, `list-patches`, `inspect-patch`, `mcp-stdio`, `mcp-config`, `smoke-test`, `release-audit`, `changelog`.

### JSON diagnostics and smoke tests

```bash
pi doctor --json
pi namespace doctor --json
pi policy doctor --json
pi smoke-test
pi smoke-test --json
```

### Release checklist

```bash
cargo check --workspace
cargo test --workspace
cargo build -p pi-cli
./target/debug/pi --version
./target/debug/pi smoke-test
./target/debug/pi smoke-test --json
./target/debug/pi release-audit
./target/debug/pi release-audit --json
./target/debug/pi doctor --json
git status
```

Version history is maintained in `CHANGELOG.md`.

## v0.10.1 audit and release-candidate cleanup

Run `pi release-audit` or `pi release-audit --json` before release-candidate tagging. The audit covers JSON diagnostics, smoke tests, changelog coverage, README command matrix coverage, and MCP adapter config generation. Hidden Unicode and secret/path scans should be run with the documented grep commands in the release checklist and should avoid `.pi`, `target`, and generated exports.
