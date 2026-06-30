# Publishing Checklist

Repository: https://github.com/Mont3ll/pi-governance-rs
License: MIT OR Apache-2.0

Do not publish remotely until explicitly approved.

- [ ] License check: `LICENSE`, `LICENSE-MIT`, `LICENSE-APACHE` present.
- [ ] Cargo metadata check: version, license, repository, description, readme, keywords, categories.
- [ ] `cargo package --list` for each publishable package.
- [ ] `cargo package` for each package when dependencies are available.
- [ ] `cargo publish --dry-run` for each package when dependencies are available.
- [ ] Local `cargo install --path crates/pi-cli` test.
- [ ] Git tag `cargo install --git https://github.com/Mont3ll/pi-governance-rs --tag v1.0.0 pi-cli` test.
- [ ] Installed-binary MCP config and `tools/list` test.
- [ ] Fresh clone test.
- [ ] Archive verification.
- [ ] Hidden/bidi scan.
- [ ] Secret/path scan.
- [ ] Docs scan for local paths and placeholder URLs.
- [ ] Misleading dependency scan.
- [ ] Manual publish approval.
- [ ] Publish supporting crates in order if approved.
- [ ] Post-publish `cargo install pi-cli` verification.
- [ ] MCP Registry submission approval before submission.
