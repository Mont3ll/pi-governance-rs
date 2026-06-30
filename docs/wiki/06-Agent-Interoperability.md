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

## rc.9 Portable Workflow Parity

`v1.0.0-rc.9` adds deterministic portable memory workflow parity: `memory-worth`, `capture`, `inbox`, `context`, `session add/search/decisions`, `recall-xray`, explicit L1/L2/L3 layers, trust class, durability, source kind, and minimal verification gates. Capture creates candidates or L3 evidence only; it does not silently apply durable L1/L2 memory. L1 is never auto-applied. L3 is session/evidence context, not authoritative memory.
