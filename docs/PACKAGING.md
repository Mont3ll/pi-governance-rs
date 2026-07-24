# Packaging

Repository: https://github.com/Mont3ll/pi-governance-rs
License: MIT OR Apache-2.0

## Cargo package layout

This repository is a Cargo workspace. The binary crate is `crates/pi-cli`, the package name is `pi-governance-rs`, and the installed binary name is `pi`.

Workspace packages:

- `pi-governance-core`
- `pi-governance-store`
- `pi-governance-retrieval`
- `pi-governance-engine`
- `pi-governance-mcp`
- `pi-governance-rs`

## Binary crate

`crates/pi-cli` remains the binary crate directory and produces the public `pi` binary. It depends on the internal workspace crates above.

## cargo install

From a local source checkout:

```bash
cargo install --path crates/pi-cli --force
pi --version
```

## cargo install --git

```bash
cargo install --git https://github.com/Mont3ll/pi-governance-rs --tag v1.1.0 pi-governance-rs
pi --version
```

## crates.io package

Once crates.io publishing is explicitly approved, the intended public command is:

```bash
cargo install pi-governance-rs
```

Because `pi-governance-rs` depends on workspace crates, crates.io publishing must publish the supporting crates first. Current intended order:

1. `pi-governance-core`
2. `pi-governance-store`
3. `pi-governance-retrieval`
4. `pi-governance-engine`
5. `pi-governance-mcp`
6. `pi-governance-rs`

Do not publish without explicit approval.

## Package preview

Run package checks before any publish:

```bash
cargo package -p pi-governance-core --list
cargo package -p pi-governance-core
cargo publish -p pi-governance-core
```

Repeat for dependent crates after each upstream package is available on crates.io.

## GitHub release binaries

GitHub releases may include source archives, a relative patch, checksums, and optional compiled platform binaries. Cross-platform binaries are optional and should only be produced when toolchains are configured.

## Archive verification

Verify source archives with a fresh extraction, `cargo check --workspace`, `cargo test --workspace`, `cargo build -p pi-governance-rs`, `pi smoke-test`, and `pi release-audit`.

## Included and excluded files

Packages should include Rust sources, tests, `Cargo.toml`, `Cargo.lock` where Cargo includes it for binaries, README text, schemas, examples, documentation, and license files. Local stores, `.pi/`, `target/`, `.env`, private config, and download artifacts must remain excluded.


Release lineage: v1.0.0 was the first stable GitHub/source release. v1.0.3 completed the initial crates.io publication. v1.1.0 is the current coordinated workspace release.
