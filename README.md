# pi-governance-rs

Portable Rust MVP for a PI-style governance layer usable by coding agents.

Current milestone: `0.2.0`.

This workspace includes:

- `pi-core`: record, patch, evidence, policy, and context types.
- `pi-store`: append-only JSONL persistence with atomic record rewrite.
- `pi-retrieval`: simple deterministic lexical retrieval and context rendering.
- `pi-governance`: policy-enforced proposal, application, retrieval, patch inspection, and doctor engine.
- `pi-mcp`: stdio JSON-RPC MCP-style adapter exposing PI tools.
- `pi-cli`: command-line binary for agents, scripts, and local testing.

## Build

```bash
cargo build -p pi-cli
```

## Run

Initialize a local store:

```bash
cargo run -p pi-cli -- --store .pi init
```

Propose and immediately apply a record:

```bash
cargo run -p pi-cli -- --store .pi propose \
  --class preference \
  --claim "User prefers exact React preview fidelity over reinterpretation." \
  --project figma-landing \
  --tag react \
  --tag fidelity \
  --evidence-uri conversation:2026-06-15 \
  --apply
```

Retrieve context:

```bash
cargo run -p pi-cli -- --store .pi retrieve \
  "React preview fidelity requirements" \
  --project figma-landing \
  --budget 900
```

Create a pending patch without applying it:

```bash
cargo run -p pi-cli -- --store .pi propose \
  --class requirement \
  --claim "Patch visibility must expose pending, applied, and rejected patch states." \
  --project pi-governance-rs \
  --tag patch-visibility \
  --evidence-uri conversation:v0.2.0
```

List patch state:

```bash
cargo run -p pi-cli -- --store .pi list-patches
```

Inspect a patch:

```bash
cargo run -p pi-cli -- --store .pi inspect-patch <patch_id>
```

Apply a pending patch:

```bash
cargo run -p pi-cli -- --store .pi apply <patch_id>
```

Run MCP stdio mode:

```bash
cargo run -p pi-cli -- --store .pi mcp-stdio
```

For MCP clients, prefer the compiled binary directly:

```bash
./target/debug/pi --store /absolute/path/to/pi-governance-rs/.pi mcp-stdio
```

## MCP tools

- `pi.retrieve_context`
- `pi.propose_record`
- `pi.apply_patch`
- `pi.list_patches`
- `pi.inspect_patch`
- `pi.doctor`
- `pi.list_records`

## v0.2.0 changes

- Adds patch summaries and patch inspection.
- Adds CLI commands: `list-patches`, `inspect-patch`.
- Adds MCP tools: `pi.list_patches`, `pi.inspect_patch`.
- Makes `apply_patch` check latest patch status instead of any historical proposed entry.
- Returns structured tool errors for expected MCP apply/inspect failures.
- Updates package and server version to `0.2.0`.

## Status

This is still a portable porting skeleton, not the full PI system. It is intentionally minimal and inspectable so additional PI features can be migrated module by module.
