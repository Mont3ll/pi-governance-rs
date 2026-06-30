# Stable v1 Gate

`v1.0.0` is the current stable public release. Stable `v1.0.0` is tagged only after every local gate passes.

## Hard Pass/Fail Gates

- [ ] No known blocking interop failures.
- [ ] No known structuredContent errors.
- [ ] No namespace propagation failures.
- [ ] No release-audit failures.
- [ ] No tests failing.
- [ ] No local path leakage in public docs or artifacts.
- [ ] No real secrets in public docs or artifacts.
- [ ] No docs claiming unsupported features.
- [ ] No unpublished stable wording.
- [x] No runtime feature additions during the stable pass.

## Final Stable Release Checklist

- [x] Bump package/runtime version to `1.0.0`.
- [ ] Update README.
- [ ] Update CHANGELOG.
- [ ] Update docs/wiki.
- [ ] Update product guide.
- [ ] Run full checks.
- [ ] Run fresh clone.
- [ ] Run archive verification.
- [ ] Run interop smoke.
- [ ] Tag `v1.0.0`.
- [ ] Do not publish until all gates pass.

## Required Command Evidence

```bash
cargo check --workspace
cargo test --workspace
cargo build -p pi-governance-rs
./target/debug/pi --version
./target/debug/pi smoke-test
./target/debug/pi release-audit
```

Stable version output must be:

```text
pi 1.0.0
```

For stable validation, expected output is:

```text
pi 1.0.0
```

See [docs/DEPLOYMENT_CHECKLIST.md](DEPLOYMENT_CHECKLIST.md), [docs/RELEASE_STRATEGY.md](RELEASE_STRATEGY.md), [docs/wiki/13-Release-And-Deployment.md](wiki/13-Release-And-Deployment.md), and [docs/wiki/14-QA-And-Test-Matrix.md](wiki/14-QA-And-Test-Matrix.md).

## Portable Workflow Parity

`v1.0.0` adds deterministic portable memory workflow parity: `memory-worth`, `capture`, `inbox`, `context`, `session add/search/decisions`, `recall-xray`, explicit L1/L2/L3 layers, trust class, durability, source kind, and minimal verification gates. Capture creates candidates or L3 evidence only; it does not silently apply durable L1/L2 memory. L1 is never auto-applied. L3 is session/evidence context, not authoritative memory.
