# QA and Test Matrix

| Area | Command/Test | Expected Result | an earlier release candidate Status | Stable Gate |
| --- | --- | --- | --- | --- |
| cargo check | `cargo check --workspace` | pass | pass | required |
| cargo test | `cargo test --workspace` | pass | pass | required |
| cargo build | `cargo build -p pi-governance-rs` | pass | pass | required |
| version | `pi --version` | `pi 1.0.0` | pass | must show `pi 1.0.0` |
| demo | `pi demo` | demo store created | pass | required |
| review | `pi review` | queue visible | pass | required |
| review --apply | review action | patch applied | pass | required |
| review --reject | review action | patch rejected | pass | required |
| review --defer | review action | patch deferred | pass | required |
| list-patches | CLI/MCP | latest patches listed | pass | required |
| inspect-patch | CLI/MCP | patch history visible | pass | required |
| list | CLI/MCP | records listed | pass | required |
| inspect-record | CLI/MCP | record visible | pass | required |
| retrieve deterministic | CLI | local results | pass | required |
| retrieve lexical | CLI | local lexical results | pass | required |
| retrieve hybrid | CLI | local hybrid results | pass | required |
| empty retrieval diagnostics | `--explain` | useful diagnostics | pass | required |
| maintenance scan | `pi maintenance scan` | findings summary | pass | required |
| doctor | `pi doctor` | health report | pass | required |
| smoke-test | `pi smoke-test` | Result: pass | pass | required |
| release-audit | `pi release-audit` | Result: pass | pass | required |
| export --redacted | CLI | redaction metadata | pass | required |
| schemas validate | docs/schemas | valid schemas | pass | required |
| mcp-config opencode | CLI | JSON config | pass | required |
| mcp-config codex | CLI | config output | pass | required |
| mcp-config pi-agent | CLI | JSON config | pass | required |
| mcp-install temp config | CLI | merge/preview safe | pass | required |
| mcp-doctor temp config | CLI | direct stdio ok | pass | required |
| OpenCode install/doctor + documented live limitation | client test | setup checks complete | documented exception | accepted for v1.0.0 |
| Codex full interop | client test | pass | pass | required |
| PI agent full interop | client test | pass | pass | required |
| OpenCode inspect-record micro-test | client test | pass in prior an earlier release candidate live validation | not rerun due client-run limitation | historical evidence |
| Codex inspect-record micro-test | client test | pass | pass | required |
| hidden/bidi scan | grep | no matches | pass | required |
| secret/path scan | grep | no real secrets, no local paths | pass | required |
| fresh clone | clone/build/test | pass | pass | required |
| archive verification | `git archive` listing | includes docs, excludes local artifacts | pass | required |


---

Related: [Wiki index](../WIKI_INDEX.md), [Deployment checklist](../DEPLOYMENT_CHECKLIST.md), [Release strategy](../RELEASE_STRATEGY.md), [Stable v1 gate](../STABLE_V1_GATE.md).

## Portable Workflow Parity

`v1.0.0` adds deterministic portable memory workflow parity: `memory-worth`, `capture`, `inbox`, `context`, `session add/search/decisions`, `recall-xray`, explicit L1/L2/L3 layers, trust class, durability, source kind, and minimal verification gates. Capture creates candidates or L3 evidence only; it does not silently apply durable L1/L2 memory. L1 is never auto-applied. L3 is session/evidence context, not authoritative memory.


## Distribution and MCP Sharing

Repository: https://github.com/Mont3ll/pi-governance-rs
License: MIT OR Apache-2.0

Install from source with `cargo build -p pi-governance-rs`, from Git with `cargo install --git https://github.com/Mont3ll/pi-governance-rs --tag v1.1.0 pi-governance-rs`, or from crates.io with `cargo install pi-governance-rs`. `pi-governance-rs` is a standalone local stdio MCP server by default; it does not provide a hosted service. It remains compatible with `pi-persistent-intelligence` through the shared PI memory contract.
