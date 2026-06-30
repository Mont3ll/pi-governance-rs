# pi-persistent-intelligence Compatibility

## Stable Relationship

`pi-governance-rs` and `pi-persistent-intelligence` are separate standalone PI memory implementations.

- `pi-governance-rs` is the standalone Rust CLI/MCP runtime for governed PI memory across Codex, Claude, OpenCode, Cursor, PI agent, and other MCP-capable tools.
- `pi-persistent-intelligence` is the standalone lightweight pi-agent-native memory extension.
- Users may use either project alone.
- Users may use both when they want pi-agent-native capture/curation UX plus global MCP memory governance.
- Neither project depends on the other to work.
- `pi-persistent-intelligence` remains a supported standalone project and does not require Rust.
- `pi-governance-rs` does not require `pi-persistent-intelligence`.

## Shared PI Memory Contract

Both projects implement or map to the shared PI memory contract:

- records
- patches
- candidates/inbox entries
- evidence
- inquiries/open questions
- L3 session entries
- reinforcement events
- namespaces
- profile/project scope
- `l1_identity`, `l2_playbook`, and `l3_session` layers
- `memory_kind`, `rule_type`, `trust_class`, `durability`, `source_kind`, and verification metadata
- active, contested, superseded, tombstoned, and deleted record statuses
- proposed, applied, rejected, and deferred patch statuses
- redaction metadata for portable bundles

## Interoperability

The compatibility layer is import/export and shared schemas, not a dependency relationship.

`pi-governance-rs` can export and import portable JSON bundles. `pi-persistent-intelligence` v0.12.0 can export/import compatible `pi-governance` bundles and can optionally run bridge diagnostics against an external Rust runtime. The JavaScript package does not run its own MCP server.

## Safety Contract

- L1 never auto-applies.
- Capture creates candidates or L3/session evidence, not silent L1/L2 mutation.
- Low-trust sources cannot auto-apply.
- Repository/generated/third-party content requires review.
- Tombstones prevent re-promotion.
- Redacted export is best-effort and user-reviewed.
