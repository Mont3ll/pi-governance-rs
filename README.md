# pi-governance-rs

Portable Rust MVP for a PI-style governance layer usable by coding agents.

This workspace includes:

- `pi-core`: record, patch, evidence, policy, and context types.
- `pi-store`: append-only JSONL persistence with atomic record rewrite.
- `pi-retrieval`: simple deterministic lexical retrieval and context rendering.
- `pi-governance`: policy-enforced proposal, application, retrieval, and doctor engine.
- `pi-mcp`: stdio JSON-RPC MCP-style adapter exposing PI tools.
- `pi-cli`: command-line binary for agents, scripts, and local testing.

## Run

```bash
cargo run -p pi-cli -- --store .pi init
```

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

```bash
cargo run -p pi-cli -- --store .pi retrieve \
  "React preview fidelity requirements" \
  --project figma-landing \
  --budget 900
```

```bash
cargo run -p pi-cli -- --store .pi mcp-stdio
```

## Status

This is the first portable porting skeleton, not the full PI system. It is intentionally minimal and inspectable so additional PI features can be migrated module by module.
