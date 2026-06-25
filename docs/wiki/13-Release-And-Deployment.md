# Release and Deployment

## Release Philosophy

`v1.0.0-rc.8` is the current validated stable-release candidate. Stable should be release-only if no blockers appear. No new features should land between rc.8 and stable unless required to fix a blocker.

## Versioning Strategy

Release candidates use `1.0.0-rc.N`; stable uses `1.0.0` only after gates pass.

## Release Candidate Flow

rc.8 remains the current validated release candidate. Validate locally, validate MCP clients, verify docs, then perform a stable-only version and wording pass if no blockers appear.

## Stable v1.0.0 Gate

Required gates: `cargo check --workspace` pass, `cargo test --workspace` pass, `cargo build -p pi-cli` pass, `pi --version` shows `1.0.0`, smoke-test pass, release-audit pass, fresh clone verification pass, archive content verification pass, OpenCode interoperability pass, Codex CLI interoperability pass, PI agent interoperability pass, MCP setup/install/doctor pass, hidden/bidi scan no matches, secret scan no real secrets, local path scan no public path leakage, README stable wording correct, CHANGELOG stable entry correct, docs stable wording correct, schemas validate, and product guide updated.

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

Retest OpenCode, Codex CLI, and PI agent after version bump and before publishing. Confirm client-prefixed tools, namespace propagation, and structuredContent.

## Rollback Strategy

Keep previous tags. Do not delete tags. If a stable issue is found, document the known issue and publish a patch release; revert via Git if needed.

## Post-release Monitoring

Watch issue reports for MCP client breakage, schema confusion, install failures, and docs drift.


---

Related: [Wiki index](../WIKI_INDEX.md), [Deployment checklist](../DEPLOYMENT_CHECKLIST.md), [Release strategy](../RELEASE_STRATEGY.md), [Stable v1 gate](../STABLE_V1_GATE.md).
