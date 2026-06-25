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
