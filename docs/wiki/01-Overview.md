# Overview

PI governs what agents are allowed to believe over time.

## Problem Statement

AI agents can accumulate working assumptions, corrections, preferences, and operational facts. Without governance, those memories can become stale, poisoned, unsupported, or silently rewritten.

## Why Governed Memory Matters

Governed memory makes durable beliefs inspectable. PI requires proposed mutations, evidence references, audit history, review actions, and reversible status changes before local memory becomes a long-lived source of context.

## What PI Provides

PI provides a local JSONL store, governed records, patch queues, namespaces, policy profiles, local retrieval, MCP tools, maintenance scans, export/import, redaction metadata, smoke tests, and release audits.

## Complements, Not Replacements

PI complements RAG, GraphRAG, codebase-memory MCP servers, and skill libraries. RAG finds documents; GraphRAG models relationships; codebase-memory indexes repositories; skill libraries teach procedures. PI governs durable beliefs and proposed memory changes.

## What PI Avoids

PI deliberately avoids becoming a hosted memory service, vector database, GraphRAG engine, agent framework, dashboard product, secret vault, or DLP system.


---

Related: [Wiki index](../WIKI_INDEX.md), [Deployment checklist](../DEPLOYMENT_CHECKLIST.md), [Release strategy](../RELEASE_STRATEGY.md), [Stable v1 gate](../STABLE_V1_GATE.md).

## Portable Workflow Parity

`v1.0.0` adds deterministic portable memory workflow parity: `memory-worth`, `capture`, `inbox`, `context`, `session add/search/decisions`, `recall-xray`, explicit L1/L2/L3 layers, trust class, durability, source kind, and minimal verification gates. Capture creates candidates or L3 evidence only; it does not silently apply durable L1/L2 memory. L1 is never auto-applied. L3 is session/evidence context, not authoritative memory.


## Distribution and MCP Sharing

Repository: https://github.com/Mont3ll/pi-governance-rs
License: MIT OR Apache-2.0

Install from source with `cargo build -p pi-cli`, from Git with `cargo install --git https://github.com/Mont3ll/pi-governance-rs --tag v1.0.0 pi-cli`, or from crates.io with `cargo install pi-cli` after crates.io publishing is explicitly approved. `pi-governance-rs` is a standalone local stdio MCP server by default; it does not provide a hosted service in v1.0.0. It remains compatible with `pi-persistent-intelligence` through the shared PI memory contract.
