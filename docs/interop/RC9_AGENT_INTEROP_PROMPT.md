# PI Governance v1.0.0 Agent Interoperability Prompt

You are testing PI Governance `v1.0.0` stable portable memory workflow parity through an MCP client. Do not edit repository files. Do not apply proposed memory unless explicitly instructed. Use namespace `interop-test`.

Report:

- CLIENT_NAME
- tool naming style observed
- portable workflow tools visible: `score_memory_worth`, `capture_candidates`, `build_context`, `session_add`, `session_search`, `session_decisions`, `recall_xray`
- capture candidate created
- candidate left pending
- context output produced
- session decision extracted
- recall-xray succeeded
- namespace `interop-test` respected
- no structured content errors
- no repo edits
- no proposal applied unless instructed
- raw errors, if any

Test these tools/capabilities:

1. `score_memory_worth`: score `Going forward, always run cargo test before release.`
2. `capture_candidates`: capture `do not skip release-audit before tagging`; confirm a patch is created and remains pending.
3. `list_inbox` if present, otherwise use `list_patches` and `inspect_patch`.
4. `build_context`: build context for `release tagging`.
5. `session_add`: add `#decision keep namespace interop-test for agent testing`.
6. `session_search`: search `namespace interop-test`.
7. `session_decisions`: confirm the decision marker is visible.
8. `recall_xray`: explain recall for `release tagging`.
9. Existing governance tools: `retrieve_context`, `propose_record`, `inspect_patch`, `inspect_record`, `maintenance_scan`, `doctor`, and `smoke_test`.

Expected safety behavior: capture creates candidates or L3/session evidence only. L1/L2 durable memory remains patch-governed.
