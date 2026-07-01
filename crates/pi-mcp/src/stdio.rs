use anyhow::{bail, Context, Result};
use pi_governance_core::{default_namespace, ContestResolution, Durability, EvidenceKind, EvidenceRef, MemoryLayer, PolicyProfile, RecordClass, RetrievalFormat, RetrievalOptions, Scope, SourceKind, TrustClass};
use pi_governance_engine::{
    build_context, claim_from_capture, evidence_for_capture, recall_xray, score_memory_worth, search_session_events, session_decisions, session_event, scope_for_project, verify_candidate,
    ContestInput, ExportInput, GovernanceEngine, ImportInput, MigrationInput, ProposalInput, ReinforceInput, ResolveContestInput,
    SupersedeInput, TombstoneInput, MemoryWorthDecision,
};
use pi_governance_retrieval::render_markdown;
use serde_json::{json, Map, Value};
use std::io::{self, BufRead, Write};
use std::str::FromStr;

const MCP_PROTOCOL_VERSION: &str = "2025-11-25";
const SERVER_NAME: &str = "pi-governance";
const SERVER_VERSION: &str = "1.0.2";

#[derive(Debug, Clone)]
pub struct McpStdioServer {
    engine: GovernanceEngine,
    default_namespace: String,
}

impl McpStdioServer {
    pub fn new(engine: GovernanceEngine) -> Self {
        Self::new_with_namespace(engine, default_namespace())
    }

    pub fn new_with_namespace(engine: GovernanceEngine, default_namespace: String) -> Self {
        Self { engine, default_namespace }
    }

    fn namespace_arg(&self, args: &Value) -> String {
        optional_string(args, "namespace").unwrap_or_else(|| self.default_namespace.clone())
    }

    pub fn run(&self) -> Result<()> {
        self.engine.init()?;

        let stdin = io::stdin();
        let mut stdout = io::stdout();

        for line in stdin.lock().lines() {
            let line = line.context("failed to read MCP stdin line")?;

            if line.trim().is_empty() {
                continue;
            }

            let response = match serde_json::from_str::<Value>(&line) {
                Ok(message) => self.handle_message(message),
                Err(error) => Some(error_response(
                    Value::Null,
                    -32700,
                    "parse error",
                    Some(json!({ "detail": error.to_string() })),
                )),
            };

            if let Some(response) = response {
                write_json_line(&mut stdout, &response)?;
            }
        }

        Ok(())
    }

    fn handle_message(&self, message: Value) -> Option<Value> {
        let id = message.get("id").cloned();
        let method = message
            .get("method")
            .and_then(Value::as_str)
            .unwrap_or_default();

        let params = message.get("params").cloned().unwrap_or_else(|| json!({}));

        match method {
            "initialize" => Some(success_response(
                id.unwrap_or(Value::Null),
                self.initialize_result(params),
            )),

            "notifications/initialized" => None,

            "ping" => Some(success_response(id.unwrap_or(Value::Null), json!({}))),

            "tools/list" => Some(success_response(
                id.unwrap_or(Value::Null),
                json!({
                    "tools": self.tool_definitions()
                }),
            )),

            "tools/call" => {
                let id = id.unwrap_or(Value::Null);

                match self.handle_tool_call(params) {
                    Ok(result) => Some(success_response(id, result)),
                    Err(error) => Some(error_response(
                        id,
                        -32000,
                        "tool execution failed",
                        Some(json!({ "detail": error.to_string() })),
                    )),
                }
            }

            "" => Some(error_response(
                id.unwrap_or(Value::Null),
                -32600,
                "invalid request: missing method",
                None,
            )),

            _ => Some(error_response(
                id.unwrap_or(Value::Null),
                -32601,
                "method not found",
                Some(json!({ "method": method })),
            )),
        }
    }

    fn initialize_result(&self, _params: Value) -> Value {
        let instructions = [
            "PI Governance exposes governed memory tools for coding agents.",
            "Use pi.retrieve_context before making project-sensitive changes.",
            "Use pi.propose_record for durable memory updates instead of directly mutating the store.",
            "Use pi.list_patches and pi.inspect_patch before applying queued patches.",
            "Use pi.supersede_record, pi.tombstone_record, and pi.reinforce_record for direct belief revision.",
            "Use pi.contest_record and pi.resolve_contest when a claim is disputed but not yet ready to supersede or tombstone.",
            "Use pi.export_store and pi.import_store to move governed memory between stores without direct file mutation.",
            "The JSONL store now uses a local store.lock file to serialize mutating operations.",
            "Identity-level records and risky mutations may require force/manual review.",
        ]
        .join("\n");

        json!({
            "protocolVersion": MCP_PROTOCOL_VERSION,
            "capabilities": {
                "tools": {
                    "listChanged": false
                }
            },
            "serverInfo": {
                "name": SERVER_NAME,
                "version": SERVER_VERSION
            },
            "instructions": instructions
        })
    }

    pub fn tool_definitions(&self) -> Value {
        json!([
            {
                "name": "pi.retrieve_context",
                "description": "Retrieve a governed, budgeted context bundle from PI memory.",
                "inputSchema": {
                    "type": "object",
                    "additionalProperties": false,
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "Search query or task description."
                        },
                        "project": {
                            "type": "string",
                            "description": "Optional project key. Global records are still eligible."
                        },
                        "namespace": {
                            "type": "string",
                            "default": "default"
                        },
                        "budget": {
                            "type": "integer",
                            "description": "Approximate token budget for returned context.",
                            "default": 1200,
                            "minimum": 100
                        },
                        "format": {
                            "type": "string",
                            "enum": ["json", "markdown"],
                            "default": "markdown"
                        },
                        "retriever": {
                            "type": "string",
                            "enum": ["deterministic", "lexical", "hybrid"],
                            "default": "deterministic"
                        },
                        "explain": {
                            "type": "boolean",
                            "default": false
                        },
                        "classes": {
                            "type": "array",
                            "items": { "type": "string" },
                            "default": []
                        },
                        "include_global": {
                            "type": "boolean",
                            "default": true
                        },
                        "include_contested": {
                            "type": "boolean",
                            "default": false
                        },
                        "min_confidence": {
                            "type": "number",
                            "minimum": 0,
                            "maximum": 1
                        }
                    },
                    "required": ["query"]
                }
            },
            {
                "name": "pi.propose_record",
                "description": "Propose a governed memory record. This queues a patch and optionally applies it if policy allows.",
                "inputSchema": {
                    "type": "object",
                    "additionalProperties": false,
                    "properties": {
                        "class": {
                            "type": "string",
                            "enum": [
                                "identity_rule",
                                "preference",
                                "project_state",
                                "requirement",
                                "correction",
                                "workflow",
                                "observation",
                                "evidence_note"
                            ]
                        },
                        "claim": {
                            "type": "string",
                            "description": "Durable claim to store."
                        },
                        "confidence": {
                            "type": "number",
                            "minimum": 0,
                            "maximum": 1,
                            "default": 0.70
                        },
                        "project": {
                            "type": "string",
                            "description": "Optional project scope key."
                        },
                        "tags": {
                            "type": "array",
                            "items": { "type": "string" },
                            "default": []
                        },
                        "evidence_uri": {
                            "type": "string",
                            "description": "Optional evidence pointer, e.g. conversation:2026-06-15 or file:path."
                        },
                        "evidence_kind": {
                            "type": "string",
                            "enum": [
                                "conversation",
                                "file",
                                "url",
                                "test",
                                "commit",
                                "user_correction",
                                "human_review"
                            ],
                            "default": "conversation"
                        },
                        "reason": {
                            "type": "string",
                            "description": "Why this patch is being proposed."
                        },
                        "apply": {
                            "type": "boolean",
                            "default": false
                        },
                        "force": {
                            "type": "boolean",
                            "description": "Allows manual-review patches to apply explicitly. Rejected patches still cannot apply.",
                            "default": false
                        }
                    },
                    "required": ["class", "claim"]
                }
            },
            {
                "name": "pi.supersede_record",
                "description": "Propose a supersession patch that replaces an active record with a new governed claim.",
                "inputSchema": {
                    "type": "object",
                    "additionalProperties": false,
                    "properties": {
                        "target_id": { "type": "string" },
                        "class": {
                            "type": "string",
                            "enum": [
                                "identity_rule",
                                "preference",
                                "project_state",
                                "requirement",
                                "correction",
                                "workflow",
                                "observation",
                                "evidence_note"
                            ]
                        },
                        "claim": { "type": "string" },
                        "confidence": {
                            "type": "number",
                            "minimum": 0,
                            "maximum": 1,
                            "default": 0.75
                        },
                        "project": { "type": "string" },
                        "tags": {
                            "type": "array",
                            "items": { "type": "string" },
                            "default": []
                        },
                        "evidence_uri": { "type": "string" },
                        "evidence_kind": {
                            "type": "string",
                            "enum": [
                                "conversation",
                                "file",
                                "url",
                                "test",
                                "commit",
                                "user_correction",
                                "human_review"
                            ],
                            "default": "conversation"
                        },
                        "reason": { "type": "string" },
                        "apply": { "type": "boolean", "default": false },
                        "force": { "type": "boolean", "default": false }
                    },
                    "required": ["target_id", "class", "claim", "reason"]
                }
            },
            {
                "name": "pi.tombstone_record",
                "description": "Propose a tombstone patch for an active record while preserving audit history.",
                "inputSchema": {
                    "type": "object",
                    "additionalProperties": false,
                    "properties": {
                        "target_id": { "type": "string" },
                        "evidence_uri": { "type": "string" },
                        "evidence_kind": {
                            "type": "string",
                            "enum": [
                                "conversation",
                                "file",
                                "url",
                                "test",
                                "commit",
                                "user_correction",
                                "human_review"
                            ],
                            "default": "conversation"
                        },
                        "reason": { "type": "string" },
                        "apply": { "type": "boolean", "default": false },
                        "force": { "type": "boolean", "default": false }
                    },
                    "required": ["target_id", "reason"]
                }
            },
            {
                "name": "pi.reinforce_record",
                "description": "Propose a reinforcement patch that adds evidence and increases confidence for an active record.",
                "inputSchema": {
                    "type": "object",
                    "additionalProperties": false,
                    "properties": {
                        "target_id": { "type": "string" },
                        "evidence_uri": { "type": "string" },
                        "evidence_kind": {
                            "type": "string",
                            "enum": [
                                "conversation",
                                "file",
                                "url",
                                "test",
                                "commit",
                                "user_correction",
                                "human_review"
                            ],
                            "default": "conversation"
                        },
                        "reason": {
                            "type": "string",
                            "default": "reinforce record with new evidence"
                        },
                        "apply": { "type": "boolean", "default": false },
                        "force": { "type": "boolean", "default": false }
                    },
                    "required": ["target_id", "evidence_uri"]
                }
            },
            {
                "name": "pi.contest_record",
                "description": "Contest an active or already contested record with evidence while preserving audit history.",
                "inputSchema": {
                    "type": "object",
                    "additionalProperties": false,
                    "properties": {
                        "target_id": { "type": "string" },
                        "evidence_uri": { "type": "string" },
                        "evidence_kind": {
                            "type": "string",
                            "enum": [
                                "conversation",
                                "file",
                                "url",
                                "test",
                                "commit",
                                "user_correction",
                                "human_review"
                            ],
                            "default": "conversation"
                        },
                        "reason": { "type": "string" },
                        "apply": { "type": "boolean", "default": false },
                        "force": { "type": "boolean", "default": false }
                    },
                    "required": ["target_id", "evidence_uri", "reason"]
                }
            },
            {
                "name": "pi.resolve_contest",
                "description": "Resolve a contested record by upholding, tombstoning, or superseding it.",
                "inputSchema": {
                    "type": "object",
                    "additionalProperties": false,
                    "properties": {
                        "target_id": { "type": "string" },
                        "resolution": {
                            "type": "string",
                            "enum": ["uphold", "tombstone", "supersede"]
                        },
                        "class": {
                            "type": "string",
                            "enum": [
                                "identity_rule",
                                "preference",
                                "project_state",
                                "requirement",
                                "correction",
                                "workflow",
                                "observation",
                                "evidence_note"
                            ]
                        },
                        "claim": { "type": "string" },
                        "confidence": {
                            "type": "number",
                            "minimum": 0,
                            "maximum": 1,
                            "default": 0.75
                        },
                        "project": { "type": "string" },
                        "tags": {
                            "type": "array",
                            "items": { "type": "string" },
                            "default": []
                        },
                        "evidence_uri": { "type": "string" },
                        "evidence_kind": {
                            "type": "string",
                            "enum": [
                                "conversation",
                                "file",
                                "url",
                                "test",
                                "commit",
                                "user_correction",
                                "human_review"
                            ],
                            "default": "conversation"
                        },
                        "reason": { "type": "string" },
                        "apply": { "type": "boolean", "default": false },
                        "force": { "type": "boolean", "default": false }
                    },
                    "required": ["target_id", "resolution", "reason"]
                }
            },
            {
                "name": "pi.apply_patch",
                "description": "Apply an existing proposed PI patch by ID.",
                "inputSchema": {
                    "type": "object",
                    "additionalProperties": false,
                    "properties": {
                        "patch_id": {
                            "type": "string"
                        },
                        "force": {
                            "type": "boolean",
                            "default": false
                        }
                    },
                    "required": ["patch_id"]
                }
            },
            {
                "name": "pi.list_patches",
                "description": "List latest patch state, one row per patch id.",
                "inputSchema": {
                    "type": "object",
                    "additionalProperties": false,
                    "properties": {
                        "limit": {
                            "type": "integer",
                            "minimum": 1,
                            "maximum": 200,
                            "default": 20
                        }
                    }
                }
            },
            {
                "name": "pi.inspect_patch",
                "description": "Inspect full patch history and whether the latest version can be applied.",
                "inputSchema": {
                    "type": "object",
                    "additionalProperties": false,
                    "properties": {
                        "patch_id": {
                            "type": "string"
                        }
                    },
                    "required": ["patch_id"]
                }
            },

            {
                "name": "pi.export_store",
                "description": "Export the PI store as a portable JSON bundle. Can optionally filter by project or redact evidence/event details.",
                "inputSchema": {
                    "type": "object",
                    "additionalProperties": false,
                    "properties": {
                        "project": {
                            "type": "string",
                            "description": "Optional project filter. Global records are included with matching project records."
                        },
                        "redacted": {
                            "type": "boolean",
                            "default": false
                        }
                    }
                }
            },
            {
                "name": "pi.import_store",
                "description": "Import a portable PI JSON bundle from a local path. Defaults to dry-run for safety and skips duplicate ids.",
                "inputSchema": {
                    "type": "object",
                    "additionalProperties": false,
                    "properties": {
                        "path": { "type": "string" },
                        "dry_run": {
                            "type": "boolean",
                            "default": true
                        },
                        "backup": {
                            "type": "boolean",
                            "default": true
                        }
                    },
                    "required": ["path"]
                }
            },
            {
                "name": "pi.config_show",
                "description": "Show PI JSON config.",
                "inputSchema": { "type": "object", "additionalProperties": false, "properties": {} }
            },
            {
                "name": "pi.config_set_policy",
                "description": "Set namespace policy profile.",
                "inputSchema": {
                    "type": "object", "additionalProperties": false,
                    "properties": {
                        "namespace": { "type": "string" },
                        "policy": { "type": "string", "enum": ["permissive", "standard", "strict"] }
                    },
                    "required": ["namespace", "policy"]
                }
            },
            {
                "name": "pi.policy_doctor",
                "description": "Inspect PI policy config.",
                "inputSchema": { "type": "object", "additionalProperties": false, "properties": {} }
            },
            {
                "name": "pi.policy_explain",
                "description": "Explain policy treatment for an operation.",
                "inputSchema": {
                    "type": "object", "additionalProperties": false,
                    "properties": { "operation": { "type": "string", "enum": ["propose", "reinforce", "supersede", "tombstone", "contest", "resolve-contest", "import"] } },
                    "required": ["operation"]
                }
            },
            {
                "name": "pi.smoke_test",
                "description": "Run PI smoke tests against a temporary store.",
                "inputSchema": { "type": "object", "additionalProperties": false, "properties": { "json": { "type": "boolean", "default": true } } }
            },
            {
                "name": "pi.mcp_config",
                "description": "Generate MCP client config.",
                "inputSchema": { "type": "object", "additionalProperties": false, "properties": { "client": { "type": "string", "enum": ["claude", "cursor", "inspector"] } }, "required": ["client"] }
            },
            {
                "name": "pi.changelog",
                "description": "Show PI changelog.",
                "inputSchema": { "type": "object", "additionalProperties": false, "properties": {} }
            },
            {
                "name": "pi.migrate_schema",
                "description": "Migrate legacy JSONL entries to the current PI schema version. Defaults to dry-run for safety.",
                "inputSchema": {
                    "type": "object",
                    "additionalProperties": false,
                    "properties": {
                        "dry_run": {
                            "type": "boolean",
                            "default": true
                        },
                        "backup": {
                            "type": "boolean",
                            "default": true
                        }
                    }
                }
            },
            {
                "name": "pi.doctor",
                "description": "Inspect PI store health, patch state, warnings, and governance errors.",
                "inputSchema": {
                    "type": "object",
                    "additionalProperties": false,
                    "properties": {}
                }
            },
            {
                "name": "pi.list_namespaces",
                "description": "List PI namespace summaries.",
                "inputSchema": {
                    "type": "object",
                    "additionalProperties": false,
                    "properties": {}
                }
            },
            {
                "name": "pi.namespace_doctor",
                "description": "Inspect PI namespace isolation health.",
                "inputSchema": {
                    "type": "object",
                    "additionalProperties": false,
                    "properties": {}
                }
            },
            {
                "name": "pi.inspect_record",
                "description": "Inspect a governed memory record by id.",
                "inputSchema": {
                    "type": "object",
                    "additionalProperties": false,
                    "properties": { "record_id": { "type": "string" }, "namespace": { "type": "string" } },
                    "required": ["record_id"]
                }
            },
            {
                "name": "pi.maintenance_scan",
                "description": "Run a read-only governance maintenance scan.",
                "inputSchema": { "type": "object", "additionalProperties": false, "properties": { "namespace": { "type": "string" } } }
            },
            {
                "name": "pi.reject_patch",
                "description": "Reject a proposed or deferred patch without mutating records.",
                "inputSchema": { "type": "object", "additionalProperties": false, "properties": { "patch_id": { "type": "string" }, "reason": { "type": "string" }, "namespace": { "type": "string" } }, "required": ["patch_id", "reason"] }
            },
            {
                "name": "pi.defer_patch",
                "description": "Defer a proposed patch without mutating records.",
                "inputSchema": { "type": "object", "additionalProperties": false, "properties": { "patch_id": { "type": "string" }, "reason": { "type": "string" }, "namespace": { "type": "string" } }, "required": ["patch_id", "reason"] }
            },
            {
                "name": "pi.list_records",
                "description": "List recent PI records for inspection.",
                "inputSchema": {
                    "type": "object",
                    "additionalProperties": false,
                    "properties": {
                        "limit": {
                            "type": "integer",
                            "minimum": 1,
                            "maximum": 200,
                            "default": 20
                        }
                    }
                }
            },
            {"name":"pi.score_memory_worth","description":"Score whether text should become governed memory.","inputSchema":{"type":"object","additionalProperties":false,"properties":{"observation":{"type":"string"},"project":{"type":"string"},"namespace":{"type":"string"},"trust_class":{"type":"string"},"source_kind":{"type":"string"}},"required":["observation"]}},
            {"name":"pi.capture_candidates","description":"Capture deterministic memory candidates without auto-applying durable memory.","inputSchema":{"type":"object","additionalProperties":false,"properties":{"text":{"type":"string"},"project":{"type":"string"},"target":{"type":"string"},"layer":{"type":"string"},"trust_class":{"type":"string"}},"required":["text"]}},
            {"name":"pi.build_context","description":"Build scoped paste-ready governed memory context.","inputSchema":{"type":"object","additionalProperties":false,"properties":{"query":{"type":"string"},"project":{"type":"string"},"format":{"type":"string","default":"markdown"},"budget":{"type":"integer","default":1200},"include_l3":{"type":"boolean"},"include_contested":{"type":"boolean"}},"required":["query"]}},
            {"name":"pi.session_add","description":"Append L3 session evidence.","inputSchema":{"type":"object","additionalProperties":false,"properties":{"text":{"type":"string"},"project":{"type":"string"}},"required":["text"]}},
            {"name":"pi.session_search","description":"Search L3 session evidence lexically.","inputSchema":{"type":"object","additionalProperties":false,"properties":{"query":{"type":"string"},"project":{"type":"string"}},"required":["query"]}},
            {"name":"pi.session_decisions","description":"List extracted session decision markers.","inputSchema":{"type":"object","additionalProperties":false,"properties":{"project":{"type":"string"},"days":{"type":"integer"}}}},
            {"name":"pi.recall_xray","description":"Explain recall inclusion and exclusion.","inputSchema":{"type":"object","additionalProperties":false,"properties":{"query":{"type":"string"},"project":{"type":"string"},"budget":{"type":"integer","default":1200},"include_l3":{"type":"boolean"},"include_contested":{"type":"boolean"}},"required":["query"]}},
            {"name":"pi.list_inbox","description":"List proposed/deferred candidate patches.","inputSchema":{"type":"object","additionalProperties":false,"properties":{"all":{"type":"boolean"}}}}
        ])
    }

    fn handle_tool_call(&self, params: Value) -> Result<Value> {
        let name = required_string(&params, "name")?;
        let arguments = params.get("arguments").cloned().unwrap_or_else(|| json!({}));

        match name.as_str() {
            "pi.retrieve_context" => self.tool_retrieve_context(arguments),
            "pi.propose_record" => self.tool_propose_record(arguments),
            "pi.supersede_record" => self.tool_supersede_record(arguments),
            "pi.tombstone_record" => self.tool_tombstone_record(arguments),
            "pi.reinforce_record" => self.tool_reinforce_record(arguments),
            "pi.contest_record" => self.tool_contest_record(arguments),
            "pi.resolve_contest" => self.tool_resolve_contest(arguments),
            "pi.apply_patch" => self.tool_apply_patch(arguments),
            "pi.reject_patch" => self.tool_reject_patch(arguments),
            "pi.defer_patch" => self.tool_defer_patch(arguments),
            "pi.list_patches" => self.tool_list_patches(arguments),
            "pi.inspect_patch" => self.tool_inspect_patch(arguments),
            "pi.export_store" => self.tool_export_store(arguments),
            "pi.import_store" => self.tool_import_store(arguments),
            "pi.migrate_schema" => self.tool_migrate_schema(arguments),
            "pi.doctor" => self.tool_doctor(),
            "pi.config_show" => self.tool_config_show(),
            "pi.config_set_policy" => self.tool_config_set_policy(arguments),
            "pi.policy_doctor" => self.tool_policy_doctor(),
            "pi.policy_explain" => self.tool_policy_explain(arguments),
            "pi.smoke_test" => self.tool_smoke_test(arguments),
            "pi.mcp_config" => self.tool_mcp_config(arguments),
            "pi.changelog" => self.tool_changelog(),
            "pi.list_namespaces" => self.tool_list_namespaces(),
            "pi.namespace_doctor" => self.tool_namespace_doctor(),
            "pi.inspect_record" => self.tool_inspect_record(arguments),
            "pi.maintenance_scan" => self.tool_maintenance_scan(arguments),
            "pi.list_records" => self.tool_list_records(arguments),
            "pi.score_memory_worth" => self.tool_score_memory_worth(arguments),
            "pi.capture_candidates" => self.tool_capture_candidates(arguments),
            "pi.build_context" => self.tool_build_context(arguments),
            "pi.session_add" => self.tool_session_add(arguments),
            "pi.session_search" => self.tool_session_search(arguments),
            "pi.session_decisions" => self.tool_session_decisions(arguments),
            "pi.recall_xray" => self.tool_recall_xray(arguments),
            "pi.list_inbox" => self.tool_list_inbox(arguments),
            other => bail!("unknown PI MCP tool: {other}"),
        }
    }

    fn tool_score_memory_worth(&self, args: Value) -> Result<Value> {
        let observation = required_string(&args, "observation")?;
        let trust = optional_string(&args, "trust_class").and_then(|s| TrustClass::from_str(&s).ok());
        let source = optional_string(&args, "source_kind").and_then(|s| SourceKind::from_str(&s).ok()).or(Some(SourceKind::ManualMcp));
        let report = score_memory_worth(&observation, trust, source);
        Ok(tool_result(serde_json::to_string_pretty(&report)?, serde_json::to_value(report)?))
    }

    fn tool_capture_candidates(&self, args: Value) -> Result<Value> {
        let namespace = self.namespace_arg(&args);
        let text = required_string(&args, "text")?;
        let project = optional_string(&args, "project");
        let layer = optional_string(&args, "layer").and_then(|s| MemoryLayer::from_str(&s).ok());
        let trust = optional_string(&args, "trust_class").and_then(|s| TrustClass::from_str(&s).ok());
        let worth = score_memory_worth(&text, trust, Some(SourceKind::ManualMcp));
        let mut report = pi_governance_engine::CaptureReport { input_summary: text.chars().take(80).collect(), candidates: Vec::new(), daily_only: Vec::new(), inquiries: Vec::new(), rejected: Vec::new(), applied: false };
        match worth.decision {
            MemoryWorthDecision::Reject => report.rejected.push(text.clone()),
            MemoryWorthDecision::DailyOnly => { let event = session_event(&namespace, project.as_deref(), &text, SourceKind::ManualMcp); self.engine.store().append_event(&event)?; report.daily_only.push(text.clone()); }
            MemoryWorthDecision::Inquiry => report.inquiries.push(text.clone()),
            MemoryWorthDecision::Candidate => {
                let claim = claim_from_capture(&text);
                let suggested_layer = layer.unwrap_or(worth.suggested_layer);
                let verification = verify_candidate(&claim, suggested_layer, worth.trust_class, worth.durability);
                let result = self.engine.propose_record(ProposalInput { namespace: namespace.clone(), class: worth.suggested_class.clone(), claim: claim.clone(), confidence: worth.confidence, scope: scope_for_project(project), tags: worth.suggested_tags.clone(), evidence_refs: vec![evidence_for_capture(SourceKind::ManualMcp, worth.trust_class, worth.durability)], reason: Some("captured deterministic memory candidate".to_string()), layer: Some(suggested_layer), memory_kind: Some(worth.suggested_memory_kind), rule_type: worth.suggested_rule_type, trust_class: worth.trust_class, durability: worth.durability, source_kind: SourceKind::ManualMcp }, false, false)?;
                report.candidates.push(pi_governance_engine::CaptureCandidate { claim, decision: worth.decision, patch_id: Some(result.patch_id), suggested_layer, trust_class: worth.trust_class, durability: worth.durability, memory_kind: worth.suggested_memory_kind, rule_type: worth.suggested_rule_type, verification });
            }
        }
        Ok(tool_result(serde_json::to_string_pretty(&report)?, serde_json::to_value(report)?))
    }

    fn tool_build_context(&self, args: Value) -> Result<Value> {
        let query = required_string(&args, "query")?;
        let project = optional_string(&args, "project");
        let budget = optional_usize(&args, "budget").unwrap_or(1200);
        let include_l3 = optional_bool(&args, "include_l3").unwrap_or(false);
        let include_contested = optional_bool(&args, "include_contested").unwrap_or(false);
        let format = optional_string(&args, "format").unwrap_or_else(|| "markdown".to_string());
        let (markdown, value) = build_context(self.engine.store(), &self.default_namespace, &query, project, budget, include_l3, include_contested)?;
        let text = if format == "json" { serde_json::to_string_pretty(&value)? } else { markdown };
        Ok(tool_result(text, value))
    }

    fn tool_session_add(&self, args: Value) -> Result<Value> {
        let namespace = self.namespace_arg(&args);
        let text = required_string(&args, "text")?;
        let project = optional_string(&args, "project");
        let event = session_event(&namespace, project.as_deref(), &text, SourceKind::ManualMcp);
        self.engine.store().append_event(&event)?;
        Ok(tool_result(serde_json::to_string_pretty(&event)?, serde_json::to_value(event)?))
    }

    fn tool_session_search(&self, args: Value) -> Result<Value> {
        let namespace = self.namespace_arg(&args);
        let query = required_string(&args, "query")?;
        let project = optional_string(&args, "project");
        let events = search_session_events(self.engine.store(), &namespace, &query, project.as_deref(), None)?;
        let value = json!({"events": events});
        Ok(tool_result(serde_json::to_string_pretty(&value)?, value))
    }

    fn tool_session_decisions(&self, args: Value) -> Result<Value> {
        let namespace = self.namespace_arg(&args);
        let project = optional_string(&args, "project");
        let days = args.get("days").and_then(Value::as_i64);
        let decisions = session_decisions(self.engine.store(), &namespace, project.as_deref(), days)?;
        let value = json!({"decisions": decisions});
        Ok(tool_result(serde_json::to_string_pretty(&value)?, value))
    }

    fn tool_recall_xray(&self, args: Value) -> Result<Value> {
        let namespace = self.namespace_arg(&args);
        let query = required_string(&args, "query")?;
        let project = optional_string(&args, "project");
        let budget = optional_usize(&args, "budget").unwrap_or(1200);
        let include_l3 = optional_bool(&args, "include_l3").unwrap_or(false);
        let include_contested = optional_bool(&args, "include_contested").unwrap_or(false);
        let report = recall_xray(self.engine.store(), &namespace, &query, project, budget, include_l3, include_contested)?;
        Ok(tool_result(serde_json::to_string_pretty(&report)?, serde_json::to_value(report)?))
    }

    fn tool_list_inbox(&self, args: Value) -> Result<Value> {
        let all = optional_bool(&args, "all").unwrap_or(false);
        let mut rows = Vec::new();
        for p in self.engine.list_patches(200)? {
            if !all && !matches!(p.latest_status, pi_governance_core::PatchStatus::Proposed | pi_governance_core::PatchStatus::Deferred) { continue; }
            rows.push(p);
        }
        let value = json!({"pending_count": rows.len(), "patches": rows});
        Ok(tool_result(serde_json::to_string_pretty(&value)?, value))
    }

    fn tool_config_show(&self) -> Result<Value> {
        let config = self.engine.config()?;
        let text = serde_json::to_string_pretty(&config)?;
        Ok(tool_result(text, serde_json::to_value(config)?))
    }

    fn tool_config_set_policy(&self, args: Value) -> Result<Value> {
        let namespace = required_string(&args, "namespace")?;
        let policy_raw = required_string(&args, "policy")?;
        let policy = PolicyProfile::from_str(&policy_raw).map_err(anyhow::Error::msg)?;
        let config = self.engine.set_policy(&namespace, policy)?;
        let text = serde_json::to_string_pretty(&config)?;
        Ok(tool_result(text, serde_json::to_value(config)?))
    }

    fn tool_policy_doctor(&self) -> Result<Value> { self.tool_config_show() }

    fn tool_policy_explain(&self, args: Value) -> Result<Value> {
        let operation = required_string(&args, "operation")?;
        let text = GovernanceEngine::policy_explain(&operation);
        Ok(tool_result(text.clone(), json!({"operation": operation, "explanation": text})))
    }

    fn tool_smoke_test(&self, _args: Value) -> Result<Value> {
        let report = GovernanceEngine::run_smoke_test();
        let text = serde_json::to_string_pretty(&report)?;
        Ok(tool_result(text, serde_json::to_value(report)?))
    }

    fn tool_mcp_config(&self, args: Value) -> Result<Value> {
        let client = required_string(&args, "client")?;
        let command = std::env::current_exe()?.display().to_string();
        let value = if client == "inspector" {
            json!({"command": format!("npx @modelcontextprotocol/inspector {} mcp-stdio", command)})
        } else {
            json!({"mcpServers": {"pi-governance": {"command": command, "args": ["mcp-stdio"]}}})
        };
        let text = serde_json::to_string_pretty(&value)?;
        Ok(tool_result(text, value))
    }

    fn tool_changelog(&self) -> Result<Value> {
        let text = include_str!("../CHANGELOG.md").to_string();
        Ok(tool_result(text.clone(), json!({"changelog": text})))
    }

    fn tool_list_namespaces(&self) -> Result<Value> {
        let summaries = self.engine.namespace_summaries()?;
        let text = serde_json::to_string_pretty(&summaries)?;
        let count = summaries.len();
        Ok(tool_result(text, json!({ "namespaces": summaries, "count": count })))
    }

    fn tool_namespace_doctor(&self) -> Result<Value> {
        let report = self.engine.namespace_doctor()?;
        let text = serde_json::to_string_pretty(&report)?;
        Ok(tool_result(text, serde_json::to_value(report)?))
    }

    fn tool_retrieve_context(&self, args: Value) -> Result<Value> {
        let query = required_string(&args, "query")?;
        let namespace = self.namespace_arg(&args);
        let project = optional_string(&args, "project");
        let budget = optional_usize(&args, "budget").unwrap_or(1200);
        let format = optional_string(&args, "format").unwrap_or_else(|| "markdown".to_string());
        let retriever = optional_string(&args, "retriever").unwrap_or_else(|| "deterministic".to_string());
        let explain = optional_bool(&args, "explain").unwrap_or(false);
        let include_global = optional_bool(&args, "include_global").unwrap_or(true);
        let include_contested = optional_bool(&args, "include_contested").unwrap_or(false);
        let min_confidence = optional_f32(&args, "min_confidence");
        let classes = optional_string_array(&args, "classes")?
            .unwrap_or_default()
            .into_iter()
            .map(|raw| RecordClass::from_str(&raw).map_err(anyhow::Error::msg))
            .collect::<Result<Vec<_>>>()?;
        let retrieval_format = match format.as_str() {
            "json" => RetrievalFormat::Json,
            "markdown" | "md" => RetrievalFormat::Markdown,
            other => bail!("unsupported retrieve format: {other}"),
        };

        let bundle = self
            .engine
            .retrieve_context_with_options(RetrievalOptions {
                query,
                retriever: retriever.clone(),
                namespace,
                project,
                budget,
                format: retrieval_format.clone(),
                explain,
                classes,
                include_global,
                include_contested,
                min_confidence,
            })
            .context("failed to retrieve PI context")?;

        let text = match retrieval_format {
            RetrievalFormat::Json => serde_json::to_string_pretty(&bundle)?,
            RetrievalFormat::Markdown => render_markdown(&bundle),
        };

        Ok(tool_result(text, serde_json::to_value(bundle)?))
    }

    fn tool_propose_record(&self, args: Value) -> Result<Value> {
        let namespace = self.namespace_arg(&args);
        let class_raw = required_string(&args, "class")?;
        let class = RecordClass::from_str(&class_raw).map_err(anyhow::Error::msg)?;

        let claim = required_string(&args, "claim")?;
        let confidence = optional_f32(&args, "confidence").unwrap_or(0.70);
        let project = optional_string(&args, "project");
        let tags = optional_string_array(&args, "tags")?.unwrap_or_default();
        let reason = optional_string(&args, "reason");
        let apply = optional_bool(&args, "apply").unwrap_or(false);
        let force = optional_bool(&args, "force").unwrap_or(false);

        let evidence_refs = match optional_string(&args, "evidence_uri") {
            Some(uri) => {
                let evidence_kind_raw = optional_string(&args, "evidence_kind")
                    .unwrap_or_else(|| "conversation".to_string());

                let evidence_kind =
                    EvidenceKind::from_str(&evidence_kind_raw).map_err(anyhow::Error::msg)?;

                vec![EvidenceRef::new(evidence_kind, uri)]
            }
            None => Vec::new(),
        };

        let scope = match project {
            Some(project_key) => Scope::project(project_key),
            None => Scope::global(),
        };

        let result = self.engine.propose_record(
            ProposalInput {
                namespace,
                class,
                claim,
                confidence,
                scope,
                tags,
                evidence_refs,
                reason,
                layer: None,
                memory_kind: None,
                rule_type: None,
                trust_class: TrustClass::DirectUserInstruction,
                durability: Durability::Project,
                source_kind: SourceKind::ManualMcp,
            },
            apply,
            force,
        )?;

        let text = serde_json::to_string_pretty(&result)?;

        Ok(tool_result(text, serde_json::to_value(result)?))
    }

    fn tool_supersede_record(&self, args: Value) -> Result<Value> {
        let namespace = self.namespace_arg(&args);
        let target_id = required_string(&args, "target_id")?;
        let class_raw = required_string(&args, "class")?;
        let class = RecordClass::from_str(&class_raw).map_err(anyhow::Error::msg)?;
        let claim = required_string(&args, "claim")?;
        let confidence = optional_f32(&args, "confidence").unwrap_or(0.75);
        let project = optional_string(&args, "project");
        let tags = optional_string_array(&args, "tags")?.unwrap_or_default();
        let reason = required_string(&args, "reason")?;
        let apply = optional_bool(&args, "apply").unwrap_or(false);
        let force = optional_bool(&args, "force").unwrap_or(false);

        let evidence_refs = match optional_string(&args, "evidence_uri") {
            Some(uri) => {
                let evidence_kind_raw = optional_string(&args, "evidence_kind")
                    .unwrap_or_else(|| "conversation".to_string());
                let evidence_kind =
                    EvidenceKind::from_str(&evidence_kind_raw).map_err(anyhow::Error::msg)?;
                vec![EvidenceRef::new(evidence_kind, uri)]
            }
            None => Vec::new(),
        };

        let scope = match project {
            Some(project_key) => Scope::project(project_key),
            None => Scope::global(),
        };

        match self.engine.supersede_record(
            SupersedeInput {
                namespace,
                target_id: target_id.clone(),
                class,
                claim,
                confidence,
                scope,
                tags,
                evidence_refs,
                reason,
            },
            apply,
            force,
        ) {
            Ok(result) => {
                let text = serde_json::to_string_pretty(&result)?;
                Ok(tool_result(text, serde_json::to_value(result)?))
            }
            Err(error) => Ok(tool_error(
                error.to_string(),
                json!({
                    "code": "supersede_record_failed",
                    "target_id": target_id
                }),
            )),
        }
    }

    fn tool_tombstone_record(&self, args: Value) -> Result<Value> {
        let namespace = self.namespace_arg(&args);
        let target_id = required_string(&args, "target_id")?;
        let reason = required_string(&args, "reason")?;
        let apply = optional_bool(&args, "apply").unwrap_or(false);
        let force = optional_bool(&args, "force").unwrap_or(false);

        let evidence_refs = match optional_string(&args, "evidence_uri") {
            Some(uri) => {
                let evidence_kind_raw = optional_string(&args, "evidence_kind")
                    .unwrap_or_else(|| "conversation".to_string());
                let evidence_kind =
                    EvidenceKind::from_str(&evidence_kind_raw).map_err(anyhow::Error::msg)?;
                vec![EvidenceRef::new(evidence_kind, uri)]
            }
            None => Vec::new(),
        };

        match self.engine.tombstone_record(
            TombstoneInput {
                namespace,
                target_id: target_id.clone(),
                evidence_refs,
                reason,
            },
            apply,
            force,
        ) {
            Ok(result) => {
                let text = serde_json::to_string_pretty(&result)?;
                Ok(tool_result(text, serde_json::to_value(result)?))
            }
            Err(error) => Ok(tool_error(
                error.to_string(),
                json!({
                    "code": "tombstone_record_failed",
                    "target_id": target_id
                }),
            )),
        }
    }

    fn tool_reinforce_record(&self, args: Value) -> Result<Value> {
        let namespace = self.namespace_arg(&args);
        let target_id = required_string(&args, "target_id")?;
        let evidence_uri = required_string(&args, "evidence_uri")?;
        let evidence_kind_raw =
            optional_string(&args, "evidence_kind").unwrap_or_else(|| "conversation".to_string());
        let evidence_kind = EvidenceKind::from_str(&evidence_kind_raw).map_err(anyhow::Error::msg)?;
        let reason = optional_string(&args, "reason")
            .unwrap_or_else(|| "reinforce record with new evidence".to_string());
        let outcome = optional_string(&args, "outcome").unwrap_or_else(|| "explicit_reinforcement".to_string());
        let apply = optional_bool(&args, "apply").unwrap_or(false);
        let force = optional_bool(&args, "force").unwrap_or(false);

        match self.engine.reinforce_record(
            ReinforceInput {
                namespace,
                target_id: target_id.clone(),
                evidence_refs: vec![EvidenceRef::new(evidence_kind, evidence_uri)],
                reason: format!("{} (outcome: {})", reason, outcome),
            },
            apply,
            force,
        ) {
            Ok(result) => {
                let text = serde_json::to_string_pretty(&result)?;
                Ok(tool_result(text, serde_json::to_value(result)?))
            }
            Err(error) => Ok(tool_error(
                error.to_string(),
                json!({
                    "code": "reinforce_record_failed",
                    "target_id": target_id
                }),
            )),
        }
    }

    fn tool_contest_record(&self, args: Value) -> Result<Value> {
        let namespace = self.namespace_arg(&args);
        let target_id = required_string(&args, "target_id")?;
        let evidence_uri = required_string(&args, "evidence_uri")?;
        let evidence_kind_raw =
            optional_string(&args, "evidence_kind").unwrap_or_else(|| "conversation".to_string());
        let evidence_kind = EvidenceKind::from_str(&evidence_kind_raw).map_err(anyhow::Error::msg)?;
        let reason = required_string(&args, "reason")?;
        let apply = optional_bool(&args, "apply").unwrap_or(false);
        let force = optional_bool(&args, "force").unwrap_or(false);

        match self.engine.contest_record(
            ContestInput {
                namespace,
                target_id: target_id.clone(),
                evidence_refs: vec![EvidenceRef::new(evidence_kind, evidence_uri)],
                reason,
            },
            apply,
            force,
        ) {
            Ok(result) => {
                let text = serde_json::to_string_pretty(&result)?;
                Ok(tool_result(text, serde_json::to_value(result)?))
            }
            Err(error) => Ok(tool_error(
                error.to_string(),
                json!({
                    "code": "contest_record_failed",
                    "target_id": target_id
                }),
            )),
        }
    }

    fn tool_resolve_contest(&self, args: Value) -> Result<Value> {
        let namespace = self.namespace_arg(&args);
        let target_id = required_string(&args, "target_id")?;
        let resolution_raw = required_string(&args, "resolution")?;
        let resolution =
            ContestResolution::from_str(&resolution_raw).map_err(anyhow::Error::msg)?;
        let class = match optional_string(&args, "class") {
            Some(raw) => Some(RecordClass::from_str(&raw).map_err(anyhow::Error::msg)?),
            None => None,
        };
        let claim = optional_string(&args, "claim");
        let confidence = optional_f32(&args, "confidence").unwrap_or(0.75);
        let project = optional_string(&args, "project");
        let tags = optional_string_array(&args, "tags")?.unwrap_or_default();
        let reason = required_string(&args, "reason")?;
        let apply = optional_bool(&args, "apply").unwrap_or(false);
        let force = optional_bool(&args, "force").unwrap_or(false);

        let evidence_refs = match optional_string(&args, "evidence_uri") {
            Some(uri) => {
                let evidence_kind_raw = optional_string(&args, "evidence_kind")
                    .unwrap_or_else(|| "conversation".to_string());
                let evidence_kind =
                    EvidenceKind::from_str(&evidence_kind_raw).map_err(anyhow::Error::msg)?;
                vec![EvidenceRef::new(evidence_kind, uri)]
            }
            None => Vec::new(),
        };

        let scope = match project {
            Some(project_key) => Scope::project(project_key),
            None => Scope::global(),
        };

        match self.engine.resolve_contest(
            ResolveContestInput {
                namespace,
                target_id: target_id.clone(),
                resolution,
                class,
                claim,
                confidence,
                scope,
                tags,
                evidence_refs,
                reason,
            },
            apply,
            force,
        ) {
            Ok(result) => {
                let text = serde_json::to_string_pretty(&result)?;
                Ok(tool_result(text, serde_json::to_value(result)?))
            }
            Err(error) => Ok(tool_error(
                error.to_string(),
                json!({
                    "code": "resolve_contest_failed",
                    "target_id": target_id,
                    "resolution": resolution_raw
                }),
            )),
        }
    }

    fn tool_reject_patch(&self, args: Value) -> Result<Value> {
        let patch_id = required_string(&args, "patch_id")?;
        let reason = required_string(&args, "reason")?;
        let namespace = self.namespace_arg(&args);
        match self.engine.reject_patch_by_id(&patch_id, &namespace, &reason) {
            Ok(result) => Ok(tool_result(serde_json::to_string_pretty(&result)?, serde_json::to_value(result)?)),
            Err(error) => Ok(tool_error(error.to_string(), json!({"code":"reject_patch_failed","patch_id":patch_id}))),
        }
    }

    fn tool_defer_patch(&self, args: Value) -> Result<Value> {
        let patch_id = required_string(&args, "patch_id")?;
        let reason = required_string(&args, "reason")?;
        let namespace = self.namespace_arg(&args);
        match self.engine.defer_patch_by_id(&patch_id, &namespace, &reason) {
            Ok(result) => Ok(tool_result(serde_json::to_string_pretty(&result)?, serde_json::to_value(result)?)),
            Err(error) => Ok(tool_error(error.to_string(), json!({"code":"defer_patch_failed","patch_id":patch_id}))),
        }
    }

    fn tool_apply_patch(&self, args: Value) -> Result<Value> {
        let patch_id = required_string(&args, "patch_id")?;
        let force = optional_bool(&args, "force").unwrap_or(false);

        match self.engine.apply_patch_by_id(&patch_id, force) {
            Ok(result) => {
                let text = serde_json::to_string_pretty(&result)?;
                Ok(tool_result(text, serde_json::to_value(result)?))
            }
            Err(error) => Ok(tool_error(
                error.to_string(),
                json!({
                    "code": "apply_patch_failed",
                    "patch_id": patch_id
                }),
            )),
        }
    }

    fn tool_list_patches(&self, args: Value) -> Result<Value> {
        let limit = optional_usize(&args, "limit").unwrap_or(20);
        let namespace = self.namespace_arg(&args);
        let patches = self.engine.list_patches_in_namespace(&namespace, limit)?;
        let text = serde_json::to_string_pretty(&patches)?;
        let count = patches.len();

        Ok(tool_result(text, json!({ "patches": patches, "count": count })))
    }

    fn tool_inspect_patch(&self, args: Value) -> Result<Value> {
        let patch_id = required_string(&args, "patch_id")?;

        match self.engine.inspect_patch(&patch_id) {
            Ok(inspection) => {
                let text = serde_json::to_string_pretty(&inspection)?;
                Ok(tool_result(text, serde_json::to_value(inspection)?))
            }
            Err(error) => Ok(tool_error(
                error.to_string(),
                json!({
                    "code": "inspect_patch_failed",
                    "patch_id": patch_id
                }),
            )),
        }
    }


    fn tool_export_store(&self, args: Value) -> Result<Value> {
        let namespace = Some(self.namespace_arg(&args));
        let all_namespaces = optional_bool(&args, "all_namespaces").unwrap_or(false);
        let project = optional_string(&args, "project");
        let redacted = optional_bool(&args, "redacted").unwrap_or(false);

        match self.engine.export_store(ExportInput { namespace, all_namespaces, project, redacted }) {
            Ok(bundle) => {
                let text = serde_json::to_string_pretty(&bundle)?;
                Ok(tool_result(text, serde_json::to_value(bundle)?))
            }
            Err(error) => Ok(tool_error(
                error.to_string(),
                json!({
                    "code": "export_store_failed"
                }),
            )),
        }
    }

    fn tool_import_store(&self, args: Value) -> Result<Value> {
        let path = required_string(&args, "path")?;
        let namespace = self.namespace_arg(&args);
        let preserve_namespaces = optional_bool(&args, "preserve_namespaces").unwrap_or(false);
        let dry_run = optional_bool(&args, "dry_run").unwrap_or(true);
        let backup = optional_bool(&args, "backup").unwrap_or(true);

        match self.engine.import_store_from_path(
            std::path::Path::new(&path),
            ImportInput { namespace, preserve_namespaces, dry_run, backup },
        ) {
            Ok(report) => {
                let text = serde_json::to_string_pretty(&report)?;
                Ok(tool_result(text, serde_json::to_value(report)?))
            }
            Err(error) => Ok(tool_error(
                error.to_string(),
                json!({
                    "code": "import_store_failed",
                    "path": path
                }),
            )),
        }
    }

    fn tool_migrate_schema(&self, args: Value) -> Result<Value> {
        let dry_run = optional_bool(&args, "dry_run").unwrap_or(true);
        let backup = optional_bool(&args, "backup").unwrap_or(true);

        let report = self.engine.migrate_store(MigrationInput { dry_run, backup })?;
        let text = serde_json::to_string_pretty(&report)?;

        Ok(tool_result(text, serde_json::to_value(report)?))
    }

    fn tool_inspect_record(&self, args: Value) -> Result<Value> {
        let record_id = required_string(&args, "record_id")?;
        let namespace = self.namespace_arg(&args);
        match self.engine.inspect_record_in_namespace(&namespace, &record_id)? {
            Some(inspection) => {
                let record = inspection.record;
                let value = json!({
                    "record": record,
                    "evidence": record.evidence,
                    "related_patches": inspection.related_patches,
                    "audit": { "supersedes": inspection.revision.supersedes, "superseded_by": inspection.revision.superseded_by, "contested": inspection.revision.contested, "tombstoned": inspection.revision.tombstoned }
                });
                Ok(tool_result(serde_json::to_string_pretty(&value)?, value))
            }
            None => Ok(tool_error(format!("record not found in namespace {namespace}: {record_id}"), json!({"code":"record_not_found","record_id":record_id,"namespace":namespace}))),
        }
    }

    fn tool_maintenance_scan(&self, args: Value) -> Result<Value> {
        let namespace = self.namespace_arg(&args);
        let report = self.engine.maintenance_scan(&namespace)?;
        Ok(tool_result(serde_json::to_string_pretty(&report)?, serde_json::to_value(report)?))
    }

    fn tool_doctor(&self) -> Result<Value> {
        let report = self.engine.doctor_in_namespace(&self.default_namespace)?;
        let text = serde_json::to_string_pretty(&report)?;

        Ok(tool_result(text, serde_json::to_value(report)?))
    }

    fn tool_list_records(&self, args: Value) -> Result<Value> {
        let limit = optional_usize(&args, "limit").unwrap_or(20);
        let namespace = self.namespace_arg(&args);
        let records = self.engine.list_records_in_namespace(&namespace, limit)?;
        let text = serde_json::to_string_pretty(&records)?;
        let count = records.len();

        Ok(tool_result(text, json!({ "records": records, "count": count })))
    }
}

fn tool_result(text: String, structured_content: Value) -> Value {
    json!({
        "content": [
            {
                "type": "text",
                "text": text
            }
        ],
        "structuredContent": structured_content,
        "isError": false
    })
}

fn tool_error(text: String, structured_content: Value) -> Value {
    json!({
        "content": [
            {
                "type": "text",
                "text": text
            }
        ],
        "structuredContent": structured_content,
        "isError": true
    })
}

fn success_response(id: Value, result: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result
    })
}

fn error_response(id: Value, code: i64, message: impl Into<String>, data: Option<Value>) -> Value {
    let mut error = json!({
        "code": code,
        "message": message.into()
    });

    if let Some(data) = data {
        error["data"] = data;
    }

    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": error
    })
}

fn write_json_line<W: Write>(writer: &mut W, value: &Value) -> Result<()> {
    let line = serde_json::to_string(value)?;
    writeln!(writer, "{line}")?;
    writer.flush()?;
    Ok(())
}

fn args_object(value: &Value) -> Result<&Map<String, Value>> {
    value
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("arguments must be a JSON object"))
}

fn required_string(value: &Value, key: &str) -> Result<String> {
    let object = args_object(value)?;

    object
        .get(key)
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .ok_or_else(|| anyhow::anyhow!("missing required string argument: {key}"))
}

fn optional_string(value: &Value, key: &str) -> Option<String> {
    value
        .as_object()
        .and_then(|object| object.get(key))
        .and_then(Value::as_str)
        .map(ToString::to_string)
}

fn optional_bool(value: &Value, key: &str) -> Option<bool> {
    value
        .as_object()
        .and_then(|object| object.get(key))
        .and_then(Value::as_bool)
}

fn optional_usize(value: &Value, key: &str) -> Option<usize> {
    value
        .as_object()
        .and_then(|object| object.get(key))
        .and_then(Value::as_u64)
        .map(|number| number as usize)
}

fn optional_f32(value: &Value, key: &str) -> Option<f32> {
    value
        .as_object()
        .and_then(|object| object.get(key))
        .and_then(Value::as_f64)
        .map(|number| number as f32)
}

fn optional_string_array(value: &Value, key: &str) -> Result<Option<Vec<String>>> {
    let Some(raw) = value.as_object().and_then(|object| object.get(key)) else {
        return Ok(None);
    };

    let array = raw
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("{key} must be an array of strings"))?;

    let mut output = Vec::with_capacity(array.len());

    for item in array {
        let Some(text) = item.as_str() else {
            bail!("{key} must contain only strings");
        };

        output.push(text.to_string());
    }

    Ok(Some(output))
}
