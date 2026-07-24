# Release and Deployment

## Current release

`v1.1.0` is the current coordinated workspace release. It adds observability and quality reports, preview-first integrity and privacy repair, portable peer reconciliation, active store identity diagnostics, and expanded JS/Rust bundle compatibility.

## Required release gates

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `cargo build --release`
- `pi --version` reports `pi 1.1.0`
- `pi smoke-test` passes
- `pi release-audit` passes
- Package file lists, security scans, and local installation pass

## Deployment targets

- crates.io packages
- verified Git source tag and source archives
- optional checksum-verified GitHub release binaries

## Crates.io order

Publish `pi-governance-core`, `pi-governance-store`, `pi-governance-retrieval`, `pi-governance-engine`, and `pi-governance-mcp` before publishing `pi-governance-rs`. Wait until each dependency version is visible in the registry before publishing its dependents.

## MCP compatibility

Retest direct stdio startup, tool listing, store identity, namespace propagation, and structured results for supported clients before publishing.

## Rollback

Keep previous tags and published versions. Correct defects through a new patch release rather than deleting or rewriting release history.

---

Related: [Wiki index](../WIKI_INDEX.md), [Deployment checklist](../DEPLOYMENT_CHECKLIST.md), [Release strategy](../RELEASE_STRATEGY.md), [QA matrix](14-QA-And-Test-Matrix.md).
