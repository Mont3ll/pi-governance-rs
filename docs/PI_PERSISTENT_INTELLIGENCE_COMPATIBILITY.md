# pi-persistent-intelligence Compatibility

`pi-governance-rs` and `pi-persistent-intelligence` are standalone implementations of the shared PI memory model.

Use `pi-governance-rs` when you want governed memory through CLI or MCP across multiple agents.

Use `pi-persistent-intelligence` when you want the lightweight native memory extension inside PI agent.

Neither project requires the other.

## Shared memory contract

Both projects use the same core ideas:

- scoped memory records
- confidence and evidence metadata
- review before durable changes
- import/export formats for moving memory between tools

## Interoperability

Use export and import workflows to move reviewed memory between stores. Review imported records and patches before relying on them in another environment.

## Safety model

PI memory is designed to be explicit and auditable. Keep high-impact rules, corrections, and identity-level claims under review, and prefer evidence-backed records over unreviewed notes.
