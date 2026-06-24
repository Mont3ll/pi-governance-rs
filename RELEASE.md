# Release candidate notes





## v1.0.0-rc.7 MCP response compatibility notes

`v1.0.0-rc.7` is the current MCP response compatibility and namespace correctness release candidate. It keeps the rc.6 client setup helpers and fixes object-shaped `structuredContent` for list-style MCP tools plus server-default namespace propagation. It does not add capture, semantic retrieval, graph memory, dashboards, hosted services, connectors, or stable `v1.0.0`.

## v1.0.0-rc.4 documentation notes

`v1.0.0-rc.4` is the current release-candidate documentation consistency pass. It cleans README duplication, stale version references, command examples, MCP documentation, security and memory-poisoning wording, and adds `docs/product-guide.html`. It does not claim stable `v1.0.0` has shipped and does not change governance semantics.

## v1.0.0-rc.3 usability notes

`v1.0.0-rc.3` focuses on open-source usability. It adds a review inbox (`pi review`), a safe demo store (`pi demo`), agent instruction output, governed skill examples, security documentation, memory-poisoning guidance, codebase-memory-mcp complement documentation, and pi-persistent-intelligence compatibility notes. It does not change existing governance semantics or remove/rename CLI commands or MCP tools.

## v1.0.0-rc.2 compatibility notes

`v1.0.0-rc.2` is a release-candidate soak and compatibility pass. It verifies fresh-user clean clone installation, README examples, MCP client configuration, MCP smoke flows, clean-store import/export portability, namespace and policy behavior after fresh init, JSON diagnostics, and release-audit output. It adds no new governance semantics and does not remove or rename any CLI commands or MCP tools.

## v1.0.0-rc.1 — first release candidate

This sprint is a release-candidate packaging sprint only. It adds no new governance semantics and preserves the JSONL store, existing CLI behavior, and existing MCP behavior.

## Public CLI surface freeze

The following command names are considered part of the release-candidate public surface for the current release-candidate line (introduced in `v1.0.0-rc.1`):

```text
init
doctor
migrate
config
policy
namespace
propose
apply
reinforce
supersede
tombstone
contest
resolve-contest
retrieve
export
import
list
list-patches
inspect-patch
mcp-stdio
mcp-config
smoke-test
release-audit
changelog
```

Command names are frozen for the release-candidate line. Output details may still receive compatibility fixes before stable `v1.0.0`.

## MCP schema freeze

MCP tool names are frozen for the release-candidate line. The governed memory tool surface includes:

```text
pi.retrieve_context
pi.propose_record
pi.apply_patch
pi.list_patches
pi.inspect_patch
pi.migrate_schema
pi.doctor
pi.list_records
pi.reinforce_record
pi.supersede_record
pi.tombstone_record
pi.contest_record
pi.resolve_contest
pi.export_store
pi.import_store
pi.config_show
pi.config_set_policy
pi.policy_doctor
pi.policy_explain
pi.smoke_test
pi.mcp_config
pi.changelog
pi.list_namespaces
pi.namespace_doctor
```

Confirm with:

```bash
STORE="/path/to/.pi"
BIN="/path/to/pi"
printf '%s\n' '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}' \
| "$BIN" --store "$STORE" mcp-stdio
```

## Fresh-user install flow

```bash
git clone <repo-url>
cd pi-governance-rs
cargo build -p pi-cli
./target/debug/pi init
./target/debug/pi smoke-test
./target/debug/pi mcp-config claude
```

Use generic local paths such as `/path/to/pi` and `/path/to/.pi` in public examples and MCP client configuration.

## Clean clone verification

Verify the release candidate in a temporary local clone:

```bash
git clone <repo-url> pi-governance-rs-rc1-clone
cd pi-governance-rs-rc1-clone
git checkout v1.0.0-rc.4
cargo check --workspace
cargo test --workspace
cargo build -p pi-cli
./target/debug/pi --version
./target/debug/pi smoke-test
./target/debug/pi release-audit
```

Expected version output:

```text
pi 1.0.0-rc.4
```

## Archive content verification

Release archives should include source and documentation files such as:

```text
Cargo.toml
Cargo.lock
README.md
CHANGELOG.md
RELEASE.md
crates/
```

Release archives must not include local runtime data or generated local artifacts:

```text
.pi/
target/
.env
local export files
local import files
local backups
```

## Release checklist dry run

```bash
cargo check --workspace
cargo test --workspace
cargo build -p pi-cli
./target/debug/pi --version
./target/debug/pi smoke-test
./target/debug/pi smoke-test --json | python -m json.tool >/dev/null
./target/debug/pi release-audit
./target/debug/pi release-audit --json | python -m json.tool >/dev/null
./target/debug/pi changelog
./target/debug/pi mcp-config claude
git status --short
```

## Release notes

`v1.0.0-rc.1` verifies:

- public CLI surface freeze
- MCP tool-name freeze
- fresh clone verification
- archive content verification
- release checklist verification
- no new governance semantics
