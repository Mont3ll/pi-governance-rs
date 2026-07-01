# Release and Deployment

## Release Philosophy

`v1.0.0` is the current validated stable public release. Stable is release-only from rc.9 with no runtime feature expansion. No runtime feature expansion was added during the stable release pass.

## Versioning Strategy

Release candidates use `1.0.0-rc.N`; stable uses `1.0.0` only after gates pass.

## Release Candidate Flow

v1.0.0 is the stable public release promoted from rc.9 after local validation, direct MCP validation, PI agent live validation, Codex CLI live validation, and a documented OpenCode client-run limitation.

## Stable v1.0.0 Gate

Required gates: `cargo check --workspace` pass, `cargo test --workspace` pass, `cargo build -p pi-governance-rs` pass, `pi --version` shows `1.0.0`, smoke-test pass, release-audit pass, fresh clone verification pass, archive content verification pass, OpenCode install/doctor pass with live rc.9 environmental/client-run limitation documented, Codex CLI interoperability pass, PI agent interoperability pass, MCP setup/install/doctor pass, hidden/bidi scan no matches, secret scan no real secrets, local path scan no public path leakage, README stable wording correct, CHANGELOG stable entry correct, docs stable wording correct, schemas validate, and product guide updated.

## Deployment Targets

- source build from Git
- GitHub source archive
- optional GitHub release assets
- future crates.io package
- future packaged binaries

## Artifact Strategy

Artifacts should be reproducible from Git and verified by archive inspection before publication.

## Crates.io Strategy

Do not claim crates.io publishing has happened until it has. Treat crates.io as a future publication target pending final packaging decisions.

## GitHub Release Strategy

Create the GitHub release only after all gates pass. Attach optional verified assets if used.

## MCP Client Compatibility Strategy

For future releases, retest OpenCode, Codex CLI, and PI agent after version bumps and before publishing. Confirm client-prefixed tools, namespace propagation, and structuredContent.

## Rollback Strategy

Keep previous tags. Do not delete tags. If a stable issue is found, document the known issue and publish a patch release; revert via Git if needed.

## Post-release Monitoring

Watch issue reports for MCP client breakage, schema confusion, install failures, and docs drift.


---

Related: [Wiki index](../WIKI_INDEX.md), [Deployment checklist](../DEPLOYMENT_CHECKLIST.md), [Release strategy](../RELEASE_STRATEGY.md), [Stable v1 gate](../STABLE_V1_GATE.md).

## Portable Workflow Parity

`v1.0.0` adds deterministic portable memory workflow parity: `memory-worth`, `capture`, `inbox`, `context`, `session add/search/decisions`, `recall-xray`, explicit L1/L2/L3 layers, trust class, durability, source kind, and minimal verification gates. Capture creates candidates or L3 evidence only; it does not silently apply durable L1/L2 memory. L1 is never auto-applied. L3 is session/evidence context, not authoritative memory.


## Distribution and MCP Sharing

Repository: https://github.com/Mont3ll/pi-governance-rs
License: MIT OR Apache-2.0

Install from source with `cargo build -p pi-governance-rs`, from Git with `cargo install --git https://github.com/Mont3ll/pi-governance-rs --tag v1.0.2 pi-governance-rs`, or from crates.io with `cargo install pi-governance-rs` after crates.io publishing is explicitly approved. `pi-governance-rs` is a standalone local stdio MCP server by default; it does not provide a hosted service in v1.0.0. It remains compatible with `pi-persistent-intelligence` through the shared PI memory contract.
