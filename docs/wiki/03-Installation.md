# Installation

```bash
git clone <repository-url> pi-governance-rs
cd pi-governance-rs
cargo build -p pi-cli
./target/debug/pi --version
```

Expected for the current stable-release candidate:

```text
pi 1.0.0-rc.8
```

## Troubleshooting Build Failures

- Confirm Rust and Cargo are installed.
- Run `cargo check --workspace` for compile diagnostics.
- Run `cargo test --workspace` after build fixes.
- Do not edit runtime behavior for docs-only release work unless validation proves documentation is inaccurate.


---

Related: [Wiki index](../WIKI_INDEX.md), [Deployment checklist](../DEPLOYMENT_CHECKLIST.md), [Release strategy](../RELEASE_STRATEGY.md), [Stable v1 gate](../STABLE_V1_GATE.md).
