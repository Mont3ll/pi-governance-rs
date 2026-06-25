# Troubleshooting

## Client Shows Zero PI Tools

Run direct `tools/list`, verify binary path, restart client, and check client-prefixed names.

## Direct MCP Works but Client Does Not

Inspect generated config, ensure the client is reading the expected config, and restart stale server processes.

## Wrong Namespace or No Records Visible

Confirm `--namespace`, store path, and MCP config namespace. Use `namespace list` and `namespace doctor`.

## structuredContent Expected Record, Received Array

Use rc.8 or later docs/tests; structuredContent compatibility was specifically validated in rc.8.

## Wrong Binary, Store, or Config Path

Use absolute generic paths such as `/path/to/pi` and `/path/to/.pi` in configs. Run `mcp-doctor`.

## Retrieval Returns Zero Records

Use `--explain`, check namespace, status filters, project filters, `include-contested`, and `min-confidence`.

## Proposal Remains Pending

Inspect the patch, review it, then apply, reject, or defer.

## Redacted Export Does Not Replace Review

Redaction is best-effort. Review exports before sharing.


---

Related: [Wiki index](../WIKI_INDEX.md), [Deployment checklist](../DEPLOYMENT_CHECKLIST.md), [Release strategy](../RELEASE_STRATEGY.md), [Stable v1 gate](../STABLE_V1_GATE.md).
