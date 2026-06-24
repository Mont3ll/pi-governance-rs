# Record Schema

Schema is release-candidate level and intended to stabilize at v1.0.0.

Records are governed memory entries stored in JSONL. Key fields: `schema_version`, `namespace`, `id`, `class`, `claim`, `confidence`, `scope`, `tags`, `evidence`, `status`, `supersedes`, `created_at`, and `updated_at`.

Record status values: `active`, `contested`, `superseded`, `tombstoned`.

Records are only created or changed through governed patches; JSONL remains the source of truth.
