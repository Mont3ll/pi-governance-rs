# Record Schema

Records are governed durable memory claims stored in local JSONL. `v1.0.0` includes portable workflow metadata while preserving backwards-compatible optional fields.

Required core fields include `id`, `namespace`, `class`, `claim`, `status`, `scope`, `evidence`, `confidence`, timestamps, and revision metadata.

## Portable Workflow Fields

- `layer`: `l1_identity`, `l2_playbook`, or `l3_session`.
- `memory_kind`: optional `fact`, `event`, `instruction`, or `task`.
- `rule_type`: optional `avoid_pattern`, `prefer_pattern`, `convention`, `architecture`, `workflow`, `preference`, `testing`, `correction`, or `tool`.
- `trust_class`: source trust signal such as `direct_user_instruction`, `user_correction`, `repository_text`, or `unknown`.
- `durability`: `temporary`, `task`, `project`, `long_term`, or `unknown`.
- `source_kind`: capture source such as `manual_cli`, `manual_mcp`, `session_text`, `stdin`, or `unknown`.

L1 records are never auto-applied. L3 records/session entries are evidence context, not authoritative durable memory.

See [`schemas/pi-record.schema.json`](../../schemas/pi-record.schema.json).
