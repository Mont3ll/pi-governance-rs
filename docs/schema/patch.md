# Patch Schema

Schema version 1 is the stable patch contract for v1.1.0.

Patches are auditable proposed changes. Key fields: `schema_version`, `namespace`, `id`, `operation`, `status`, `target_id`, `proposed_record`, `contest_resolution`, `evidence`, `reason`, `created_at`, and `updated_at`.

Patch operation values include `propose_record`, `reinforce_record`, `supersede_record`, `tombstone_record`, `contest_record`, and `resolve_contest`.

Patch status values: `proposed`, `applied`, `rejected`, `deferred`.
