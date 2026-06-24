# MCP Troubleshooting

PI MCP stdio can work while an agent client still shows zero PI tools if the active client config does not contain a `pi-governance` server entry. rc.6 added setup helpers for that exact failure mode; rc.7 additionally fixes MCP list response compatibility and server-default namespace propagation.

## Quick Diagnosis

1. Generate the expected client config.
2. Install or dry-run the merge.
3. Run `mcp-doctor` before opening the client.
4. Restart the client after `mcp-install`.

## Direct mcp-stdio Smoke Test

```bash
printf '%s\n' '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}' \
| /path/to/pi --store /path/to/.pi --namespace default mcp-stdio
```

If the direct smoke test passes but the client shows 0 tools, the problem is client registration or client config loading, not PI runtime.

## Generate Config

```bash
pi mcp-config opencode --command /path/to/pi --store /path/to/.pi --namespace default
pi mcp-config codex --command /path/to/pi --store /path/to/.pi --namespace default
pi mcp-config pi-agent --command /path/to/pi --store /path/to/.pi --namespace default
```

## Install Config Safely

Preview first:

```bash
pi mcp-install opencode --command /path/to/pi --store /path/to/.pi --namespace default --dry-run
```

Install with confirmation bypassed:

```bash
pi mcp-install opencode --command /path/to/pi --store /path/to/.pi --namespace default --yes
```

`mcp-install` backs up existing configs before writing, preserves other MCP servers, and only adds or updates `pi-governance`. For JSONC files with comments it refuses to rewrite and prints a safe failure instead of destroying comments.

## Run MCP Doctor

```bash
pi mcp-doctor opencode --command /path/to/pi --store /path/to/.pi --namespace default
```

Doctor checks that the config exists, parses, contains `pi-governance`, points to an executable PI binary, references the expected store and namespace, ends with `mcp-stdio`, and can run a direct `tools/list`.

## OpenCode Setup

Default config: `~/.config/opencode/opencode.jsonc`.

```bash
pi mcp-install opencode --command /path/to/pi --store /path/to/.pi --namespace default --yes
pi mcp-doctor opencode --command /path/to/pi --store /path/to/.pi --namespace default
```

## Codex CLI Setup

Default config: `~/.codex/config.toml`.

```bash
pi mcp-install codex --command /path/to/pi --store /path/to/.pi --namespace default --yes
pi mcp-doctor codex --command /path/to/pi --store /path/to/.pi --namespace default
```

## PI Agent / Shared MCP Setup

Default config: `~/.config/mcp/mcp.json`.

```bash
pi mcp-install pi-agent --command /path/to/pi --store /path/to/.pi --namespace default --yes
pi mcp-doctor pi-agent --command /path/to/pi --store /path/to/.pi --namespace default
```

## Common Failure: Client Shows 0 PI Tools

Confirm the active client config contains a `pi-governance` entry. If direct `tools/list` passes but the client has no tools, inspect the client's config loading rules and restart the client.

## Common Failure: Wrong Working Directory

Use absolute paths for `--command` and `--store` in public testing. Relative paths may resolve differently inside the client.

## Common Failure: Wrong Binary Path

Run `pi mcp-doctor ...`; `Command exists` and `Command executable` must be ok.

## Common Failure: Wrong Store Path

Run `pi mcp-doctor ...`; `Store exists` and the configured `--store` value must match the expected store.

## Common Failure: Namespace Mismatch

The namespace in the client config must match the namespace used during testing, for example `interop-test`.

## Common Failure: Server Visible but Disconnected

Run the direct smoke test. If it fails, fix the binary, store, namespace, or permissions. If it passes, check client logs and config reload behavior.

## Client-Prefixed Tool Names

Some MCP clients expose PI tools with client/server-prefixed names, such as:

- `pi-governance_pi_retrieve_context`
- `pi_governance_pi.retrieve_context`

These are equivalent to `pi.retrieve_context`. The prefix is a client display or routing convention, not a different PI tool.

## Expected rc.7 Tool List

Expected MCP tools include:

- `pi.retrieve_context`
- `pi.propose_record`
- `pi.apply_patch`
- `pi.list_patches`
- `pi.inspect_patch`
- `pi.migrate_schema`
- `pi.doctor`
- `pi.list_records`
- `pi.reinforce_record`
- `pi.supersede_record`
- `pi.tombstone_record`
- `pi.contest_record`
- `pi.resolve_contest`
- `pi.export_store`
- `pi.import_store`
- `pi.config_show`
- `pi.config_set_policy`
- `pi.policy_doctor`
- `pi.policy_explain`
- `pi.smoke_test`

## Known Limitation: pi.inspect_record MCP Tool Deferred

`pi inspect-record` exists as a CLI command in rc.5+, but the `pi.inspect_record` MCP tool remains deferred unless added separately in a future release candidate. `mcp-doctor` reports whether it is present or unavailable.

## rc.8 MCP Notes

rc.8 adds `pi.inspect_record` and `pi.maintenance_scan`. `pi.retrieve_context` accepts `retriever` values `deterministic`, `lexical`, and `hybrid`. Some clients still show prefixed tool names; these are equivalent to the canonical `pi.*` names.
