use anyhow::{bail, Context, Result};
use pi_core::{EvidenceKind, EvidenceRef, RecordClass, Scope};
use pi_governance::{GovernanceEngine, ProposalInput};
use pi_retrieval::render_markdown;
use serde_json::{json, Map, Value};
use std::io::{self, BufRead, Write};
use std::str::FromStr;

const MCP_PROTOCOL_VERSION: &str = "2025-11-25";
const SERVER_NAME: &str = "pi-governance";
const SERVER_VERSION: &str = "0.2.0";

#[derive(Debug, Clone)]
pub struct McpStdioServer {
    engine: GovernanceEngine,
}

impl McpStdioServer {
    pub fn new(engine: GovernanceEngine) -> Self {
        Self { engine }
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

    fn tool_definitions(&self) -> Value {
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
                "name": "pi.doctor",
                "description": "Inspect PI store health, patch state, warnings, and governance errors.",
                "inputSchema": {
                    "type": "object",
                    "additionalProperties": false,
                    "properties": {}
                }
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
            }
        ])
    }

    fn handle_tool_call(&self, params: Value) -> Result<Value> {
        let name = required_string(&params, "name")?;
        let arguments = params.get("arguments").cloned().unwrap_or_else(|| json!({}));

        match name.as_str() {
            "pi.retrieve_context" => self.tool_retrieve_context(arguments),
            "pi.propose_record" => self.tool_propose_record(arguments),
            "pi.apply_patch" => self.tool_apply_patch(arguments),
            "pi.list_patches" => self.tool_list_patches(arguments),
            "pi.inspect_patch" => self.tool_inspect_patch(arguments),
            "pi.doctor" => self.tool_doctor(),
            "pi.list_records" => self.tool_list_records(arguments),
            other => bail!("unknown PI MCP tool: {other}"),
        }
    }

    fn tool_retrieve_context(&self, args: Value) -> Result<Value> {
        let query = required_string(&args, "query")?;
        let project = optional_string(&args, "project");
        let budget = optional_usize(&args, "budget").unwrap_or(1200);
        let format = optional_string(&args, "format").unwrap_or_else(|| "markdown".to_string());

        let bundle = self
            .engine
            .retrieve_context(query, project, budget)
            .context("failed to retrieve PI context")?;

        let text = match format.as_str() {
            "json" => serde_json::to_string_pretty(&bundle)?,
            "markdown" | "md" => render_markdown(&bundle),
            other => bail!("unsupported retrieve format: {other}"),
        };

        Ok(tool_result(text, serde_json::to_value(bundle)?))
    }

    fn tool_propose_record(&self, args: Value) -> Result<Value> {
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
        )?;

        let text = serde_json::to_string_pretty(&result)?;

        Ok(tool_result(text, serde_json::to_value(result)?))
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
        let patches = self.engine.list_patches(limit)?;
        let text = serde_json::to_string_pretty(&patches)?;

        Ok(tool_result(text, serde_json::to_value(patches)?))
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

    fn tool_doctor(&self) -> Result<Value> {
        let report = self.engine.doctor()?;
        let text = serde_json::to_string_pretty(&report)?;

        Ok(tool_result(text, serde_json::to_value(report)?))
    }

    fn tool_list_records(&self, args: Value) -> Result<Value> {
        let limit = optional_usize(&args, "limit").unwrap_or(20);
        let records = self.engine.list_records(limit)?;
        let text = serde_json::to_string_pretty(&records)?;

        Ok(tool_result(text, serde_json::to_value(records)?))
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
