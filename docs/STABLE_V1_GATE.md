# Stable v1 Gate

`v1.0.0-rc.8` is the current stable-release candidate. Stable `v1.0.0` must not be tagged or published until every gate passes.

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
- [ ] No feature additions after rc.8 unless blocker.

## Final Stable Release Checklist

- [ ] Bump `1.0.0-rc.8` to `1.0.0`.
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
cargo build -p pi-cli
./target/debug/pi --version
./target/debug/pi smoke-test
./target/debug/pi release-audit
```

Stable version output must be:

```text
pi 1.0.0
```

For rc.8 release-candidate validation, expected output remains:

```text
pi 1.0.0-rc.8
```

See [docs/DEPLOYMENT_CHECKLIST.md](DEPLOYMENT_CHECKLIST.md), [docs/RELEASE_STRATEGY.md](RELEASE_STRATEGY.md), [docs/wiki/13-Release-And-Deployment.md](wiki/13-Release-And-Deployment.md), and [docs/wiki/14-QA-And-Test-Matrix.md](wiki/14-QA-And-Test-Matrix.md).
