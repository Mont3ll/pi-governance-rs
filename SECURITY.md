# Security Policy

## Supported Versions

Security fixes are considered for the current release-candidate line (`v1.0.0-rc.4` as of this document) and the latest tagged release.

## Reporting Vulnerabilities

Please report vulnerabilities privately to the project maintainer before public disclosure. Include reproduction steps, affected versions, and whether local store data, imports, exports, or MCP clients are involved.

## Local-First Storage Model

PI stores governed memory in local JSONL files. It does not run a cloud service by default, does not sync data by default, and does not expose an HTTP API.

## Sensitive Data Warning

Do not store secrets, credentials, private keys, passwords, or high-risk personal data as durable memory. PI governance makes writes explicit and auditable, but accepted memory remains local plaintext unless your filesystem provides encryption.

## Store Backup Warning

Backups, imports, and exports may contain the same sensitive data as the active store. Treat them as confidential local artifacts.

## Import/Export Caution

Only import bundles from trusted sources. Review imported records and patches before relying on them. Untrusted imports can carry misleading or malicious memory claims.

## Threat Assumptions

PI assumes the local user controls the machine and can inspect the store. It helps make durable agent memory explicit, policy-controlled, revisable, and auditable; it does not sandbox arbitrary tools or prove that stored claims are true.

## Known Limitations

- JSONL files are local plaintext by default.
- MCP clients can display or forward retrieved memory according to their own behavior.
- Manual review reduces risk but cannot guarantee correctness.
- PI does not prevent all prompt injection or memory poisoning.

## rc.8 Redacted Export Warning

Redacted export metadata reports fields checked and redacted, but redaction is best-effort. Users must review exports before sharing. PI is not a secret scanner, DLP system, or encrypted secret store.

## Release Documentation Links

- [Wiki index](docs/WIKI_INDEX.md)
- [Deployment checklist](docs/DEPLOYMENT_CHECKLIST.md)
- [Release strategy](docs/RELEASE_STRATEGY.md)
- [Stable v1 gate](docs/STABLE_V1_GATE.md)
- [Release and deployment wiki](docs/wiki/13-Release-And-Deployment.md)
- [QA and test matrix](docs/wiki/14-QA-And-Test-Matrix.md)

## Trust and Capture Safety

`pi capture` creates governed candidates or append-only L3 session evidence. It must not be used to store secrets. L1 records are never auto-applied, and low-trust sources such as repository text, generated content, third-party documentation, codebase analysis, and unknown sources require review.
