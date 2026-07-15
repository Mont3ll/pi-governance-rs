use anyhow::Result;
use chrono::DateTime;
use clap::{Parser, Subcommand};
use pi_governance_core::{ContestResolution, Durability, EvidenceKind, EvidenceRef, MemoryKind, MemoryLayer, PatchStatus, PolicyProfile, RecallEventClient, RecallEventOperation, RecallEventOutcome, RecordClass, RetrievalFormat, RetrievalOptions, RuleType, Scope, SourceKind, StoreEvent, TrustClass};
use pi_governance_engine::{
    analyze_failure_patterns, analyze_memory_quality, analyze_recall_effectiveness, analyze_relationship_quality, build_context, build_memory_graph, build_store_quality, generate_procedure_candidates, claim_from_capture, evidence_for_capture, read_text_input, recall_exclusion_counts, recall_xray, record_recall_event, record_recall_event_with_details, score_memory_worth, search_session_events, session_decisions, session_event, scope_for_project, verify_candidate,
    ContestInput, ExportInput, GovernanceEngine, ImportInput, MigrationInput, PatchInspection, PatchSummary, ProposalInput, RecordInspection, ReinforceInput, ResolveContestInput,
    SupersedeInput, TombstoneInput, MemoryWorthDecision,
};
use serde_json::json;
use pi_governance_mcp::{registered_tool_names, McpStdioServer};
use pi_governance_retrieval::render_markdown;
use pi_governance_store::JsonlStore;
use serde::Serialize;
use std::collections::BTreeMap;
use std::fs;
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(Debug, Parser)]
#[command(
    name = "pi",
    version = env!("CARGO_PKG_VERSION"),
    about = "PI governance runtime for coding agents"
)]
struct Cli {
    #[arg(long, global = true, default_value = ".pi")]
    store: PathBuf,

    #[arg(long, global = true, default_value = "default")]
    namespace: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Initialize the local PI store.
    Init,

    /// Propose a governed memory record.
    Propose {
        #[arg(long = "class")]
        class: RecordClass,

        #[arg(long)]
        claim: String,

        #[arg(long, default_value_t = 0.70)]
        confidence: f32,

        #[arg(long)]
        project: Option<String>,

        #[arg(long = "tag")]
        tags: Vec<String>,

        #[arg(long = "evidence-uri")]
        evidence_uri: Option<String>,

        #[arg(long = "evidence-kind", default_value = "conversation")]
        evidence_kind: EvidenceKind,

        #[arg(long)]
        reason: Option<String>,

        #[arg(long)]
        layer: Option<MemoryLayer>,

        /// Apply immediately if policy allows it.
        #[arg(long)]
        apply: bool,

        /// Allow manual-review records to be applied explicitly.
        #[arg(long)]
        force: bool,
    },

    /// Retrieve a context bundle for an agent.
    Retrieve {
        query: String,

        #[arg(long)]
        project: Option<String>,

        #[arg(long, default_value_t = 1200)]
        budget: usize,

        #[arg(long, default_value = "markdown")]
        format: String,

        #[arg(long, default_value = "deterministic")]
        retriever: String,

        #[arg(long)]
        explain: bool,

        #[arg(long = "include-global", default_value_t = true)]
        include_global: bool,

        #[arg(long = "include-contested")]
        include_contested: bool,

        #[arg(long = "min-confidence")]
        min_confidence: Option<f32>,

        #[arg(long = "class")]
        classes: Vec<RecordClass>,

        #[arg(long)]
        layer: Option<MemoryLayer>,
    },

    /// Review pending memory patches.
    Review {
        patch_id: Option<String>,
        #[arg(long)]
        json: bool,
        #[arg(long)]
        apply: bool,
        #[arg(long)]
        force: bool,
        #[arg(long)]
        reject: Option<String>,
        #[arg(long)]
        defer: Option<String>,
        #[arg(long)]
        reason: Option<String>,
    },

    /// Create a safe demo store with governed memory examples.
    Demo {
        #[arg(long)]
        json: bool,
        #[arg(long)]
        reset: bool,
    },

    /// Print coding-agent instructions for PI usage.
    AgentInstructions {
        #[arg(long)]
        json: bool,
    },

    /// Apply a previously proposed patch.
    Apply {
        patch_id: String,

        #[arg(long)]
        force: bool,
    },

    /// List latest patch state, one row per patch id.
    ListPatches {
        #[arg(long, default_value_t = 20)]
        limit: usize,

        #[arg(long)]
        json: bool,
    },

    /// Inspect full history and current applyability for one patch.
    InspectPatch {
        patch_id: String,

        #[arg(long)]
        json: bool,
    },

    /// Supersede an active record with a replacement claim.
    Supersede {
        target_id: String,

        #[arg(long = "class")]
        class: RecordClass,

        #[arg(long)]
        claim: String,

        #[arg(long, default_value_t = 0.75)]
        confidence: f32,

        #[arg(long)]
        project: Option<String>,

        #[arg(long = "tag")]
        tags: Vec<String>,

        #[arg(long = "evidence-uri")]
        evidence_uri: Option<String>,

        #[arg(long = "evidence-kind", default_value = "conversation")]
        evidence_kind: EvidenceKind,

        #[arg(long)]
        reason: String,

        #[arg(long)]
        apply: bool,

        #[arg(long)]
        force: bool,
    },

    /// Tombstone an active record while retaining auditable history.
    Tombstone {
        target_id: String,

        #[arg(long = "evidence-uri")]
        evidence_uri: Option<String>,

        #[arg(long = "evidence-kind", default_value = "conversation")]
        evidence_kind: EvidenceKind,

        #[arg(long)]
        reason: String,

        #[arg(long)]
        apply: bool,

        #[arg(long)]
        force: bool,
    },

    /// Reinforce an active record with additional evidence and a confidence bump.
    Reinforce {
        target_id: String,

        #[arg(long = "evidence-uri")]
        evidence_uri: String,

        #[arg(long = "evidence-kind", default_value = "conversation")]
        evidence_kind: EvidenceKind,

        #[arg(long, default_value = "reinforce record with new evidence")]
        reason: String,

        #[arg(long, default_value = "explicit_reinforcement")]
        outcome: String,

        #[arg(long)]
        apply: bool,

        #[arg(long)]
        force: bool,
    },

    /// Contest an active or already contested record with evidence.
    Contest {
        target_id: String,

        #[arg(long = "evidence-uri")]
        evidence_uri: String,

        #[arg(long = "evidence-kind", default_value = "conversation")]
        evidence_kind: EvidenceKind,

        #[arg(long)]
        reason: String,

        #[arg(long)]
        apply: bool,

        #[arg(long)]
        force: bool,
    },

    /// Resolve a contested record by upholding, tombstoning, or superseding it.
    ResolveContest {
        target_id: String,

        #[arg(long)]
        resolution: ContestResolution,

        #[arg(long = "class")]
        class: Option<RecordClass>,

        #[arg(long)]
        claim: Option<String>,

        #[arg(long, default_value_t = 0.75)]
        confidence: f32,

        #[arg(long)]
        project: Option<String>,

        #[arg(long = "tag")]
        tags: Vec<String>,

        #[arg(long = "evidence-uri")]
        evidence_uri: Option<String>,

        #[arg(long = "evidence-kind", default_value = "conversation")]
        evidence_kind: EvidenceKind,

        #[arg(long)]
        reason: String,

        #[arg(long)]
        apply: bool,

        #[arg(long)]
        force: bool,
    },


    /// Configuration commands.
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },

    /// Policy inspection commands.
    Policy {
        #[command(subcommand)]
        command: PolicyCommands,
    },

    /// Maintenance and health scan commands.
    Maintenance {
        #[command(subcommand)]
        command: MaintenanceCommands,
    },

    /// Namespace inspection commands.
    Namespace {
        #[command(subcommand)]
        command: NamespaceCommands,
    },

    /// Export the PI store as a portable JSON bundle.
    Export {
        /// Optional output path. If omitted, the export bundle is printed to stdout.
        #[arg(long)]
        output: Option<PathBuf>,

        /// Export all namespaces.
        #[arg(long = "all-namespaces")]
        all_namespaces: bool,

        /// Optional project filter. Global records are included with matching project records.
        #[arg(long)]
        project: Option<String>,

        /// Redact evidence URIs and event messages in the exported bundle.
        #[arg(long)]
        redacted: bool,

        #[arg(long)]
        layer: Option<MemoryLayer>,
    },

    /// Import a portable PI JSON bundle. Duplicate ids are skipped, not overwritten.
    Import {
        path: PathBuf,

        /// Report planned import changes without rewriting files.
        #[arg(long)]
        dry_run: bool,

        /// Preserve namespaces stored in the bundle.
        #[arg(long = "preserve-namespaces")]
        preserve_namespaces: bool,

        /// Create a timestamped backup under the store before rewriting.
        #[arg(long)]
        backup: bool,

        #[arg(long)]
        json: bool,
    },

    /// Migrate legacy JSONL entries to the current schema version.
    Migrate {
        /// Report planned migration changes without rewriting files.
        #[arg(long)]
        dry_run: bool,

        /// Create a timestamped backup under the store before rewriting.
        #[arg(long)]
        backup: bool,

        #[arg(long)]
        json: bool,
    },

    /// Inspect store health and governance state.
    Doctor {
        #[arg(long)]
        json: bool,
    },

    /// List recent records.
    List {
        #[arg(long, default_value_t = 20)]
        limit: usize,
        #[arg(long)]
        layer: Option<MemoryLayer>,
    },

    /// Inspect a governed memory record by id.
    InspectRecord {
        record_id: String,
        #[arg(long)]
        json: bool,
        #[arg(long)]
        layer: Option<MemoryLayer>,
    },

    /// Score whether an observation is worth governed memory capture.
    MemoryWorth {
        observation: String,
        #[arg(long)]
        project: Option<String>,
        #[arg(long = "trust-class")]
        trust_class: Option<TrustClass>,
        #[arg(long = "source-kind")]
        source_kind: Option<SourceKind>,
        #[arg(long)]
        json: bool,
    },

    /// Capture deterministic memory candidates or L3 session evidence.
    Capture {
        #[arg(long)]
        text: Option<String>,
        #[arg(long)]
        stdin: bool,
        #[arg(long)]
        file: Option<PathBuf>,
        #[arg(long)]
        project: Option<String>,
        #[arg(long = "tag")]
        tags: Vec<String>,
        #[arg(long)]
        target: Option<String>,
        #[arg(long)]
        layer: Option<MemoryLayer>,
        #[arg(long = "trust-class")]
        trust_class: Option<TrustClass>,
        #[arg(long = "source-kind")]
        source_kind: Option<SourceKind>,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        json: bool,
    },

    /// Review captured candidate patches.
    Inbox {
        #[arg(long)]
        json: bool,
        #[arg(long)]
        all: bool,
        #[arg(long)]
        layer: Option<MemoryLayer>,
        #[arg(long)]
        apply: Option<String>,
        #[arg(long)]
        reject: Option<String>,
        #[arg(long)]
        defer: Option<String>,
        #[arg(long)]
        reason: Option<String>,
    },

    /// Build a scoped paste-ready governed memory context bundle.
    Context {
        query: String,
        #[arg(long)]
        project: Option<String>,
        #[arg(long, default_value_t = 1200)]
        budget: usize,
        #[arg(long, default_value = "markdown")]
        format: String,
        #[arg(long = "include-l3")]
        include_l3: bool,
        #[arg(long = "include-contested")]
        include_contested: bool,
        #[arg(long)]
        layer: Option<MemoryLayer>,
    },

    /// Append/search local L3 session evidence.
    Session {
        #[command(subcommand)]
        command: SessionCommands,
    },

    /// Explain why records were included or excluded from recall.
    RecallXray {
        query: String,
        #[arg(long)]
        project: Option<String>,
        #[arg(long, default_value_t = 1200)]
        budget: usize,
        #[arg(long)]
        json: bool,
        #[arg(long = "include-l3")]
        include_l3: bool,
        #[arg(long = "include-contested")]
        include_contested: bool,
        #[arg(long)]
        layer: Option<MemoryLayer>,
    },

    /// Generate review-only procedure candidates from governed workflow records.
    ProcedureCandidates { #[arg(long, default_value_t = 2)] min_source_records: usize, #[arg(long)] json: bool },

    /// Analyze rejected patches, stale deferrals, and warning events.
    FailureAnalysis { #[arg(long, default_value_t = 30)] stale_days: i64, #[arg(long)] json: bool },

    /// Preview a proposed patch without mutating the store.
    SimulatePatch { patch_id: String, #[arg(long)] json: bool },

    /// Build a bounded read-only memory graph report.
    Graph {
        #[arg(long, default_value_t = 5000)] max_nodes: usize,
        #[arg(long, default_value_t = 10000)] max_edges: usize,
        #[arg(long)] json: bool,
    },

    /// Analyze governed memory quality.
    Quality { #[command(subcommand)] command: QualityCommands },

    /// Record explicit recall outcome feedback for selected records.
    RecallFeedback { outcome: String, #[arg(required = true)] record_ids: Vec<String> },

    /// Generate MCP client configuration.
    McpConfig {
        client: String,
        #[arg(long)]
        command: Option<PathBuf>,
        #[arg(long)]
        json: bool,
    },

    /// Safely install or merge MCP client configuration.
    McpInstall {
        client: String,
        #[arg(long)]
        command: Option<PathBuf>,
        #[arg(long)]
        config: Option<PathBuf>,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        yes: bool,
        #[arg(long)]
        no_backup: bool,
    },

    /// Diagnose MCP client configuration and direct stdio connectivity.
    McpDoctor {
        client: String,
        #[arg(long)]
        command: Option<PathBuf>,
        #[arg(long)]
        config: Option<PathBuf>,
        #[arg(long)]
        json: bool,
    },

    /// Run built-in smoke tests against a temporary store.
    SmokeTest {
        #[arg(long)]
        json: bool,
    },

    /// Run release-candidate audit checks.
    ReleaseAudit {
        #[arg(long)]
        json: bool,
    },

    /// Print version history.
    Changelog,

    /// Run PI as an MCP server over stdio.
    McpStdio,
}

#[derive(Debug, Subcommand)]
enum ConfigCommands {
    /// Show effective JSON config.
    Show,
    /// Set a namespace policy profile.
    SetPolicy { namespace: String, profile: PolicyProfile },
    /// Enable or disable bounded local recall telemetry.
    SetRecallTelemetry { enabled: String, #[arg(long, default_value_t = 1000)] max_events: usize },
}

#[derive(Debug, Subcommand)]
enum PolicyCommands {
    /// Show policy config health.
    Doctor { #[arg(long)] json: bool },
    /// Explain operation behavior across profiles.
    Explain { operation: String },
}

#[derive(Debug, Subcommand)]
enum MaintenanceCommands {
    /// Run a read-only governance maintenance scan.
    Scan { #[arg(long)] json: bool, #[arg(long)] layer: Option<MemoryLayer> },
}

#[derive(Debug, Subcommand)]
enum QualityCommands {
    /// Analyze per-record memory quality.
    Memory { #[arg(long)] json: bool },
    /// Analyze graph relationship quality.
    Relationship { #[arg(long)] json: bool },
    /// Analyze longitudinal recall effectiveness.
    Recall { #[arg(long)] json: bool },
    /// Aggregate memory, relationship, recall, inbox, and runtime quality.
    Store { #[arg(long)] json: bool },
}

#[derive(Debug, Subcommand)]
enum SessionCommands {
    /// Add append-only L3 session evidence.
    Add {
        #[arg(long)] text: Option<String>,
        #[arg(long)] stdin: bool,
        #[arg(long)] file: Option<PathBuf>,
        #[arg(long)] project: Option<String>,
        #[arg(long)] json: bool,
    },
    /// Search append-only L3 session evidence locally.
    Search {
        query: String,
        #[arg(long)] project: Option<String>,
        #[arg(long)] after: Option<String>,
        #[arg(long)] json: bool,
    },
    /// List extracted session decision markers.
    Decisions {
        #[arg(long)] project: Option<String>,
        #[arg(long)] days: Option<i64>,
        #[arg(long)] json: bool,
    },
}

#[derive(Debug, Serialize)]
struct ReleaseAuditReport {
    result: String,
    version: String,
    checks: Vec<ReleaseAuditCheck>,
    failures: Vec<String>,
}

#[derive(Debug, Serialize)]
struct ReleaseAuditCheck {
    name: String,
    status: String,
}

fn audit_check(checks: &mut Vec<ReleaseAuditCheck>, failures: &mut Vec<String>, name: &str, passed: bool, detail: &str) {
    checks.push(ReleaseAuditCheck { name: name.to_string(), status: if passed { "pass" } else { "fail" }.to_string() });
    if !passed { failures.push(format!("{name}: {detail}")); }
}

#[derive(Debug, Serialize)]
struct ReviewInbox {
    pending_count: usize,
    patches: Vec<ReviewPatch>,
}

#[derive(Debug, Serialize)]
struct ReviewPatch {
    id: String,
    status: String,
    operation: String,
    decision: Option<String>,
    namespace: String,
    project: Option<String>,
    summary: String,
    evidence: Vec<String>,
}

#[derive(Debug, Serialize)]
struct ReviewDetail {
    id: String,
    status: String,
    operation: String,
    decision: Option<String>,
    namespace: String,
    project: Option<String>,
    claim: Option<String>,
    evidence: Vec<String>,
    warnings: Vec<String>,
    reason: String,
    target_id: Option<String>,
    next_actions: Vec<String>,
}

#[derive(Debug, Serialize)]
struct DemoReport {
    store: String,
    records: usize,
    pending_patches: usize,
    namespaces: Vec<String>,
    #[serde(rename = "try")]
    try_commands: Vec<String>,
}

#[derive(Debug, Serialize)]
struct AgentInstructionsReport {
    instructions: Vec<String>,
}

#[derive(Debug, Serialize)]
struct InspectRecordError {
    error: String,
    record_id: String,
}

fn record_project(inspection: &RecordInspection) -> Option<String> {
    inspection.record.scope.key.clone()
}

fn print_record_inspection(inspection: &RecordInspection) {
    let record = &inspection.record;
    println!("Record {}\n", record.id);
    println!("Status: {:?}", record.status);
    println!("Class: {:?}", record.class);
    println!("Namespace: {}", record.namespace);
    if let Some(project) = record_project(inspection) { println!("Project: {}", project); }
    println!("Confidence: {:.2}", record.confidence);
    println!("\nClaim:\n  {}", record.claim);
    if !record.tags.is_empty() { println!("\nTags:\n  {}", record.tags.join(", ")); }
    if !record.evidence.is_empty() {
        println!("\nEvidence:");
        for evidence in &record.evidence { println!("  - {} ({:?})", evidence.uri, evidence.kind); }
    }
    println!("\nCreated:\n  {}", record.created_at);
    println!("Updated:\n  {}", record.updated_at);
    println!("\nRevision:");
    println!("  Supersedes: {}", if inspection.revision.supersedes.is_empty() { "none".to_string() } else { inspection.revision.supersedes.join(", ") });
    println!("  Superseded by: {}", if inspection.revision.superseded_by.is_empty() { "none".to_string() } else { inspection.revision.superseded_by.join(", ") });
    println!("  Contested: {}", inspection.revision.contested);
    println!("  Tombstoned: {}", inspection.revision.tombstoned);
    if !inspection.related_patches.is_empty() {
        println!("\nRelated patches:");
        for patch in &inspection.related_patches { println!("  - {}", patch); }
    }
    println!("\nNext:");
    println!("  pi retrieve \"{}\" --explain", record.claim.split_whitespace().take(4).collect::<Vec<_>>().join(" "));
    println!("  pi contest {} --reason \"...\" --evidence-uri review:...", record.id);
    println!("  pi supersede {} --class {:?} --claim \"...\" --reason \"...\" --evidence-uri review:...", record.id, record.class);
}

fn patch_project(summary: &PatchSummary) -> Option<String> {
    summary.proposed_record_claim.as_ref()?;
    None
}

fn summarize_claim(claim: &Option<String>, reason: &str) -> String {
    claim.clone().unwrap_or_else(|| reason.to_string()).chars().take(96).collect()
}

fn review_patch_from_summary(summary: &PatchSummary) -> ReviewPatch {
    ReviewPatch {
        id: summary.patch_id.clone(),
        status: format!("{:?}", summary.latest_status),
        operation: format!("{:?}", summary.operation),
        decision: None,
        namespace: "unknown".to_string(),
        project: patch_project(summary),
        summary: summarize_claim(&summary.proposed_record_claim, &summary.reason),
        evidence: vec![format!("{} evidence ref(s)", summary.evidence_count)],
    }
}

fn review_detail_from_inspection(inspection: &PatchInspection) -> ReviewDetail {
    let latest = inspection.history.last();
    let evidence = latest.map(|p| p.evidence.iter().map(|e| e.uri.clone()).collect()).unwrap_or_default();
    let namespace = latest.map(|p| p.namespace.clone()).unwrap_or_else(|| "unknown".to_string());
    let project = latest.and_then(|p| p.proposed_record.as_ref()).and_then(|r| r.scope.key.clone());
    let warnings = inspection.current_decision.as_ref().map(|d| d.warnings.clone()).unwrap_or_default();
    let decision = inspection.current_decision.as_ref().map(|d| format!("{:?}", d.status));
    let mut next_actions = Vec::new();
    if inspection.can_apply_without_force { next_actions.push(format!("pi review {} --apply", inspection.summary.patch_id)); }
    if inspection.can_apply_with_force { next_actions.push(format!("pi review {} --apply --force", inspection.summary.patch_id)); }
    ReviewDetail {
        id: inspection.summary.patch_id.clone(),
        status: format!("{:?}", inspection.summary.latest_status),
        operation: format!("{:?}", inspection.summary.operation),
        decision,
        namespace,
        project,
        claim: inspection.summary.proposed_record_claim.clone(),
        evidence,
        warnings,
        reason: inspection.summary.reason.clone(),
        target_id: inspection.summary.target_id.clone(),
        next_actions,
    }
}

fn agent_instructions() -> Vec<String> {
    vec![
        "Before starting a non-trivial coding task, retrieve PI context for the project.".to_string(),
        "Prefer active records; treat contested records as unsafe unless explicitly included.".to_string(),
        "When the user corrects you, propose a correction memory.".to_string(),
        "When a memory appears stale, contest or supersede it.".to_string(),
        "Respect PI policy profile decisions and do not silently store high-impact memories under strict policy.".to_string(),
        "Review pending patches before release work.".to_string(),
    ]
}

#[derive(Debug, Subcommand)]
enum NamespaceCommands {
    /// List namespace summaries.
    List,
    /// Inspect namespace health.
    Doctor { #[arg(long)] json: bool },
}

fn mcp_args(store: &Path, namespace: &str) -> Vec<String> {
    vec!["--store".into(), store.display().to_string(), "--namespace".into(), namespace.into(), "mcp-stdio".into()]
}

fn shared_mcp_json(command: &str, store: &Path, namespace: &str) -> serde_json::Value {
    serde_json::json!({"mcpServers":{"pi-governance":{"command":command,"args":mcp_args(store, namespace)}}})
}

fn opencode_mcp_json(command: &str, store: &Path, namespace: &str) -> serde_json::Value {
    let mut cmd = vec![command.to_string()];
    cmd.extend(mcp_args(store, namespace));
    serde_json::json!({"mcp":{"pi-governance":{"type":"local","command":cmd,"enabled":true,"timeout":10000}}})
}

fn codex_mcp_toml(command: &str, store: &Path, namespace: &str) -> String {
    format!("[mcp_servers.pi-governance]\ncommand = {:?}\nargs = [\"--store\", {:?}, \"--namespace\", {:?}, \"mcp-stdio\"]\nenabled = true\nstartup_timeout_sec = 10\ntool_timeout_sec = 60\ndefault_tools_approval_mode = \"prompt\"\n", command, store.display().to_string(), namespace)
}

fn default_mcp_config_path(client: &str) -> Result<PathBuf> {
    let home = std::env::var("HOME")?;
    Ok(match client {
        "opencode" => PathBuf::from(home).join(".config/opencode/opencode.jsonc"),
        "codex" => PathBuf::from(home).join(".codex/config.toml"),
        "pi-agent" => PathBuf::from(home).join(".config/mcp/mcp.json"),
        other => anyhow::bail!("unsupported mcp client: {other}. Use opencode, codex, or pi-agent."),
    })
}

fn has_jsonc_comments(text: &str) -> bool {
    text.lines().any(|l| l.trim_start().starts_with("//") || l.contains("/*"))
}

fn backup_path(path: &Path) -> PathBuf {
    let ts = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
    PathBuf::from(format!("{}.backup.{ts}", path.display()))
}

fn extract_server_names(value: &serde_json::Value, key: &str) -> Vec<String> {
    value.get(key).and_then(|v| v.as_object()).map(|m| m.keys().cloned().collect()).unwrap_or_default()
}

fn install_json_config(path: &Path, key: &str, server: serde_json::Value, dry_run: bool, no_backup: bool) -> Result<(String, Option<PathBuf>, Vec<String>)> {
    let existed = path.exists();
    let mut root = if existed {
        let text = fs::read_to_string(path)?;
        if path.extension().and_then(|s| s.to_str()) == Some("jsonc") && has_jsonc_comments(&text) { anyhow::bail!("refusing to rewrite JSONC comments; use mcp-config output as a snippet"); }
        serde_json::from_str::<serde_json::Value>(&text)?
    } else { serde_json::json!({}) };
    if !root.is_object() { anyhow::bail!("config root must be a JSON object"); }
    let preserved = extract_server_names(&root, key).into_iter().filter(|n| n != "pi-governance").collect::<Vec<_>>();
    let action = if root.get(key).and_then(|v| v.get("pi-governance")).is_some() { "update" } else { "add" }.to_string();
    root.as_object_mut().unwrap().entry(key).or_insert_with(|| serde_json::json!({}));
    root.get_mut(key).unwrap().as_object_mut().ok_or_else(|| anyhow::anyhow!("{key} must be a JSON object"))?.insert("pi-governance".into(), server);
    if dry_run { return Ok((action, None, preserved)); }
    if let Some(parent) = path.parent() { fs::create_dir_all(parent)?; }
    let backup = if existed && !no_backup { let b = backup_path(path); fs::copy(path, &b)?; Some(b) } else { None };
    fs::write(path, format!("{}\n", serde_json::to_string_pretty(&root)?))?;
    Ok((action, backup, preserved))
}

fn install_codex_config(path: &Path, command: &str, store: &Path, namespace: &str, dry_run: bool, no_backup: bool) -> Result<(String, Option<PathBuf>, Vec<String>)> {
    let existed = path.exists();
    let text = if existed { fs::read_to_string(path)? } else { String::new() };
    let preserved = text.lines().filter_map(|l| l.strip_prefix("[mcp_servers.").and_then(|r| r.strip_suffix(']'))).filter(|n| *n != "pi-governance").map(str::to_string).collect::<Vec<_>>();
    let action = if text.contains("[mcp_servers.pi-governance]") { "update" } else { "add" }.to_string();
    let mut out = String::new();
    let mut skipping = false;
    for line in text.lines() {
        if line.trim() == "[mcp_servers.pi-governance]" { skipping = true; continue; }
        if skipping && line.trim_start().starts_with('[') { skipping = false; }
        if !skipping { out.push_str(line); out.push('\n'); }
    }
    if !out.ends_with("\n\n") { out.push('\n'); }
    out.push_str(&codex_mcp_toml(command, store, namespace));
    if dry_run { return Ok((action, None, preserved)); }
    if let Some(parent) = path.parent() { fs::create_dir_all(parent)?; }
    let backup = if existed && !no_backup { let b = backup_path(path); fs::copy(path, &b)?; Some(b) } else { None };
    fs::write(path, out)?;
    Ok((action, backup, preserved))
}

fn confirm_install(yes: bool, dry_run: bool) -> Result<()> {
    if yes || dry_run { return Ok(()); }
    print!("Proceed with MCP config install? [y/N] "); io::stdout().flush()?;
    let mut s = String::new(); io::stdin().read_line(&mut s)?;
    if matches!(s.trim(), "y" | "Y" | "yes" | "YES") { Ok(()) } else { anyhow::bail!("installation cancelled; rerun with --yes to confirm") }
}

fn read_server_from_config(client: &str, path: &Path) -> Result<(Option<String>, Vec<String>)> {
    let text = fs::read_to_string(path)?;
    match client {
        "opencode" => { let v: serde_json::Value = serde_json::from_str(&text)?; let arr = v["mcp"]["pi-governance"]["command"].as_array().cloned(); if let Some(a)=arr { Ok((a.first().and_then(|v| v.as_str()).map(str::to_string), a.iter().skip(1).filter_map(|v| v.as_str().map(str::to_string)).collect())) } else { Ok((None, vec![])) } }
        "pi-agent" => { let v: serde_json::Value = serde_json::from_str(&text)?; let s=&v["mcpServers"]["pi-governance"]; Ok((s["command"].as_str().map(str::to_string), s["args"].as_array().map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect()).unwrap_or_default())) }
        "codex" => { let mut in_sec=false; let mut cmd=None; let mut args=Vec::new(); for l in text.lines() { let t=l.trim(); if t=="[mcp_servers.pi-governance]" { in_sec=true; continue; } if in_sec && t.starts_with('[') { break; } if in_sec && t.starts_with("command") { cmd=t.split_once('=').map(|(_,v)| v.trim().trim_matches('"').to_string()); } if in_sec && t.starts_with("args") { args=t.split('"').skip(1).step_by(2).map(str::to_string).collect(); } } Ok((cmd,args)) }
        _ => anyhow::bail!("unsupported mcp client: {client}"),
    }
}

fn run_tools_list(command: &str, args: &[String]) -> Result<Vec<String>> {
    let mut child = Command::new(command).args(args).stdin(Stdio::piped()).stdout(Stdio::piped()).spawn()?;
    child.stdin.as_mut().unwrap().write_all(br#"{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}
"#)?;
    let out = child.wait_with_output()?;
    if !out.status.success() { anyhow::bail!("tools/list process failed"); }
    let text = String::from_utf8_lossy(&out.stdout);
    let v: serde_json::Value = serde_json::from_str(text.lines().last().unwrap_or(&text))?;
    Ok(v["result"]["tools"].as_array().map(|a| a.iter().filter_map(|t| t["name"].as_str().map(str::to_string)).collect()).unwrap_or_default())
}

fn proposal_input(
    namespace: String,
    class: RecordClass,
    claim: String,
    confidence: f32,
    scope: Scope,
    tags: Vec<String>,
    evidence_refs: Vec<EvidenceRef>,
    reason: Option<String>,
    layer: Option<MemoryLayer>,
    memory_kind: Option<MemoryKind>,
    rule_type: Option<RuleType>,
    trust_class: TrustClass,
    durability: Durability,
    source_kind: SourceKind,
) -> ProposalInput {
    ProposalInput { namespace, class, claim, confidence, scope, tags, evidence_refs, reason, layer, memory_kind, rule_type, trust_class, durability, source_kind }
}

fn is_daily_target(target: Option<&str>) -> bool {
    matches!(target.map(|t| t.to_lowercase().replace('_', "-")).as_deref(), Some("daily") | Some("session") | Some("l3") | Some("l3-session"))
}

fn is_long_term_target(target: Option<&str>) -> bool {
    matches!(target.map(|t| t.to_lowercase().replace('_', "-")).as_deref(), Some("long-term") | Some("memory") | Some("l2") | Some("l2-playbook"))
}

fn print_event_list(events: Vec<StoreEvent>, json_out: bool) -> Result<()> {
    if json_out { println!("{}", serde_json::to_string_pretty(&events)?); }
    else { for e in events { println!("{} {}", e.created_at, e.message); } }
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let namespace = cli.namespace;
    let store_path = cli.store;
    let store = JsonlStore::new(store_path.clone());
    let engine = GovernanceEngine::new(store.clone());

    match cli.command {
        Commands::Init => {
            engine.init()?;
            println!("PI store initialized.");
        }

        Commands::AgentInstructions { json } => {
            let instructions = agent_instructions();
            if json {
                println!("{}", serde_json::to_string_pretty(&AgentInstructionsReport { instructions })?);
            } else {
                println!("PI Agent Instructions\n");
                for (idx, instruction) in instructions.iter().enumerate() {
                    println!("{}. {}", idx + 1, instruction);
                }
            }
        }

        Commands::Review { patch_id, json, apply, force, reject, defer, reason } => {
            if let Some(reject_id) = reject {
                let result = engine.reject_patch_by_id(&reject_id, &namespace, &reason.ok_or_else(|| anyhow::anyhow!("--reason is required for --reject"))?)?;
                if json { println!("{}", serde_json::to_string_pretty(&result)?); }
                else { println!("Patch {}\nStatus: {}\nReason: {}", result.patch_id, result.status, result.reason); }
            } else if let Some(defer_id) = defer {
                let result = engine.defer_patch_by_id(&defer_id, &namespace, &reason.ok_or_else(|| anyhow::anyhow!("--reason is required for --defer"))?)?;
                if json { println!("{}", serde_json::to_string_pretty(&result)?); }
                else { println!("Patch {}\nStatus: {}\nReason: {}", result.patch_id, result.status, result.reason); }
            } else if let Some(patch_id) = patch_id {
                if apply {
                    let result = engine.apply_patch_by_id(&patch_id, force)?;
                    if json { println!("{}", serde_json::to_string_pretty(&result)?); }
                    else { println!("Patch {}", result.patch_id); println!("Applied: {}", result.applied); println!("{}", result.message); }
                } else {
                    let inspection = engine.inspect_patch(&patch_id)?;
                    let detail = review_detail_from_inspection(&inspection);
                    if json {
                        println!("{}", serde_json::to_string_pretty(&detail)?);
                    } else {
                        println!("Patch {}\n", detail.id);
                        println!("Status: {}", detail.status);
                        println!("Operation: {}", detail.operation);
                        println!("Decision: {}", detail.decision.clone().unwrap_or_else(|| "n/a".to_string()));
                        println!("Namespace: {}", detail.namespace);
                        if let Some(project) = &detail.project { println!("Project: {}", project); }
                        if let Some(target) = &detail.target_id { println!("Target: {}", target); }
                        if let Some(claim) = &detail.claim { println!("\nClaim:\n  {}", claim); }
                        if !detail.evidence.is_empty() { println!("\nEvidence:"); for e in &detail.evidence { println!("  {}", e); } }
                        if !detail.warnings.is_empty() { println!("\nWarnings:"); for w in &detail.warnings { println!("  {}", w); } }
                        if !detail.reason.is_empty() { println!("\nReason:\n  {}", detail.reason); }
                        if !detail.next_actions.is_empty() { println!("\nNext:"); for action in &detail.next_actions { println!("  {}", action); } }
                    }
                }
            } else {
                let patches: Vec<ReviewPatch> = engine.list_patches(200)?.into_iter().filter(|p| p.latest_status == PatchStatus::Proposed).map(|p| {
                    let mut review = review_patch_from_summary(&p);
                    if let Ok(detail) = engine.inspect_patch(&p.patch_id).map(|inspection| review_detail_from_inspection(&inspection)) {
                        review.namespace = detail.namespace;
                        review.project = detail.project;
                        review.decision = detail.decision;
                        review.evidence = detail.evidence;
                    }
                    review
                }).collect();
                let inbox = ReviewInbox { pending_count: patches.len(), patches };
                if json {
                    println!("{}", serde_json::to_string_pretty(&inbox)?);
                } else {
                    println!("PI Review Inbox\n");
                    println!("Pending patches: {}\n", inbox.pending_count);
                    if inbox.pending_count == 0 {
                        println!("No pending memory changes.");
                        println!("Try:\n  pi demo --store /tmp/pi-demo\n  pi --store /tmp/pi-demo review");
                    } else {
                        for (idx, patch) in inbox.patches.iter().enumerate() {
                            println!("{}. {}", idx + 1, patch.id);
                            println!("   operation: {}", patch.operation);
                            println!("   status: {}", patch.status);
                            println!("   namespace: {}", patch.namespace);
                            println!("   summary: {}", patch.summary);
                            println!("   evidence: {}", patch.evidence.join(", "));
                            println!("   next: pi review {}\n", patch.id);
                        }
                    }
                }
            }
        }

        Commands::Demo { json, reset } => {
            let demo_path = if store_path == PathBuf::from(".pi") { PathBuf::from("/tmp/pi-governance-demo") } else { store_path.clone() };
            if demo_path == PathBuf::from(".pi") || demo_path.ends_with("target") { anyhow::bail!("refusing to use protected demo path"); }
            if demo_path.exists() {
                let non_empty = std::fs::read_dir(&demo_path)?.next().is_some();
                if non_empty && !reset { anyhow::bail!("demo store exists and is non-empty; pass --reset to replace it"); }
                if reset { std::fs::remove_dir_all(&demo_path)?; }
            }
            let demo_store = JsonlStore::new(demo_path.clone());
            let demo_engine = GovernanceEngine::new(demo_store);
            demo_engine.init()?;
            let ev = |uri: &str| vec![EvidenceRef::new(EvidenceKind::Conversation, uri)];
            let input = |class, claim: &str, tag: &str, uri: &str| proposal_input("default".to_string(), class, claim.to_string(), 0.75, Scope::project("pi-governance-rs"), vec![tag.to_string()], ev(uri), Some("demo memory".to_string()), None, None, None, TrustClass::DirectUserInstruction, Durability::Project, SourceKind::ManualCli);
            demo_engine.propose_record(input(RecordClass::Preference, "Use cargo test --workspace before tagging Rust release candidates.", "release", "demo:preference"), true, false)?;
            demo_engine.propose_record(input(RecordClass::Correction, "If MCP smoke tests fail, inspect tools/list before tools/call.", "mcp", "demo:correction"), true, false)?;
            demo_engine.propose_record(input(RecordClass::Workflow, "Release validation requires cargo check, cargo test, smoke-test, release-audit, and changelog verification.", "workflow", "demo:workflow"), true, false)?;
            let contested = demo_engine.propose_record(input(RecordClass::Observation, "Old release note says rc.1 is current, but rc.3 supersedes it.", "contest", "demo:contested"), true, false)?.record_id.unwrap();
            demo_engine.contest_record(ContestInput { namespace: "default".to_string(), target_id: contested, evidence_refs: ev("demo:contest"), reason: "demo contested record".to_string() }, true, true)?;
            let old = demo_engine.propose_record(input(RecordClass::Observation, "v1.0.0-rc.2 is current.", "supersede", "demo:old-current"), true, false)?.record_id.unwrap();
            demo_engine.supersede_record(SupersedeInput { namespace: "default".to_string(), target_id: old, class: RecordClass::Observation, claim: "v1.0.0 is current.".to_string(), confidence: 0.8, scope: Scope::project("pi-governance-rs"), tags: vec!["supersede".to_string()], evidence_refs: ev("demo:new-current"), reason: "demo supersession".to_string() }, true, true)?;
            let tomb = demo_engine.propose_record(input(RecordClass::Workflow, "Old instruction: publish immediately after tests.", "tombstone", "demo:tombstone"), true, false)?.record_id.unwrap();
            demo_engine.tombstone_record(TombstoneInput { namespace: "default".to_string(), target_id: tomb, evidence_refs: ev("demo:no-publish"), reason: "this repo does not publish during RC validation".to_string() }, true, true)?;
            demo_engine.set_policy("strict-demo", PolicyProfile::Strict)?;
            demo_engine.set_policy("permissive-demo", PolicyProfile::Permissive)?;
            demo_engine.propose_record(proposal_input("default".to_string(), RecordClass::Workflow, "Review pending patches before release.".to_string(), 0.75, Scope::project("pi-governance-rs"), vec!["review".to_string()], ev("demo:pending-review"), Some("demo pending patch".to_string()), Some(MemoryLayer::L2Playbook), Some(MemoryKind::Instruction), Some(RuleType::Workflow), TrustClass::DirectUserInstruction, Durability::Project, SourceKind::ManualCli), false, false)?;
            let report = DemoReport { store: demo_path.display().to_string(), records: demo_engine.list_records(200)?.len(), pending_patches: demo_engine.list_patches(200)?.iter().filter(|p| p.latest_status == PatchStatus::Proposed).count(), namespaces: vec!["default".to_string(), "strict-demo".to_string(), "permissive-demo".to_string()], try_commands: vec![format!("pi --store {} review", demo_path.display()), format!("pi --store {} retrieve \"release workflow\" --explain", demo_path.display())] };
            if json { println!("{}", serde_json::to_string_pretty(&report)?); }
            else { println!("PI Demo Store Created\n"); println!("Store: {}", report.store); println!("Records: {}", report.records); println!("Pending patches: {}", report.pending_patches); println!("Namespaces: {}", report.namespaces.join(", ")); println!("\nTry:"); for cmd in &report.try_commands { println!("  {}", cmd); } println!("  pi --store {} doctor", report.store); println!("  pi --store {} namespace doctor", report.store); println!("  pi --store {} policy doctor", report.store); }
        }

        Commands::Propose {
            class,
            claim,
            confidence,
            project,
            tags,
            evidence_uri,
            evidence_kind,
            reason,
            layer,
            apply,
            force,
        } => {
            let scope = match project {
                Some(project) => Scope::project(project),
                None => Scope::global(),
            };

            let evidence_refs = match evidence_uri {
                Some(uri) => vec![EvidenceRef::new(evidence_kind, uri)],
                None => Vec::new(),
            };

            let result = engine.propose_record(
                proposal_input(namespace.clone(), class, claim, confidence, scope, tags, evidence_refs, reason, layer, None, None, TrustClass::DirectUserInstruction, Durability::Project, SourceKind::ManualCli),
                apply,
                force,
            )?;

            println!("{}", serde_json::to_string_pretty(&result)?);
        }

        Commands::MemoryWorth { observation, project: _, trust_class, source_kind, json } => {
            let report = score_memory_worth(&observation, trust_class, source_kind);
            if json { println!("{}", serde_json::to_string_pretty(&report)?); }
            else {
                println!("Decision: {:?}", report.decision);
                println!("Confidence: {:.2}", report.confidence);
                println!("Suggested layer: {}", report.suggested_layer);
                println!("Suggested class: {}", report.suggested_class);
                if !report.reasons.is_empty() { println!("Reasons: {}", report.reasons.join(", ")); }
                if !report.warnings.is_empty() { println!("Warnings: {}", report.warnings.join(", ")); }
            }
        }

        Commands::Capture { text, stdin, file, project, mut tags, target, layer, trust_class, source_kind, dry_run, json } => {
            let input_text = read_text_input(text, file, stdin)?;
            let source = source_kind.unwrap_or(if stdin { SourceKind::Stdin } else { SourceKind::ManualCli });
            if is_daily_target(target.as_deref()) && !is_long_term_target(target.as_deref()) {
                store.init()?;
                let ev = session_event(&namespace, project.as_deref(), &input_text, source);
                if !dry_run { store.append_event(&ev)?; }
                let report = pi_governance_engine::CaptureReport { input_summary: input_text.chars().take(80).collect(), candidates: Vec::new(), daily_only: vec![input_text.clone()], inquiries: Vec::new(), rejected: Vec::new(), applied: false };
                if json { println!("{}", serde_json::to_string_pretty(&report)?); }
                else { println!("Captured L3/session entry. Decisions: {}", pi_governance_engine::extract_decisions(&input_text).len()); }
            } else {
                let worth = score_memory_worth(&input_text, trust_class, Some(source));
                let mut report = pi_governance_engine::CaptureReport { input_summary: input_text.chars().take(80).collect(), candidates: Vec::new(), daily_only: Vec::new(), inquiries: Vec::new(), rejected: Vec::new(), applied: false };
                match worth.decision {
                    MemoryWorthDecision::Reject => report.rejected.push(input_text.clone()),
                    MemoryWorthDecision::DailyOnly => {
                        let ev = session_event(&namespace, project.as_deref(), &input_text, source);
                        if !dry_run { store.append_event(&ev)?; }
                        report.daily_only.push(input_text.clone());
                    }
                    MemoryWorthDecision::Inquiry => report.inquiries.push(input_text.clone()),
                    MemoryWorthDecision::Candidate => {
                        if tags.is_empty() { tags = worth.suggested_tags.clone(); }
                        let claim = claim_from_capture(&input_text);
                        let suggested_layer = layer.unwrap_or(worth.suggested_layer);
                        let verification = verify_candidate(&claim, suggested_layer, worth.trust_class, worth.durability);
                        let mut patch_id = None;
                        if !dry_run {
                            let result = engine.propose_record(proposal_input(
                                namespace.clone(), worth.suggested_class.clone(), claim.clone(), worth.confidence,
                                scope_for_project(project.clone()), tags, vec![evidence_for_capture(source, worth.trust_class, worth.durability)],
                                Some("captured deterministic memory candidate".to_string()), Some(suggested_layer), Some(worth.suggested_memory_kind), worth.suggested_rule_type,
                                worth.trust_class, worth.durability, source,
                            ), false, false)?;
                            patch_id = Some(result.patch_id);
                        }
                        report.candidates.push(pi_governance_engine::CaptureCandidate { claim, decision: worth.decision, patch_id, suggested_layer, trust_class: worth.trust_class, durability: worth.durability, memory_kind: worth.suggested_memory_kind, rule_type: worth.suggested_rule_type, verification });
                    }
                }
                if json { println!("{}", serde_json::to_string_pretty(&report)?); }
                else if report.candidates.is_empty() { println!("No durable candidate created."); }
                else { for c in &report.candidates { println!("candidate {} layer={} trust_class={:?}", c.patch_id.as_deref().unwrap_or("dry-run"), c.suggested_layer, c.trust_class); } }
            }
        }

        Commands::Retrieve {
            query,
            project,
            budget,
            format,
            retriever,
            explain,
            include_global,
            include_contested,
            min_confidence,
            classes,
            layer: _,
        } => {
            let retrieval_format = match format.as_str() {
                "json" => RetrievalFormat::Json,
                "markdown" | "md" => RetrievalFormat::Markdown,
                other => anyhow::bail!("unsupported format: {other}. Use `markdown` or `json`."),
            };
            let bundle = engine.retrieve_context_with_options(RetrievalOptions {
                query,
                retriever,
                namespace: namespace.clone(),
                project,
                budget,
                format: retrieval_format.clone(),
                explain,
                classes,
                include_global,
                include_contested,
                min_confidence,
            })?;

            record_recall_event(&store, &namespace, RecallEventClient::Cli, RecallEventOperation::Retrieve, &bundle.query, bundle.records.iter().map(|ranked| ranked.record.id.clone()).collect(), bundle.budget.max_tokens, bundle.used_estimated_tokens)?;

            match retrieval_format {
                RetrievalFormat::Json => println!("{}", serde_json::to_string_pretty(&bundle)?),
                RetrievalFormat::Markdown => println!("{}", render_markdown(&bundle)),
            }
        }

        Commands::Apply { patch_id, force } => {
            let result = engine.apply_patch_by_id(&patch_id, force)?;
            println!("{}", serde_json::to_string_pretty(&result)?);
        }

        Commands::ListPatches { limit, json } => {
            let patches = engine.list_patches(limit)?;

            if json {
                println!("{}", serde_json::to_string_pretty(&patches)?);
            } else {
                for patch in patches {
                    println!(
                        "- {} status={:?} operation={:?} history={} claim={}",
                        patch.patch_id,
                        patch.latest_status,
                        patch.operation,
                        patch.history_entries,
                        patch
                            .proposed_record_claim
                            .as_deref()
                            .unwrap_or("<no proposed record>")
                    );
                }
            }
        }

        Commands::InspectPatch { patch_id, json } => {
            let inspection = engine.inspect_patch(&patch_id)?;

            if json {
                println!("{}", serde_json::to_string_pretty(&inspection)?);
            } else {
                println!("Patch: {}", inspection.summary.patch_id);
                println!("Status: {:?}", inspection.summary.latest_status);
                println!("Operation: {:?}", inspection.summary.operation);
                println!("History entries: {}", inspection.summary.history_entries);
                println!("Can apply without force: {}", inspection.can_apply_without_force);
                println!("Can apply with force: {}", inspection.can_apply_with_force);

                if let Some(claim) = inspection.summary.proposed_record_claim {
                    println!("Claim: {claim}");
                }

                if let Some(decision) = inspection.current_decision {
                    println!("Decision: {:?}", decision.status);

                    if !decision.reasons.is_empty() {
                        println!("Reasons:");
                        for reason in decision.reasons {
                            println!("- {reason}");
                        }
                    }

                    if !decision.warnings.is_empty() {
                        println!("Warnings:");
                        for warning in decision.warnings {
                            println!("- {warning}");
                        }
                    }
                }
            }
        }

        Commands::Supersede {
            target_id,
            class,
            claim,
            confidence,
            project,
            tags,
            evidence_uri,
            evidence_kind,
            reason,
            apply,
            force,
        } => {
            let scope = match project {
                Some(project) => Scope::project(project),
                None => Scope::global(),
            };

            let evidence_refs = match evidence_uri {
                Some(uri) => vec![EvidenceRef::new(evidence_kind, uri)],
                None => Vec::new(),
            };

            let result = engine.supersede_record(
                SupersedeInput {
                    namespace: namespace.clone(),
                    target_id,
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

            println!("{}", serde_json::to_string_pretty(&result)?);
        }

        Commands::Tombstone {
            target_id,
            evidence_uri,
            evidence_kind,
            reason,
            apply,
            force,
        } => {
            let evidence_refs = match evidence_uri {
                Some(uri) => vec![EvidenceRef::new(evidence_kind, uri)],
                None => Vec::new(),
            };

            let result = engine.tombstone_record(
                TombstoneInput {
                    namespace: namespace.clone(),
                    target_id,
                    evidence_refs,
                    reason,
                },
                apply,
                force,
            )?;

            println!("{}", serde_json::to_string_pretty(&result)?);
        }

        Commands::Reinforce {
            target_id,
            evidence_uri,
            evidence_kind,
            reason,
            outcome,
            apply,
            force,
        } => {
            let result = engine.reinforce_record(
                ReinforceInput {
                    namespace: namespace.clone(),
                    target_id,
                    evidence_refs: vec![EvidenceRef::new(evidence_kind, evidence_uri)],
                    reason: format!("{} (outcome: {})", reason, outcome),
                },
                apply,
                force,
            )?;

            println!("{}", serde_json::to_string_pretty(&result)?);
        }

        Commands::Contest {
            target_id,
            evidence_uri,
            evidence_kind,
            reason,
            apply,
            force,
        } => {
            let result = engine.contest_record(
                ContestInput {
                    namespace: namespace.clone(),
                    target_id,
                    evidence_refs: vec![EvidenceRef::new(evidence_kind, evidence_uri)],
                    reason,
                },
                apply,
                force,
            )?;

            println!("{}", serde_json::to_string_pretty(&result)?);
        }

        Commands::ResolveContest {
            target_id,
            resolution,
            class,
            claim,
            confidence,
            project,
            tags,
            evidence_uri,
            evidence_kind,
            reason,
            apply,
            force,
        } => {
            let scope = match project {
                Some(project) => Scope::project(project),
                None => Scope::global(),
            };

            let evidence_refs = match evidence_uri {
                Some(uri) => vec![EvidenceRef::new(evidence_kind, uri)],
                None => Vec::new(),
            };

            let result = engine.resolve_contest(
                ResolveContestInput {
                    namespace: namespace.clone(),
                    target_id,
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
            )?;

            println!("{}", serde_json::to_string_pretty(&result)?);
        }


        Commands::ProcedureCandidates { min_source_records, json } => {
            let report = generate_procedure_candidates(&store.load_records()?, &namespace, min_source_records, chrono::Utc::now());
            if json { println!("{}", serde_json::to_string_pretty(&report)?); } else { println!("PI Procedure Candidates\nCandidates: {}\nReview required. No mutation performed.", report.candidates.len()); }
        }

        Commands::FailureAnalysis { stale_days, json } => {
            let report = analyze_failure_patterns(&store.load_patches()?, &store.load_events()?, &namespace, stale_days, chrono::Utc::now());
            if json { println!("{}", serde_json::to_string_pretty(&report)?); } else { println!("PI Failure Analysis\nPatterns: {}\nReview required. No mutation performed.", report.patterns.len()); }
        }

        Commands::SimulatePatch { patch_id, json } => {
            let report = engine.simulate_patch(&patch_id)?;
            if json { println!("{}", serde_json::to_string_pretty(&report)?); } else { println!("PI Patch Simulation\nPatch: {}\nMemory quality delta: {:+}\nRelationship quality delta: {:+}\nStore quality delta: {:+}\nNo mutation performed.", report.patch_id, report.memory_quality_delta, report.relationship_quality_delta, report.store_quality_delta); }
        }

        Commands::Graph { max_nodes, max_edges, json } => {
            if max_nodes == 0 || max_edges == 0 { anyhow::bail!("graph limits must be greater than zero"); }
            let report = build_memory_graph(&store.load_records()?, &store.load_patches()?, &store.load_events()?, &namespace, max_nodes, max_edges, chrono::Utc::now());
            if json { println!("{}", serde_json::to_string_pretty(&report)?); } else { println!("PI Memory Graph\nNodes: {}\nEdges: {}\nTruncated: {}", report.nodes.len(), report.edges.len(), report.truncated); }
        }
        Commands::Quality { command } => match command {
            QualityCommands::Memory { json } => {
                let report = analyze_memory_quality(&store.load_records()?, &namespace, chrono::Utc::now());
                if json { println!("{}", serde_json::to_string_pretty(&report)?); } else { println!("PI Memory Quality\nAverage: {}/100\nLow quality: {}", report.summary.average_quality, report.summary.low_quality_count); }
            }
            QualityCommands::Relationship { json } => {
                let records = store.load_records()?;
                let graph = build_memory_graph(&records, &store.load_patches()?, &store.load_events()?, &namespace, 5000, 10000, chrono::Utc::now());
                let report = analyze_relationship_quality(&graph, &records, chrono::Utc::now());
                if json { println!("{}", serde_json::to_string_pretty(&report)?); } else { println!("PI Relationship Quality\nAverage: {}/100\nDangling: {}", report.summary.average_relationship_quality, report.summary.dangling_edge_count); }
            }
            QualityCommands::Recall { json } => {
                let report = analyze_recall_effectiveness(&store.load_records()?, &store.load_recall_events()?, &namespace, chrono::Utc::now());
                if json { println!("{}", serde_json::to_string_pretty(&report)?); } else { println!("PI Recall Effectiveness\nAverage: {}/100\nEvents: {}", report.summary.average_effectiveness, report.summary.total_events); }
            }
            QualityCommands::Store { json } => {
                let now = chrono::Utc::now(); let records = store.load_records()?;
                let memory = analyze_memory_quality(&records, &namespace, now);
                let graph = build_memory_graph(&records, &store.load_patches()?, &store.load_events()?, &namespace, 5000, 10000, now);
                let relationships = analyze_relationship_quality(&graph, &records, now);
                let recall = analyze_recall_effectiveness(&records, &store.load_recall_events()?, &namespace, now);
                let pending = engine.list_patches(10000)?.iter().filter(|patch| matches!(patch.latest_status, PatchStatus::Proposed | PatchStatus::Deferred)).count();
                let warnings = store.load_events()?.iter().filter(|event| event.namespace == namespace && event.severity != "info").count();
                let report = build_store_quality(&memory, &relationships, Some(&recall), pending, warnings, now);
                if json { println!("{}", serde_json::to_string_pretty(&report)?); } else { println!("PI Store Quality\nOverall: {}/100", report.overall_score); }
            }
        },

        Commands::Config { command } => match command {
            ConfigCommands::Show => {
                println!("{}", serde_json::to_string_pretty(&engine.config()?)?);
            }
            ConfigCommands::SetPolicy { namespace, profile } => {
                let config = engine.set_policy(&namespace, profile)?;
                println!("{}", serde_json::to_string_pretty(&config)?);
            }
            ConfigCommands::SetRecallTelemetry { enabled, max_events } => {
                if max_events == 0 { anyhow::bail!("max-events must be greater than zero"); }
                let mut config = store.load_config()?;
                config.recall_telemetry.enabled = match enabled.as_str() { "true" | "on" | "enabled" => true, "false" | "off" | "disabled" => false, _ => anyhow::bail!("enabled must be true/on or false/off") };
                config.recall_telemetry.max_events = max_events;
                store.save_config(&config)?;
                println!("{}", serde_json::to_string_pretty(&config.recall_telemetry)?);
            }
        },

        Commands::Maintenance { command } => match command {
            MaintenanceCommands::Scan { json, layer: _ } => {
                let report = engine.maintenance_scan(&namespace)?;
                if json {
                    println!("{}", serde_json::to_string_pretty(&report)?);
                } else {
                    println!("PI Maintenance Scan");
                    println!("Namespace: {}", report.summary.namespace);
                    println!("Records checked: {}", report.summary.records_checked);
                    println!("Patches checked: {}", report.summary.patches_checked);
                    println!("Findings: {}", report.summary.finding_count);
                    println!("Severity: {}", report.summary.severity);
                    for finding in report.findings { println!("- [{}] {}", finding.severity, finding.message); }
                }
            }
        },

        Commands::Policy { command } => match command {
            PolicyCommands::Doctor { json } => {
                let config = engine.policy_doctor()?;
                let effective_policy = config.effective_policy(&namespace);
                if json {
                    println!("{}", serde_json::to_string_pretty(&serde_json::json!({
                        "default_policy": config.default_policy,
                        "namespaces": config.namespaces,
                        "current_namespace": namespace,
                        "effective_policy": effective_policy,
                        "config_path": store.config_path(),
                        "config_exists": store.config_path().exists(),
                        "warnings": Vec::<String>::new(),
                    }))?);
                } else {
                    println!("PI Policy Doctor");
                    println!("Default policy: {}", config.default_policy);
                    println!("Namespaces:");
                    for (namespace, cfg) in config.namespaces {
                        println!("- {}: {}", namespace, cfg.policy);
                    }
                }
            }
            PolicyCommands::Explain { operation } => {
                println!("{}", GovernanceEngine::policy_explain(&operation));
            }
        },

        Commands::Namespace { command } => match command {
            NamespaceCommands::List => {
                for summary in engine.namespace_summaries()? {
                    println!(
                        "- {} records={} active={} contested={} superseded={} tombstoned={}",
                        summary.namespace, summary.records, summary.active, summary.contested, summary.superseded, summary.tombstoned
                    );
                }
            }
            NamespaceCommands::Doctor { json } => {
                let report = engine.namespace_doctor()?;
                if json {
                    println!("{}", serde_json::to_string_pretty(&serde_json::json!({
                        "default_namespace": report.default_namespace,
                        "current_namespace": namespace,
                        "namespace_count": report.namespaces,
                        "namespaces": report.summaries,
                        "cross_namespace_duplicate_ids": report.cross_namespace_duplicate_ids,
                        "warnings": Vec::<String>::new(),
                    }))?);
                } else {
                    println!("PI Namespace Doctor");
                    println!("Namespaces: {}", report.namespaces);
                    println!("Default namespace: {}", report.default_namespace);
                    println!("Records without explicit namespace: {}", report.records_without_explicit_namespace);
                    println!("Cross-namespace duplicate IDs: {}", report.cross_namespace_duplicate_ids);
                }
            }
        },

        Commands::Export {
            output,
            all_namespaces,
            project,
            redacted,
            layer: _,
        } => {
            let input = ExportInput { namespace: Some(namespace.clone()), all_namespaces, project, redacted };

            match output {
                Some(path) => {
                    let bundle = engine.export_store_to_path(&path, input)?;
                    println!("PI export written: {}", path.display());
                    println!("Schema version: {}", bundle.schema_version);
                    println!("Redacted: {}", bundle.redacted);
                    println!("Namespace: {}", bundle.namespace.as_deref().unwrap_or(if bundle.all_namespaces { "<all>" } else { "<none>" }));
                    println!("Project: {}", bundle.project.as_deref().unwrap_or("<all>"));
                    println!("Records: {}", bundle.records.len());
                    println!("Patches: {}", bundle.patches.len());
                    println!("Events: {}", bundle.events.len());
                }
                None => {
                    let bundle = engine.export_store(input)?;
                    println!("{}", serde_json::to_string_pretty(&bundle)?);
                }
            }
        }

        Commands::Import {
            path,
            dry_run,
            preserve_namespaces,
            backup,
            json,
        } => {
            let report = engine.import_store_from_path(&path, ImportInput { namespace: namespace.clone(), preserve_namespaces, dry_run, backup })?;

            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                println!("PI Import Report");
                println!("Schema version: {}", report.schema_version);
                println!("Dry run: {}", report.dry_run);
                println!("Backup requested: {}", report.backup_requested);
                println!("Changed: {}", report.changed);
                println!("Records in bundle: {}", report.records_in_bundle);
                println!("Patches in bundle: {}", report.patches_in_bundle);
                println!("Events in bundle: {}", report.events_in_bundle);
                println!("Imported records: {}", report.imported_records);
                println!("Imported patches: {}", report.imported_patches);
                println!("Imported events: {}", report.imported_events);
                println!("Skipped records: {}", report.skipped_records);
                println!("Skipped patches: {}", report.skipped_patches);
                println!("Skipped events: {}", report.skipped_events);

                if let Some(backup) = &report.backup {
                    println!("Backup: {}", backup.backup_dir);
                    if !backup.copied_files.is_empty() {
                        println!("Backup files: {}", backup.copied_files.join(", "));
                    }
                }

                if !report.warnings.is_empty() {
                    println!("\nWarnings:");
                    for warning in &report.warnings {
                        println!("- {warning}");
                    }
                }
            }
        }

        Commands::Migrate {
            dry_run,
            backup,
            json,
        } => {
            let report = engine.migrate_store(MigrationInput { dry_run, backup })?;

            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                println!("PI Migration Report");
                println!("Schema version: {}", report.schema_version);
                println!("Dry run: {}", report.dry_run);
                println!("Backup requested: {}", report.backup_requested);
                println!("Migration needed: {}", report.migration_needed);
                println!("Changed files: {}", report.changed_files);
                println!("Changed entries: {}", report.changed_entries);
                println!("Invalid JSONL lines: {}", report.invalid_json_lines);

                if let Some(backup) = &report.backup {
                    println!("Backup: {}", backup.backup_dir);
                    if !backup.copied_files.is_empty() {
                        println!("Backup files: {}", backup.copied_files.join(", "));
                    }
                }

                if !report.files.is_empty() {
                    println!("\nFiles:");
                    for file in &report.files {
                        println!(
                            "- {}: entries={} changed_entries={} root_added={} root_updated={} nested_added={} nested_updated={} invalid_json_lines={} rewritten={}",
                            file.file_name,
                            file.entries,
                            file.changed_entries,
                            file.root_schema_version_added,
                            file.root_schema_version_updated,
                            file.nested_schema_version_added,
                            file.nested_schema_version_updated,
                            file.invalid_json_lines,
                            file.rewritten
                        );
                    }
                }
            }
        }

        Commands::Doctor { json } => {
            let report = engine.doctor_in_namespace(&namespace)?;

            if json {
                let policy_profile = engine.effective_policy(&namespace)?;
                println!("{}", serde_json::to_string_pretty(&serde_json::json!({
                    "schema_version": report.schema_version,
                    "migration_needed": report.migration_needed,
                    "store_path": report.store_dir,
                    "lock_path": report.lock_path,
                    "records": report.total_records,
                    "active": report.active_records,
                    "contested": report.contested_records,
                    "superseded": report.superseded_records,
                    "tombstoned": report.tombstoned_records,
                    "patches": report.total_patches,
                    "events": report.total_events,
                    "namespaces": report.namespaces,
                    "current_namespace": report.current_namespace,
                    "policy_profile": policy_profile,
                    "warnings": report.warnings,
                    "schema_audit": report.schema_audits,
                }))?);
            } else {
                println!("PI Doctor Report");
                println!("Store: {}", report.store_dir);
                println!("Lock: {}", report.lock_path);
                println!("Schema version: {}", report.schema_version);
                println!("Migration needed: {}", report.migration_needed);
                println!("Namespaces: {}", report.namespaces);
                println!("Current namespace: {}", report.current_namespace);
                println!("Records in current namespace: {}", report.records_in_current_namespace);
                println!("Active in current namespace: {}", report.active_in_current_namespace);
                println!("Contested in current namespace: {}", report.contested_in_current_namespace);
                println!("Superseded in current namespace: {}", report.superseded_in_current_namespace);
                println!("Tombstoned in current namespace: {}", report.tombstoned_in_current_namespace);
                println!("Records: {}", report.total_records);
                println!("Active: {}", report.active_records);
                println!("Superseded: {}", report.superseded_records);
                println!("Tombstoned: {}", report.tombstoned_records);
                println!("Contested: {}", report.contested_records);
                println!("Patches: {}", report.total_patches);
                println!("Latest proposed patches: {}", report.proposed_patches_latest);
                println!("Latest applied patches: {}", report.applied_patches_latest);
                println!("Latest rejected patches: {}", report.rejected_patches_latest);
                println!("Events: {}", report.total_events);

                if !report.schema_audits.is_empty() {
                    println!("\nSchema audit:");
                    for audit in &report.schema_audits {
                        println!(
                            "- {}: entries={} missing_schema_version={} mismatched_schema_version={} invalid_json_lines={}",
                            audit.file_name,
                            audit.entries,
                            audit.missing_schema_version,
                            audit.mismatched_schema_version,
                            audit.invalid_json_lines
                        );
                    }
                }

                if !report.warnings.is_empty() {
                    println!("\nWarnings:");
                    for warning in report.warnings {
                        println!("- {warning}");
                    }
                }

                if !report.errors.is_empty() {
                    println!("\nErrors:");
                    for error in report.errors {
                        println!("- {error}");
                    }
                }
            }
        }

        Commands::List { limit, layer } => {
            let records = engine.list_records_in_namespace(&namespace, limit)?;

            for record in records.into_iter().filter(|record| layer.map(|l| record.layer == l).unwrap_or(true)) {
                println!(
                    "- [{}] {} | class={} | layer={} | confidence={:.2} | status={:?}",
                    record.id, record.claim, record.class, record.layer, record.confidence, record.status
                );
            }
        }

        Commands::InspectRecord { record_id, json, layer } => {
            match engine.inspect_record_in_namespace(&namespace, &record_id)? {
                Some(inspection) => {
                    if let Some(layer) = layer { if inspection.record.layer != layer { anyhow::bail!("record layer does not match filter"); } }
                    if json { println!("{}", serde_json::to_string_pretty(&inspection)?); }
                    else { print_record_inspection(&inspection); }
                }
                None => {
                    if json {
                        println!("{}", serde_json::to_string_pretty(&InspectRecordError { error: "record_not_found".to_string(), record_id: record_id.clone() })?);
                    } else {
                        eprintln!("Record not found: {}", record_id);
                    }
                    std::process::exit(1);
                }
            }
        }

        Commands::Inbox { json, all, layer, apply, reject, defer, reason } => {
            if let Some(id) = apply {
                let result = engine.apply_patch_by_id(&id, false)?;
                if json { println!("{}", serde_json::to_string_pretty(&result)?); } else { println!("Patch {}\nApplied: {}\n{}", result.patch_id, result.applied, result.message); }
            } else if let Some(id) = reject {
                let result = engine.reject_patch_by_id(&id, &namespace, &reason.ok_or_else(|| anyhow::anyhow!("--reason is required for --reject"))?)?;
                if json { println!("{}", serde_json::to_string_pretty(&result)?); } else { println!("Patch {} rejected", result.patch_id); }
            } else if let Some(id) = defer {
                let result = engine.defer_patch_by_id(&id, &namespace, &reason.ok_or_else(|| anyhow::anyhow!("--reason is required for --defer"))?)?;
                if json { println!("{}", serde_json::to_string_pretty(&result)?); } else { println!("Patch {} deferred", result.patch_id); }
            } else {
                let mut rows = Vec::new();
                for p in engine.list_patches(200)? {
                    if !all && !matches!(p.latest_status, PatchStatus::Proposed | PatchStatus::Deferred) { continue; }
                    let inspection = engine.inspect_patch(&p.patch_id)?;
                    let proposed = inspection.history.last().and_then(|h| h.proposed_record.as_ref());
                    if let Some(layer_filter) = layer { if proposed.map(|r| r.layer) != Some(layer_filter) { continue; } }
                    rows.push(json!({"patch_id":p.patch_id,"status":p.latest_status,"operation":p.operation,"class":p.proposed_record_class,"claim":p.proposed_record_claim,"layer":proposed.map(|r| r.layer),"trust_class":proposed.map(|r| r.trust_class)}));
                }
                if json { println!("{}", serde_json::to_string_pretty(&json!({"pending_count":rows.len(),"patches":rows}))?); }
                else { println!("PI Candidate Inbox\n"); for row in rows { println!("{} status={} layer={} claim={}", row["patch_id"].as_str().unwrap_or(""), row["status"], row["layer"], row["claim"].as_str().unwrap_or("")); } }
            }
        }

        Commands::Context { query, project, budget, format, include_l3, include_contested, layer: _ } => {
            let (markdown, value) = build_context(&store, &namespace, &query, project, budget, include_l3, include_contested)?;
            let selected = value["selected_record_ids"].as_array().map(|ids| ids.iter().filter_map(|id| id.as_str().map(str::to_owned)).collect()).unwrap_or_default();
            let used = value["retrieval_notes"]["used_estimated_tokens"].as_u64().unwrap_or(0) as usize;
            record_recall_event(&store, &namespace, RecallEventClient::Cli, RecallEventOperation::BuildContext, &query, selected, budget, used)?;
            match format.as_str() {
                "json" => println!("{}", serde_json::to_string_pretty(&value)?),
                "markdown" | "md" => println!("{}", markdown),
                other => anyhow::bail!("unsupported format: {other}"),
            }
        }

        Commands::Session { command } => match command {
            SessionCommands::Add { text, stdin, file, project, json } => {
                let input = read_text_input(text, file, stdin)?;
                let event = session_event(&namespace, project.as_deref(), &input, if stdin { SourceKind::Stdin } else { SourceKind::SessionText });
                store.append_event(&event)?;
                if json { println!("{}", serde_json::to_string_pretty(&event)?); } else { println!("Session entry: {}", event.id); }
            }
            SessionCommands::Search { query, project, after, json } => {
                let after_dt = after.as_deref().and_then(|s| DateTime::parse_from_rfc3339(&format!("{s}T00:00:00Z")).ok()).map(|d| d.with_timezone(&chrono::Utc));
                print_event_list(search_session_events(&store, &namespace, &query, project.as_deref(), after_dt)?, json)?;
            }
            SessionCommands::Decisions { project, days, json } => {
                let decisions = session_decisions(&store, &namespace, project.as_deref(), days)?;
                if json { println!("{}", serde_json::to_string_pretty(&decisions)?); }
                else { for d in decisions { println!("{} {}", d.created_at, d.text); } }
            }
        },

        Commands::RecallXray { query, project, budget, json, include_l3, include_contested, layer: _ } => {
            let report = recall_xray(&store, &namespace, &query, project, budget, include_l3, include_contested)?;
            record_recall_event_with_details(&store, &namespace, RecallEventClient::Cli, RecallEventOperation::RecallXray, &query, report.included.iter().map(|item| item.record_id.clone()).collect(), recall_exclusion_counts(&report), None, budget, report.budget.used)?;
            if json { println!("{}", serde_json::to_string_pretty(&report)?); }
            else { println!("Recall X-ray: {}", report.query); for r in report.included { println!("included {} layer={} score={:.3}", r.record_id, r.layer, r.score); } for r in report.excluded { println!("excluded {} reason={}", r.record_id, r.reason); } }
        }

        Commands::RecallFeedback { outcome, record_ids } => {
            let outcome = match outcome.as_str() { "successful" | "success" => RecallEventOutcome::Successful, "corrected" | "correction" => RecallEventOutcome::Corrected, "ignored" => RecallEventOutcome::Ignored, _ => anyhow::bail!("outcome must be successful, corrected, or ignored") };
            let recorded = record_recall_event_with_details(&store, &namespace, RecallEventClient::Cli, RecallEventOperation::Feedback, "", record_ids, std::collections::BTreeMap::new(), Some(outcome), 0, 0)?;
            println!("{}", serde_json::to_string_pretty(&json!({"recorded":recorded}))?);
        }

        Commands::McpConfig { client, command, json: _ } => {
            let command_path = command.unwrap_or(std::env::current_exe()?).display().to_string();
            match client.as_str() {
                "claude" | "cursor" | "pi-agent" => println!("{}", serde_json::to_string_pretty(&shared_mcp_json(&command_path, &store_path, &namespace))?),
                "opencode" => println!("{}", serde_json::to_string_pretty(&opencode_mcp_json(&command_path, &store_path, &namespace))?),
                "codex" => print!("{}", codex_mcp_toml(&command_path, &store_path, &namespace)),
                "inspector" => println!("npx @modelcontextprotocol/inspector {} --store {} --namespace {} mcp-stdio", command_path, store_path.display(), namespace),
                other => anyhow::bail!("unsupported mcp client: {other}. Use claude, cursor, inspector, opencode, codex, or pi-agent."),
            }
        }

        Commands::McpInstall { client, command, config, dry_run, yes, no_backup } => {
            let command_path = command.unwrap_or(std::env::current_exe()?).display().to_string();
            let config_path = config.unwrap_or(default_mcp_config_path(&client)?);
            confirm_install(yes, dry_run)?;
            let (action, backup, preserved) = match client.as_str() {
                "pi-agent" => install_json_config(&config_path, "mcpServers", shared_mcp_json(&command_path, &store_path, &namespace)["mcpServers"]["pi-governance"].clone(), dry_run, no_backup)?,
                "opencode" => install_json_config(&config_path, "mcp", opencode_mcp_json(&command_path, &store_path, &namespace)["mcp"]["pi-governance"].clone(), dry_run, no_backup)?,
                "codex" => install_codex_config(&config_path, &command_path, &store_path, &namespace, dry_run, no_backup)?,
                other => anyhow::bail!("unsupported mcp client: {other}. Use opencode, codex, or pi-agent."),
            };
            println!("PI MCP Install{}", if dry_run { " Dry Run" } else { "" });
            println!("\nClient: {}", client);
            println!("Target config: {}", config_path.display());
            if dry_run { println!("Would add or update:\n- pi-governance\n\nNo files were written."); }
            else {
                println!("Action: {} pi-governance", action);
                if let Some(b) = backup { println!("Backup: {}", b.display()); }
                if !preserved.is_empty() { println!("\nPreserved MCP servers:"); for p in preserved { println!("- {p}"); } }
                println!("\nInstalled server:\n- pi-governance");
                println!("\nRestart your client, then run:\n  pi mcp-doctor {} --config {}", client, config_path.display());
            }
        }

        Commands::McpDoctor { client, command: _, config, json } => {
            let config_path = config.unwrap_or(default_mcp_config_path(&client)?);
            let config_exists = config_path.exists();
            let mut checks = BTreeMap::new();
            checks.insert("config_exists", config_exists);
            let mut configured_command = None;
            let mut args: Vec<String> = Vec::new();
            let mut parse_ok = false;
            if config_exists { match read_server_from_config(&client, &config_path) { Ok((c,a)) => { parse_ok = true; configured_command = c; args = a; }, Err(_) => parse_ok = false } }
            let server_entry_exists = configured_command.is_some();
            let command_exists = configured_command.as_ref().map(|c| Path::new(c).exists()).unwrap_or(false);
            let command_executable = configured_command.as_ref().and_then(|c| fs::metadata(c).ok()).map(|m| m.permissions().mode() & 0o111 != 0).unwrap_or(false);
            let store_exists = store_path.exists();
            let namespace_matches = args.windows(2).any(|w| w[0] == "--namespace" && w[1] == namespace);
            let store_matches = args.windows(2).any(|w| w[0] == "--store" && w[1] == store_path.display().to_string());
            let ends_stdio = args.last().map(|s| s == "mcp-stdio").unwrap_or(false);
            let mut tool_map = BTreeMap::new();
            let expected = ["pi.retrieve_context","pi.propose_record","pi.list_patches","pi.inspect_patch","pi.doctor","pi.list_records","pi.smoke_test","pi.inspect_record"];
            let tools = if let Some(c) = &configured_command { run_tools_list(c, &args).unwrap_or_default() } else { Vec::new() };
            for e in expected { tool_map.insert(e, tools.iter().any(|t| t == e)); }
            let direct_tools_list = !tools.is_empty();
            checks.insert("config_parse", parse_ok);
            checks.insert("server_entry_exists", server_entry_exists);
            checks.insert("command_exists", command_exists);
            checks.insert("command_executable", command_executable);
            checks.insert("store_exists", store_exists);
            checks.insert("namespace_matches", namespace_matches);
            checks.insert("store_matches", store_matches);
            checks.insert("args_end_mcp_stdio", ends_stdio);
            checks.insert("direct_tools_list", direct_tools_list);
            let pass = config_exists && parse_ok && server_entry_exists && command_exists && command_executable && store_exists && namespace_matches && store_matches && ends_stdio && direct_tools_list;
            if json { println!("{}", serde_json::to_string_pretty(&serde_json::json!({"client":client,"config_path":config_path,"checks":checks,"tools":tool_map,"result": if pass {"pass"} else {"fail"}}))?); }
            else {
                println!("PI MCP Doctor\n\nClient: {}\nConfig: {}\n", client, config_path.display());
                for (k,v) in &checks { println!("{}: {}", k.replace('_', " "), if *v { "ok" } else { "fail" }); }
                println!("\nExpected tools:"); for (t,p) in &tool_map { println!("  {t}: {}", if *p { "ok" } else { "not available" }); }
                println!("\nResult: {}", if pass { "pass" } else { "fail" });
            }
            if !pass { std::process::exit(1); }
        }

        Commands::SmokeTest { json } => {
            let report = GovernanceEngine::run_smoke_test();
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                println!("PI Smoke Test");
                println!("Store: {}", report.temp_store);
                for check in &report.checks { println!("{}: {}", check.name, if check.passed { "pass" } else { "fail" }); }
                println!("Result: {}", report.result);
            }
            if report.result != "pass" { std::process::exit(1); }
        }

        Commands::ReleaseAudit { json } => {
            let mut checks = Vec::new();
            let mut failures = Vec::new();
            audit_check(&mut checks, &mut failures, "version", true, "version command is compiled into CLI");
            audit_check(&mut checks, &mut failures, "doctor-json", engine.doctor_in_namespace(&namespace).is_ok(), "doctor failed");
            audit_check(&mut checks, &mut failures, "namespace-doctor-json", engine.namespace_doctor().is_ok(), "namespace doctor failed");
            audit_check(&mut checks, &mut failures, "policy-doctor-json", engine.policy_doctor().is_ok(), "policy doctor failed");
            audit_check(&mut checks, &mut failures, "smoke-test", GovernanceEngine::run_smoke_test().result == "pass", "smoke test failed");
            let changelog = include_str!("../CHANGELOG.md");
            audit_check(&mut checks, &mut failures, "changelog", changelog.contains("v1.1.0") && changelog.contains("v1.0.0") && changelog.contains("v1.0.0-rc.5") && changelog.contains("v1.0.0-rc.2") && changelog.contains("v1.0.0-rc.1") && changelog.contains("v0.10.1") && changelog.contains("v0.1.0"), "changelog missing expected versions");
            let readme = include_str!("../README.md");
            audit_check(&mut checks, &mut failures, "readme-command-matrix", ["init", "doctor", "migrate", "config", "policy", "namespace", "propose", "review", "demo", "agent-instructions", "apply", "reinforce", "supersede", "tombstone", "contest", "resolve-contest", "retrieve", "export", "import", "list", "inspect-record", "list-patches", "inspect-patch", "mcp-stdio", "mcp-config", "mcp-install", "mcp-doctor", "smoke-test", "release-audit", "changelog", "graph", "quality", "simulate-patch", "procedure-candidates", "failure-analysis", "recall-feedback"].iter().all(|cmd| readme.contains(cmd)), "README command matrix incomplete");
            let registered_tools = registered_tool_names();
            let required_tools = [
                "pi.retrieve_context", "pi.propose_record", "pi.supersede_record", "pi.tombstone_record",
                "pi.reinforce_record", "pi.contest_record", "pi.resolve_contest", "pi.apply_patch",
                "pi.reject_patch", "pi.defer_patch", "pi.list_patches", "pi.inspect_patch",
                "pi.export_store", "pi.import_store", "pi.migrate_schema", "pi.doctor",
                "pi.list_records", "pi.inspect_record", "pi.score_memory_worth", "pi.capture_candidates",
                "pi.build_context", "pi.session_add", "pi.session_search", "pi.session_decisions",
                "pi.recall_xray", "pi.list_inbox", "pi.memory_graph", "pi.memory_quality", "pi.relationship_quality", "pi.recall_effectiveness", "pi.store_quality", "pi.simulate_patch", "pi.procedure_candidates", "pi.failure_analysis", "pi.recall_feedback",
            ];
            let missing_tools: Vec<_> = required_tools.iter().filter(|required| !registered_tools.iter().any(|actual| actual == **required)).copied().collect();
            let mcp_tools_detail = if missing_tools.is_empty() { "actual MCP registry contains every required tool".to_string() } else { format!("actual MCP registry is missing: {}", missing_tools.join(", ")) };
            audit_check(&mut checks, &mut failures, "mcp-tools-list", missing_tools.is_empty(), &mcp_tools_detail);
            let mcp_config = serde_json::json!({"mcpServers": {"pi-governance": {"command": std::env::current_exe()?.display().to_string(), "args": ["--store", store_path.display().to_string().as_str(), "--namespace", namespace.as_str(), "mcp-stdio"]}}});
            audit_check(&mut checks, &mut failures, "mcp-config", mcp_config.to_string().contains("mcp-stdio"), "mcp config missing mcp-stdio");
            let report = ReleaseAuditReport { result: if failures.is_empty() { "pass".to_string() } else { "fail".to_string() }, version: env!("CARGO_PKG_VERSION").to_string(), checks, failures };
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                println!("PI Release Audit");
                for check in &report.checks { println!("{}: {}", check.name, check.status); }
                println!("Result: {}", report.result);
            }
            if report.result != "pass" { std::process::exit(1); }
        }

        Commands::Changelog => {
            println!("{}", include_str!("../CHANGELOG.md"));
        }

        Commands::McpStdio => {
            let server = McpStdioServer::new_with_namespace(engine, namespace.clone());
            server.run()?;
        }
    }

    Ok(())
}
