# Security and Memory Poisoning

Do not store secrets in PI. Memory poisoning risks include false claims, stale corrections, malicious instructions, weak evidence, and namespace confusion. PI mitigates these with patch-before-mutation, manual review, contest and resolve workflows, tombstones instead of hard delete, namespace isolation, and auditable history. Redacted export is limited and best-effort.

See [SECURITY.md](../../SECURITY.md) and [docs/MEMORY_POISONING.md](../MEMORY_POISONING.md).


---

Related: [Wiki index](../WIKI_INDEX.md), [Deployment checklist](../DEPLOYMENT_CHECKLIST.md), [Release strategy](../RELEASE_STRATEGY.md), [Stable v1 gate](../STABLE_V1_GATE.md).


## Distribution and MCP Sharing

Repository: https://github.com/Mont3ll/pi-governance-rs
License: MIT OR Apache-2.0

Install from source with `cargo build -p pi-governance-rs`, from Git with `cargo install --git https://github.com/Mont3ll/pi-governance-rs --tag v1.1.0 pi-governance-rs`, or from crates.io with `cargo install pi-governance-rs`. `pi-governance-rs` is a standalone local stdio MCP server by default; it does not provide a hosted service. It remains compatible with `pi-persistent-intelligence` through the shared PI memory contract.
