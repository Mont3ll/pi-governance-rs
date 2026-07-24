# Policy Schema

Schema version 1 is the stable policy contract for v1.1.0.

Policy profiles are `permissive`, `standard`, and `strict`. Namespace policy config maps namespaces to profiles and falls back to the default policy.

Policies decide whether patch operations are allowed, require manual review, or are rejected. They do not bypass the patch governance flow.
