# Policy Schema

Schema is release-candidate level and intended to stabilize at v1.0.0.

Policy profiles are `permissive`, `standard`, and `strict`. Namespace policy config maps namespaces to profiles and falls back to the default policy.

Policies decide whether patch operations are allowed, require manual review, or are rejected. They do not bypass the patch governance flow.
