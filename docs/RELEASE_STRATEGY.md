# Release Strategy

## Release principles

PI Governance releases are conservative, auditable, and reproducible from source. Runtime behavior, public documentation, crate metadata, and release notes must describe the same version.

## v1.1.0 release process

1. Run rustfmt, clippy with warnings denied, workspace tests, release build, smoke test, and release audit.
2. Verify all six workspace packages report version `1.1.0` and the binary reports `pi 1.1.0`.
3. Inspect every package file list and scan packaged text for private paths, hidden bidirectional controls, and secret-like content.
4. Verify local `cargo install --path crates/pi-cli --locked`.
5. Publish supporting crates in dependency order, waiting for each version to appear on crates.io before publishing dependents.
6. Publish `pi-governance-rs` last and verify `cargo install pi-governance-rs --version 1.1.0` from the registry.
7. Create and push `v1.1.0` only after the release commit and package artifacts are verified.

## Artifact publishing

The primary artifacts are the crates.io packages and the source archive for the verified Git tag. Optional compiled binaries may be attached to a GitHub release only when they are built reproducibly and accompanied by checksums.

## Crates.io publishing order

1. `pi-governance-core`
2. `pi-governance-store`
3. `pi-governance-retrieval`
4. `pi-governance-engine`
5. `pi-governance-mcp`
6. `pi-governance-rs`

Do not claim a package is available until crates.io serves the expected version and a registry installation succeeds.

## Documentation and security

Public documentation must remain user-facing and accurate. Release archives must exclude local stores, credentials, private configuration, development reports, and machine-specific paths. Redaction, privacy purge, reconciliation, and migration guarantees must match the tested implementation.

## Compatibility

Direct MCP, namespace propagation, client-prefixed tool names, structured content, portable bundle import/export, and report-only reconciliation are compatibility gates for v1.1.0.

## Rollback

Never rewrite or delete a published version. If a release has a defect, document it, revert through Git where appropriate, and publish a new patch release after the full release gates pass.
