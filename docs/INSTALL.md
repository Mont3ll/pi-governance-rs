# Installation

Repository: https://github.com/Mont3ll/pi-governance-rs

License: MIT OR Apache-2.0

`pi-governance-rs` installs a local-first governed memory CLI and MCP stdio server for AI agents. It does not run a hosted service by default.

## Install from source

```bash
git clone https://github.com/Mont3ll/pi-governance-rs
cd pi-governance-rs
cargo build -p pi-governance-rs
./target/debug/pi --version
```

Expected version:

```text
pi 1.0.2
```

## Install from Git tag

```bash
cargo install --git https://github.com/Mont3ll/pi-governance-rs --tag v1.0.2 pi-governance-rs
pi --version
```

## Install from crates.io once published

```bash
cargo install pi-governance-rs
pi --version
```

Note: crates.io publishing may still be pending until explicitly published.

## Verify version

```bash
pi --version
```

## Create a demo store

```bash
pi demo --store /tmp/pi-demo-store --reset
pi --store /tmp/pi-demo-store doctor
```

## Configure an MCP client

The MCP client launches the local `pi` binary as a subprocess. Use a real installed binary path in place of `/path/to/pi`.

```bash
pi mcp-config opencode --command /path/to/pi --store /path/to/.pi --namespace default
pi mcp-config codex --command /path/to/pi --store /path/to/.pi --namespace default
pi mcp-config pi-agent --command /path/to/pi --store /path/to/.pi --namespace default
```

To install into supported client config files, first dry-run, then opt in:

```bash
pi mcp-install opencode --command /path/to/pi --store /path/to/.pi --namespace default --dry-run
pi mcp-install opencode --command /path/to/pi --store /path/to/.pi --namespace default --yes
pi mcp-doctor opencode --command /path/to/pi --store /path/to/.pi --namespace default
```

## Troubleshooting PATH issues

If `pi` is not found after `cargo install`, add Cargo's binary directory to your shell PATH or call the installed binary by absolute path. To avoid ambiguity in MCP client configs, prefer an absolute command path such as `/path/to/pi`.


Release lineage: v1.0.0 was the first GitHub/source release. v1.0.1 prepared package identity but crates.io publication was partial. v1.0.2 completes crates.io publishing and is the public crates.io install target.
