# Export Bundle Schema

Schema version 1 is the stable portable-bundle contract for v1.1.0.

Export bundles contain `schema_version`, `format`, `producer`, `exported_at`, `redacted`, `redaction`, `namespace`, `all_namespaces`, `project`, `records`, `patches`, `evidence`, `inquiries`, `sessions`, `reinforcement`, `events`, `tombstones`, and `warnings`. Record scope levels include `global`, `project`, `domain`, and `session`.

Reconciliation reports compare the eight artifact sections by stable ID and include directional, matching, divergent, duplicate, and conflicting ID sets. Envelope export time and producer version are not semantic differences; record status, scope, timestamps, and claims are.

Redaction metadata is best-effort and includes fields checked, fields redacted, and notes. Users must review bundles before sharing. PI is not a secret scanner or DLP system.
