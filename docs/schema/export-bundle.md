# Export Bundle Schema

Schema is release-candidate level and intended to stabilize at v1.0.0.

Export bundles contain `schema_version`, `exported_at`, `redacted`, `redaction`, `namespace`, `all_namespaces`, `project`, `records`, `patches`, and `events`.

Redaction metadata is best-effort and includes fields checked, fields redacted, and notes. Users must review bundles before sharing. PI is not a secret scanner or DLP system.
