# Export, Import, and Redaction

```bash
pi --store .pi export --output pi-export.json
pi --store .pi export --redacted --output pi-export.redacted.json
pi --store .pi export --all-namespaces --redacted --output pi-export.all.redacted.json
pi --store .pi import pi-export.json
pi --store .pi import pi-export.json --backup
pi --store .pi reconcile peer-export.json --json
```

Redacted export includes metadata indicating redaction was requested. PI is not a secret scanner or DLP system. Do not store secrets in PI. Redacted export is best-effort and must be reviewed before sharing.

Project filters retain global and matching project records, keep domain scope distinct from project scope, reconstruct compatible auxiliary sections before filtering, and report omitted unscoped artifacts. Generic events remain generic events.

`reconcile` compares snapshots rather than synchronizing stores. It normalizes set-like arrays but treats status, scope, timestamps, and claims as substantive. Both stores remain independent canonical peers, and the command has no mutation path.


---

Related: [Wiki index](../WIKI_INDEX.md), [Deployment checklist](../DEPLOYMENT_CHECKLIST.md), [Release strategy](../RELEASE_STRATEGY.md), [Stable v1 gate](../STABLE_V1_GATE.md).


## Distribution and MCP Sharing

Repository: https://github.com/Mont3ll/pi-governance-rs
License: MIT OR Apache-2.0

Install from source with `cargo build -p pi-governance-rs`, from Git with `cargo install --git https://github.com/Mont3ll/pi-governance-rs --tag v1.1.0 pi-governance-rs`, or from crates.io with `cargo install pi-governance-rs`. `pi-governance-rs` is a standalone local stdio MCP server by default; it does not provide a hosted service. It remains compatible with `pi-persistent-intelligence` through the shared PI memory contract.
