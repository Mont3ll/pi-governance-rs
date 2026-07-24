# Memory Poisoning and PI

PI reduces memory-poisoning risk by making durable memory writes explicit, inspectable, policy-controlled, revisable, and auditable. It does not prove that all accepted memories are true.

## What Memory Poisoning Is

Memory poisoning happens when an agent stores or later retrieves misleading, malicious, stale, or over-broad durable instructions. Persistent memory is risky because one bad write can influence future sessions.

## How PI Reduces Risk

PI uses governed patches instead of silent memory writes. Proposed memories can be inspected before application, policies can require manual review, and old memories can be contested, superseded, or tombstoned while preserving audit history.

Helpful controls include:

- strict policy for high-impact namespaces
- manual review with `pi review`
- contesting disputed records
- superseding stale records
- tombstoning invalid records
- namespace isolation for project boundaries
- import/export dry runs before merge

## What PI Does Not Solve

PI does not guarantee truth, detect every malicious claim, replace human review, sandbox tools, or prevent all prompt injection. Agents can still misuse retrieved context if their instructions are weak.

## Safe Use Guidance

Use strict policy for identity rules, release workflows, security guidance, and cross-project instructions. Review pending patches before release work. Treat contested records as unsafe unless explicitly included. Do not import untrusted bundles without review.

## Agent Instruction Risks

Agent instructions should say when to retrieve PI memory, how to treat contested records, and when to propose corrections. They should not tell agents to blindly apply every memory or to store secrets.

## an earlier release candidate Maintenance and Redaction Notes

`pi maintenance scan` is read-only and does not create suggestions or mutate memory. Redacted export is best-effort and does not replace user review. PI is not a secret scanner or DLP system; do not store secrets in PI.

## Release Documentation Links

- [Wiki index](WIKI_INDEX.md)
- [Deployment checklist](DEPLOYMENT_CHECKLIST.md)
- [Release strategy](RELEASE_STRATEGY.md)
- [Historical v1.0 gate](STABLE_V1_GATE.md)
- [Release and deployment wiki](wiki/13-Release-And-Deployment.md)
- [QA and test matrix](wiki/14-QA-And-Test-Matrix.md)
