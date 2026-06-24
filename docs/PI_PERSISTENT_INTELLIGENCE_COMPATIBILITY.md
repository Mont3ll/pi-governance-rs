# pi-persistent-intelligence Compatibility

## Recommended Relationship

`pi-governance-rs` is the Rust local-first governance runtime. It provides the CLI, MCP server, source-of-truth JSONL memory store, and release-grade binary.

`pi-persistent-intelligence` is best suited as the JS/TS SDK and integration layer: app/framework adapters, dashboard or client candidates, memory capture middleware candidates, and eval/dogfood tooling.

## Ecosystem Split

```text
Rust PI = canonical governance engine
TS PI   = SDK, adapters, dashboard, web/product layer
```

## Future Compatibility Goals

- shared JSON schemas
- compatible import/export bundles
- policy profile parity
- namespace parity
- record/patch schema parity
- MCP client wrapper
- dashboard using Rust CLI/MCP outputs

This release-candidate sprint does not implement cross-repo functionality.
