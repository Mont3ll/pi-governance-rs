# Agent Interoperability

## rc.8 Tested Clients

| Client | Status |
| --- | --- |
| OpenCode | pass |
| Codex CLI | pass |
| PI agent | pass |

## Tested Capabilities

`retrieve_context`, `propose_record`, `list_patches`, `inspect_patch`, `inspect_record`, `list_records`, `maintenance_scan`, `doctor`, `smoke_test`, review action discovery, namespace propagation, and structuredContent object compatibility.

## rc.8 Interoperability Prompt

Use the release-candidate interoperability prompt from the project release-preparation notes for full client validation. The prompt should verify direct MCP `tools/list`, tool calls, namespace propagation, structured content, and client-visible prefixed tool names.


---

Related: [Wiki index](../WIKI_INDEX.md), [Deployment checklist](../DEPLOYMENT_CHECKLIST.md), [Release strategy](../RELEASE_STRATEGY.md), [Stable v1 gate](../STABLE_V1_GATE.md).
