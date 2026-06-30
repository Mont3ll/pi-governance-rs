# Schema Reference

Schema docs: [record](../schema/record.md), [patch](../schema/patch.md), [policy](../schema/policy.md), [export bundle](../schema/export-bundle.md), and [MCP](../schema/mcp.md). JSON schemas live under [`schemas/`](../../schemas/).

Schemas are release-candidate level and intended to stabilize at `v1.0.0`.


---

Related: [Wiki index](../WIKI_INDEX.md), [Deployment checklist](../DEPLOYMENT_CHECKLIST.md), [Release strategy](../RELEASE_STRATEGY.md), [Stable v1 gate](../STABLE_V1_GATE.md).


## Distribution and MCP Sharing

Repository: https://github.com/Mont3ll/pi-governance-rs
License: MIT OR Apache-2.0

Install from source with `cargo build -p pi-cli`, from Git with `cargo install --git https://github.com/Mont3ll/pi-governance-rs --tag v1.0.0 pi-cli`, or from crates.io with `cargo install pi-cli` after crates.io publishing is explicitly approved. `pi-governance-rs` is a standalone local stdio MCP server by default; it does not provide a hosted service in v1.0.0. It remains compatible with `pi-persistent-intelligence` through the shared PI memory contract.
