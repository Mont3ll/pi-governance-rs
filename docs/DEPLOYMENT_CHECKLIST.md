# Deployment Checklist

`v1.0.0` is the current stable public release.

## Pre-release Local Checks

- [ ] Confirm clean working tree.
- [ ] Confirm intended branch and tag state.
- [ ] Confirm no new features are being added after rc.8 unless a issue exists.

## Workspace Checks

```bash
cargo check --workspace
cargo test --workspace
cargo build -p pi-governance-rs
```

## CLI Checks

```bash
./target/debug/pi --version
./target/debug/pi smoke-test
./target/debug/pi release-audit
./target/debug/pi demo --store /tmp/pi-release-demo --reset
./target/debug/pi --store /tmp/pi-release-demo retrieve "release workflow" --retriever hybrid --explain
./target/debug/pi --store /tmp/pi-release-demo maintenance scan
```

## MCP Checks

```bash
printf '%s\n' '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}' \
| ./target/debug/pi --store /tmp/pi-release-demo --namespace default mcp-stdio
```

```bash
./target/debug/pi mcp-config opencode --command /tmp/pi-bin --store /tmp/pi-store --namespace interop-test
./target/debug/pi mcp-config codex --command /tmp/pi-bin --store /tmp/pi-store --namespace interop-test
./target/debug/pi mcp-config pi-agent --command /tmp/pi-bin --store /tmp/pi-store --namespace interop-test
```

## Interop Checks

- [ ] OpenCode install/doctor pass; live rc.9 client run documented as environmental/client-run incomplete.
- [ ] Codex CLI full interop pass.
- [ ] PI agent full interop pass.
- [ ] `inspect_record`, namespace propagation, and structuredContent pass.

## Docs Checks

- [ ] README is current and does not claim stable has shipped.
- [ ] [docs/WIKI_INDEX.md](WIKI_INDEX.md) links every wiki page.
- [ ] [docs/RELEASE_STRATEGY.md](RELEASE_STRATEGY.md) is current.
- [ ] [docs/STABLE_V1_GATE.md](STABLE_V1_GATE.md) is current.
- [ ] [docs/wiki/13-Release-And-Deployment.md](wiki/13-Release-And-Deployment.md) is current.
- [ ] [docs/wiki/14-QA-And-Test-Matrix.md](wiki/14-QA-And-Test-Matrix.md) is current.

## Security Checks

```bash
grep -RInP "[\x{202A}-\x{202E}\x{2066}-\x{2069}\x{200B}\x{200C}\x{200D}\x{FEFF}]" README.md CHANGELOG.md RELEASE.md SECURITY.md docs examples .github Cargo.toml crates scripts schemas 2>/dev/null || true

grep -RInE "(api[_-]?key|secret|token|password|PRIVATE KEY|BEGIN RSA|BEGIN OPENSSH)" README.md CHANGELOG.md RELEASE.md SECURITY.md docs examples .github Cargo.toml crates scripts schemas 2>/dev/null || true

LOCAL_PATH_RE='(/home/[^/]+|/'''Users/|C:\\'''Users\\)'
grep -RInE "$LOCAL_PATH_RE" README.md CHANGELOG.md RELEASE.md SECURITY.md docs examples .github Cargo.toml crates scripts schemas 2>/dev/null || true
```

Expected: no hidden/bidi matches, no public-doc local path leakage, and only documented false positives for secret/token terminology.

## Artifact Checks

- [ ] Fresh clone builds and tests.
- [ ] Git archive includes README, docs, schemas, crates, examples, and `.github`.
- [ ] Git archive excludes `.pi`, `target`, `.env`, local exports/imports, download artifacts, and real user configs.

## Tagging Checks

- [ ] Stable version bump complete.
- [ ] Stable docs wording complete.
- [ ] `CHANGELOG.md` stable entry complete.
- [ ] `v1.0.0` tag not created until all gates pass.

## Post-release Checks

- [ ] GitHub release page matches tag and artifacts.
- [ ] Source archive verified.
- [ ] MCP client smoke tests rerun where practical.

## Rollback Checks

- [ ] Previous tag retained.
- [ ] Tags are not deleted.
- [ ] Known issue documented if stable issue is found.
- [ ] Patch release plan prepared if needed.

## Portable Workflow Parity

`v1.0.0` adds deterministic portable memory workflow parity: `memory-worth`, `capture`, `inbox`, `context`, `session add/search/decisions`, `recall-xray`, explicit L1/L2/L3 layers, trust class, durability, source kind, and minimal verification gates. Capture creates candidates or L3 evidence only; it does not silently apply durable L1/L2 memory. L1 is never auto-applied. L3 is session/evidence context, not authoritative memory.
