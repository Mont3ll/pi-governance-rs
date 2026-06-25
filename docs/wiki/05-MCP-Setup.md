# MCP Setup

PI supports MCP stdio setup for OpenCode, Codex CLI, and PI agent/shared MCP. If existing client support is present for Claude, Cursor, or Inspector-style testing, use the generated config as the source of truth and run `mcp-doctor` before relying on a client.

## OpenCode Example

```bash
pi mcp-config opencode --command /path/to/pi --store /path/to/.pi --namespace default
pi mcp-install opencode --command /path/to/pi --store /path/to/.pi --namespace default --dry-run
pi mcp-install opencode --command /path/to/pi --store /path/to/.pi --namespace default --yes
pi mcp-doctor opencode --command /path/to/pi --store /path/to/.pi --namespace default
```

## Codex CLI

```bash
pi mcp-config codex --command /path/to/pi --store /path/to/.pi --namespace default
pi mcp-doctor codex --command /path/to/pi --store /path/to/.pi --namespace default
```

## PI Agent / Shared MCP

```bash
pi mcp-config pi-agent --command /path/to/pi --store /path/to/.pi --namespace default
pi mcp-doctor pi-agent --command /path/to/pi --store /path/to/.pi --namespace default
```

Restart the client after installation. Client-prefixed tool names may look like `pi.retrieve_context`, `pi-governance_pi_retrieve_context`, `pi_governance_pi.retrieve_context`, or `mcp__pi_governance__.pi_retrieve_context`.


---

Related: [Wiki index](../WIKI_INDEX.md), [Deployment checklist](../DEPLOYMENT_CHECKLIST.md), [Release strategy](../RELEASE_STRATEGY.md), [Stable v1 gate](../STABLE_V1_GATE.md).
