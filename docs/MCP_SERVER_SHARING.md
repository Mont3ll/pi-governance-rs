# MCP Server Sharing

Repository: https://github.com/Mont3ll/pi-governance-rs
License: MIT OR Apache-2.0

## What an MCP server is

An MCP server exposes tools to an MCP-capable client. The client sends JSON-RPC messages, lists available tools, and calls those tools with structured arguments.

## Local stdio deployment

pi-governance-rs is a local-first MCP stdio server. The MCP client launches the pi binary as a subprocess. This keeps governed memory on the user's machine by default.

Example server command:

```bash
pi --store /path/to/.pi --namespace default mcp-stdio
```

## Remote HTTP deployment

Remote HTTP MCP deployment means a client connects to a network service instead of launching a local process. `pi-governance-rs` does not provide a hosted HTTP MCP server in v1.0.0. Remote hosted MCP is future work and would require separate authentication, deployment, tenancy, and data-protection design.

## Why local stdio first

Local stdio keeps memory files local, avoids a default network listener, avoids hosted-service assumptions, and matches the governance goal of explicit local control.

## Share with OpenCode

```bash
pi mcp-config opencode --command /path/to/pi --store /path/to/.pi --namespace default
pi mcp-install opencode --command /path/to/pi --store /path/to/.pi --namespace default
pi mcp-install opencode --command /path/to/pi --store /path/to/.pi --namespace default --yes
pi mcp-doctor opencode --command /path/to/pi --store /path/to/.pi --namespace default
```

## Share with Codex

```bash
pi mcp-config codex --command /path/to/pi --store /path/to/.pi --namespace default
pi mcp-install codex --command /path/to/pi --store /path/to/.pi --namespace default
pi mcp-doctor codex --command /path/to/pi --store /path/to/.pi --namespace default
```

## Share with PI agent

```bash
pi mcp-config pi-agent --command /path/to/pi --store /path/to/.pi --namespace default
pi mcp-install pi-agent --command /path/to/pi --store /path/to/.pi --namespace default
pi mcp-doctor pi-agent --command /path/to/pi --store /path/to/.pi --namespace default
```

## Claude and Cursor

`pi mcp-config` also supports generic Claude/Cursor-oriented config shapes for clients that accept local command MCP servers. Check the client's current MCP configuration location and prefer preview or manual config review before enabling.

## Verify tools/list

```bash
printf '%s\n' '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}' \
| pi --store /path/to/.pi --namespace default mcp-stdio
```

Expected tools include `pi.retrieve_context`, `pi.propose_record`, `pi.list_patches`, `pi.inspect_record`, `pi.maintenance_scan`, `pi.score_memory_worth`, `pi.capture_candidates`, `pi.build_context`, `pi.session_add`, `pi.session_search`, `pi.session_decisions`, and `pi.recall_xray`.

## Client-prefixed tool names

Some clients prefix server or namespace names. A tool may appear as `pi.retrieve_context`, `pi-governance_pi_retrieve_context`, `pi_governance_pi.retrieve_context`, or `mcp__pi_governance__.pi_retrieve_context`. This is client display behavior; the server tool names remain stable.

## Security cautions

- Use an absolute trusted `pi` binary path.
- Keep stores local unless you intentionally copy/export them.
- Do not store secrets as durable memory.
- Review imports and generated memory candidates before applying.
- Restart clients after config changes.
