# MCP Server Sharing

PI Governance runs as a local stdio MCP server. Clients connect by launching the local `pi` binary with a store path and namespace.

## Local stdio deployment

```bash
pi mcp-config codex --command "$(which pi)" --store /path/to/.pi --namespace default
pi mcp-doctor codex --command "$(which pi)" --store /path/to/.pi --namespace default
```

Use `pi mcp-config` to print configuration for supported client shapes, then place that configuration where your client expects MCP servers.

## Common clients

Common setups include Codex CLI, OpenCode, PI agent, and other MCP clients that support local stdio servers such as Claude Desktop-compatible configurations.

## Verify tools/list

You can also inspect the server directly:

```bash
printf '%s\n' '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}' \
  | pi --store /path/to/.pi --namespace default mcp-stdio
```

## Security cautions

- Use a store path you control.
- Review proposed patches before applying them.
- Avoid sharing stores that contain private evidence or sensitive project context unless you have reviewed and redacted them.
- Prefer local stdio unless you explicitly need a remote deployment model.
