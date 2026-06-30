# Release Strategy

## Release Principles

PI Governance releases should be conservative, auditable, and easy to verify from source. `v1.0.0` is the stable public release.

## Historical rc.8 Baseline

rc.8 validated CLI checks, MCP interoperability, review actions, maintenance scan, local retrieval modes, redacted export metadata, and schema documentation. rc.9 added portable workflow parity and is promoted by v1.0.0.

## Feature Freeze Strategy

The stable release is a release-only pass from rc.9. Do not add product features, redesign governance semantics, or alter runtime behavior during release preparation.

## Stable Release Process

1. Re-run full workspace, CLI, MCP, interop, docs, security, fresh clone, and archive checks.
2. Stable v1.0.0 bumps the package/runtime version to `1.0.0` without runtime feature expansion.
3. Update README, CHANGELOG, wiki docs, release docs, and product guide stable wording.
4. Re-run all checks after the bump.
5. Tag `v1.0.0` only after all gates pass.
6. Publish artifacts only after tag and archive verification.

## Artifact Publishing Strategy

Primary artifact is source from Git. Optional GitHub release assets and future packaged binaries should be generated from the verified tag and inspected before publication.

## Crates.io Strategy

Crates.io publishing is a future target. Do not claim it has happened until the package is actually published and verified.

## GitHub Release Strategy

Use the verified `v1.0.0` tag, include release notes that match `CHANGELOG.md`, attach only verified assets, and avoid claiming unsupported features.

## Version Bump Strategy

Change only version identifiers and stable wording required for the release unless a blocker requires a targeted fix. The version command must report `pi 1.0.0` for stable.

## Changelog Strategy

`CHANGELOG.md` keeps historical rc entries and includes a top stable v1.0.0 entry.

## Documentation Strategy

Docs should stay honest about scope: no embeddings, no vector backend, no hosted service, no dashboard, no secret-scanner claims, and no shipped-stable wording before the stable release.

## MCP Compatibility Strategy

For stable v1.0.0, direct MCP, `mcp-config`, `mcp-install`, `mcp-doctor`, namespace propagation, client-prefixed tool names, and structuredContent compatibility were validated for PI agent and Codex CLI; OpenCode install/doctor passed, with the live client run documented as an environmental/client-run limitation.

## Rollback Strategy

Keep previous tags and do not delete tags. If stable has an issue, document the known issue, revert via Git if needed, and publish a patch release rather than rewriting release history.

## Post-release Patch Policy

Patch releases should be small, targeted, documented, and independently verified through the same core checks that guard stable.

## Portable Workflow Parity

`v1.0.0` adds deterministic portable memory workflow parity: `memory-worth`, `capture`, `inbox`, `context`, `session add/search/decisions`, `recall-xray`, explicit L1/L2/L3 layers, trust class, durability, source kind, and minimal verification gates. Capture creates candidates or L3 evidence only; it does not silently apply durable L1/L2 memory. L1 is never auto-applied. L3 is session/evidence context, not authoritative memory.
