# Installation

PI Governance is distributed as the `pi-governance-rs` crate. It installs a binary named `pi`.

## Install from crates.io

```bash
cargo install pi-governance-rs
pi --version
```

## Build from source

```bash
git clone https://github.com/Mont3ll/pi-governance-rs.git
cd pi-governance-rs
cargo build -p pi-governance-rs
./target/debug/pi --version
```

## Create a demo store

```bash
pi demo --store .pi --reset
pi --store .pi doctor
pi --store .pi retrieve "release workflow" --explain
```

## Configure an MCP client

Print a client configuration and check it with the MCP doctor:

```bash
pi mcp-config codex --command "$(which pi)" --store /path/to/.pi --namespace default
pi mcp-doctor codex --command "$(which pi)" --store /path/to/.pi --namespace default
```

## Troubleshooting PATH issues

If your shell cannot find `pi`, confirm Cargo's bin directory is on your `PATH`:

```bash
echo "$PATH"
```

Cargo usually installs binaries to `$HOME/.cargo/bin`.
