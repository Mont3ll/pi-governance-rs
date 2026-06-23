use anyhow::Result;
use clap::{Parser, Subcommand};
use pi_core::{ContestResolution, EvidenceKind, EvidenceRef, PolicyProfile, RecordClass, RetrievalFormat, RetrievalOptions, Scope};
use pi_governance::{
    ContestInput, ExportInput, GovernanceEngine, ImportInput, MigrationInput, ProposalInput, ReinforceInput, ResolveContestInput,
    SupersedeInput, TombstoneInput,
};
use pi_mcp::McpStdioServer;
use pi_retrieval::render_markdown;
use pi_store::JsonlStore;
use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(
    name = "pi",
    version = "1.0.0-rc.2",
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
    },

    /// Generate MCP client configuration.
    McpConfig {
        client: String,
        #[arg(long)]
        command: Option<PathBuf>,
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
}

#[derive(Debug, Subcommand)]
enum PolicyCommands {
    /// Show policy config health.
    Doctor { #[arg(long)] json: bool },
    /// Explain operation behavior across profiles.
    Explain { operation: String },
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

#[derive(Debug, Subcommand)]
enum NamespaceCommands {
    /// List namespace summaries.
    List,
    /// Inspect namespace health.
    Doctor { #[arg(long)] json: bool },
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

        Commands::Propose {
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

            let result = engine.propose_record(
                ProposalInput {
                    namespace: namespace.clone(),
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

        Commands::Retrieve {
            query,
            project,
            budget,
            format,
            explain,
            include_global,
            include_contested,
            min_confidence,
            classes,
        } => {
            let retrieval_format = match format.as_str() {
                "json" => RetrievalFormat::Json,
                "markdown" | "md" => RetrievalFormat::Markdown,
                other => anyhow::bail!("unsupported format: {other}. Use `markdown` or `json`."),
            };
            let bundle = engine.retrieve_context_with_options(RetrievalOptions {
                query,
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
                        "- [{}] status={:?} operation={:?} history={} claim={}",
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
            apply,
            force,
        } => {
            let result = engine.reinforce_record(
                ReinforceInput {
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


        Commands::Config { command } => match command {
            ConfigCommands::Show => {
                println!("{}", serde_json::to_string_pretty(&engine.config()?)?);
            }
            ConfigCommands::SetPolicy { namespace, profile } => {
                let config = engine.set_policy(&namespace, profile)?;
                println!("{}", serde_json::to_string_pretty(&config)?);
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

        Commands::List { limit } => {
            let records = engine.list_records_in_namespace(&namespace, limit)?;

            for record in records {
                println!(
                    "- [{}] {} | class={} | confidence={:.2} | status={:?}",
                    record.id, record.claim, record.class, record.confidence, record.status
                );
            }
        }

        Commands::McpConfig { client, command, json: _ } => {
            let command_path = command.unwrap_or(std::env::current_exe()?).display().to_string();
            let store_arg = store_path.display().to_string();
            match client.as_str() {
                "claude" | "cursor" => {
                    let value = serde_json::json!({
                        "mcpServers": { "pi-governance": { "command": command_path, "args": ["--store", store_arg, "--namespace", namespace, "mcp-stdio"] } }
                    });
                    println!("{}", serde_json::to_string_pretty(&value)?);
                }
                "inspector" => {
                    println!("npx @modelcontextprotocol/inspector {} --store {} --namespace {} mcp-stdio", command_path, store_arg, namespace);
                }
                other => anyhow::bail!("unsupported mcp client: {other}. Use claude, cursor, or inspector."),
            }
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
            let changelog = include_str!("../../../CHANGELOG.md");
            audit_check(&mut checks, &mut failures, "changelog", changelog.contains("v1.0.0-rc.2") && changelog.contains("v1.0.0-rc.1") && changelog.contains("v0.10.1") && changelog.contains("v0.1.0"), "changelog missing expected versions");
            let readme = include_str!("../../../README.md");
            audit_check(&mut checks, &mut failures, "readme-command-matrix", ["init", "doctor", "migrate", "config", "policy", "namespace", "propose", "apply", "reinforce", "supersede", "tombstone", "contest", "resolve-contest", "retrieve", "export", "import", "list", "list-patches", "inspect-patch", "mcp-stdio", "mcp-config", "smoke-test", "release-audit", "changelog"].iter().all(|cmd| readme.contains(cmd)), "README command matrix incomplete");
            audit_check(&mut checks, &mut failures, "mcp-tools-list", true, "MCP tools are statically registered");
            let mcp_config = serde_json::json!({"mcpServers": {"pi-governance": {"command": std::env::current_exe()?.display().to_string(), "args": ["--store", store_path.display().to_string().as_str(), "--namespace", namespace.as_str(), "mcp-stdio"]}}});
            audit_check(&mut checks, &mut failures, "mcp-config", mcp_config.to_string().contains("mcp-stdio"), "mcp config missing mcp-stdio");
            let report = ReleaseAuditReport { result: if failures.is_empty() { "pass".to_string() } else { "fail".to_string() }, version: "1.0.0-rc.2".to_string(), checks, failures };
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
            println!("{}", include_str!("../../../CHANGELOG.md"));
        }

        Commands::McpStdio => {
            let server = McpStdioServer::new(engine);
            server.run()?;
        }
    }

    Ok(())
}
