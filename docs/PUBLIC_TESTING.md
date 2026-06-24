# Public Testing Guide

## Who Should Test PI

PI is ready for controlled testing by coding-agent users, MCP users, developer-tool builders, people building long-running AI assistants, and security-conscious agent builders.

## What PI Is Trying to Validate

- Can a new user understand governed memory within 10 minutes?
- Does `pi demo` make the value clear?
- Does `pi review` make patch governance understandable?
- Does `pi inspect-record` make auditability clear?
- Is MCP setup understandable?
- Do docs explain what PI is not?

## Setup

```bash
git clone <repo-url>
cd pi-governance-rs
cargo build -p pi-cli
./target/debug/pi --version
```

## 10-Minute Test

```bash
cargo build -p pi-cli
./target/debug/pi demo --store /tmp/pi-public-test-demo --reset
./target/debug/pi --store /tmp/pi-public-test-demo review
./target/debug/pi --store /tmp/pi-public-test-demo retrieve "release workflow" --explain
./target/debug/pi --store /tmp/pi-public-test-demo list
./target/debug/pi --store /tmp/pi-public-test-demo inspect-record <record_id>
./target/debug/pi agent-instructions
```

Replace `<record_id>` with an ID from `pi list`.

## 30-Minute Test

1. Create a fresh store with `pi init`.
2. Propose a memory with `pi propose`.
3. Review it with `pi review`.
4. Retrieve it with `pi retrieve`.
5. Inspect it with `pi inspect-record`.
6. Contest it with `pi contest`.
7. Resolve the contest with `pi resolve-contest`.
8. Export it with `pi export`.
9. Dry-run import into a new store with `pi import --dry-run --json`.

## Coding-Agent MCP Test

Generate config with `pi mcp-config claude` or `pi mcp-config cursor`, then confirm `tools/list` contains PI tools and `pi.smoke_test` returns pass.

## Memory Governance Test

Check whether propose, review, apply, contest, supersede, tombstone, and resolve-contest make the memory lifecycle understandable.

## Record Inspection Test

Use `pi list` to find a record ID, then run `pi inspect-record <record_id>` and `pi inspect-record <record_id> --json`.

## What Not to Test Yet

Do not test capture/extraction, semantic retrieval, vector search, graph memory, dashboards, hosted services, cloud sync, or connectors. These are deferred.

## Known Limitations

PI uses local plaintext JSONL by default, does not prove memories are true, does not sandbox tools, and does not prevent all prompt injection or memory poisoning.

## Feedback Questions

- Did you understand what PI does?
- Which command confused you first?
- Did review/inspect make trust clearer?
- Would you let a coding agent propose memories?
- What felt too manual?
- What should be automated later?
- Would you use this with Claude, Cursor, or Codex?
- What should block stable v1.0.0?

## How to Report Issues

Use the GitHub issue templates for bugs, usability feedback, MCP setup issues, memory governance feedback, or docs feedback. Redact secrets and sensitive store contents.
