# Export, Import, and Redaction

```bash
pi --store .pi export --output pi-export.json
pi --store .pi export --redacted --output pi-export.redacted.json
pi --store .pi export --all-namespaces --redacted --output pi-export.all.redacted.json
pi --store .pi import pi-export.json
pi --store .pi import pi-export.json --backup
```

Redacted export includes metadata indicating redaction was requested. PI is not a secret scanner or DLP system. Do not store secrets in PI. Redacted export is best-effort and must be reviewed before sharing.


---

Related: [Wiki index](../WIKI_INDEX.md), [Deployment checklist](../DEPLOYMENT_CHECKLIST.md), [Release strategy](../RELEASE_STRATEGY.md), [Stable v1 gate](../STABLE_V1_GATE.md).


## Distribution and MCP Sharing

Repository: https://github.com/Mont3ll/pi-governance-rs
License: MIT OR Apache-2.0

Install from source with `cargo build -p pi-governance-rs`, from Git with `cargo install --git https://github.com/Mont3ll/pi-governance-rs --tag v1.0.2 pi-governance-rs`, or from crates.io with `cargo install pi-governance-rs` after crates.io publishing is explicitly approved. `pi-governance-rs` is a standalone local stdio MCP server by default; it does not provide a hosted service in v1.0.0. It remains compatible with `pi-persistent-intelligence` through the shared PI memory contract.
