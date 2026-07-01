# Security Policy

## Reporting Vulnerabilities

Please report vulnerabilities privately to the project maintainer before public disclosure.

Include:

- affected version
- reproduction steps
- whether local store data, imports, exports, or MCP clients are involved
- any logs or examples needed to understand the issue

## Local-First Storage Model

PI Governance stores governed memory in local JSONL files.

It does not run a cloud service by default, does not sync data by default, and does not expose an HTTP API.

## Sensitive Data

Do not store secrets, credentials, private keys, passwords, or high-risk personal data as durable memory.

PI Governance makes durable writes explicit and auditable, but accepted memory remains local plaintext unless your filesystem provides encryption.

## Backups, Imports, and Exports

Backups, imports, and exports can contain the same sensitive data as the active store. Treat them as confidential local artifacts.

Only import bundles from trusted sources. Review imported records and patches before relying on them.

## MCP Client Behavior

MCP clients may display, store, transform, or forward retrieved memory according to their own behavior. Review your MCP client configuration and data-handling settings before connecting it to a PI store.

## Threat Model

PI Governance assumes the local user controls the machine and can inspect the store.

It helps make durable agent memory explicit, policy-controlled, revisable, and auditable. It does not sandbox arbitrary tools, prove that stored claims are true, or prevent every form of prompt injection or memory poisoning.

## Known Limitations

- JSONL files are local plaintext by default.
- Manual review reduces risk but cannot guarantee correctness.
- Redacted exports are best-effort and should be reviewed before sharing.
- PI Governance is not a secret scanner, DLP system, encrypted vault, or sandbox.

## Trust and Capture Safety

`pi capture` creates governed candidates or append-only session evidence.

L1 records are never auto-applied. Low-trust sources such as repository text, generated content, third-party documentation, codebase analysis, and unknown sources require review.
