# codebase-memory-mcp plus PI

`codebase-memory-mcp` and PI solve complementary problems.

`codebase-memory-mcp` answers structural codebase questions:

- where is this function used?
- what routes exist?
- what calls this class?
- what is the impact of changing this file?

PI answers durable memory governance questions:

- what decision did we make last time?
- what workflow should this repo follow?
- what correction did the user give?
- what old belief has been superseded?
- what memory is contested?

## Workflow

1. Use codebase-memory-mcp to inspect architecture.
2. Agent identifies a project convention.
3. Agent proposes the convention into PI.
4. User reviews the patch with `pi review`.
5. Future agents retrieve that governed memory before editing.
6. If code changes, contest or supersede the old memory.

## Example

```bash
pi propose --class workflow \
  --claim "Before editing the MCP server, inspect tools/list and smoke_test behavior." \
  --project pi-governance-rs \
  --tag mcp \
  --evidence-uri codebase-memory-mcp:architecture-note

pi review
pi retrieve "MCP server editing workflow" --project pi-governance-rs --explain
```
