use anyhow::Result;
use clap::{Parser, Subcommand};
use pi_core::{ContestResolution, EvidenceKind, EvidenceRef, RecordClass, Scope};
use pi_governance::{
    ContestInput, ExportInput, GovernanceEngine, ImportInput, MigrationInput, ProposalInput, ReinforceInput, ResolveContestInput,
    SupersedeInput, TombstoneInput,
};
use pi_mcp::McpStdioServer;
use pi_retrieval::render_markdown;
use pi_store::JsonlStore;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(
    name = "pi",
    version = "0.6.0",
    about = "PI governance runtime for coding agents"
)]
struct Cli {
    #[arg(long, global = true, default_value = ".pi")]
    store: PathBuf,

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


    /// Export the PI store as a portable JSON bundle.
    Export {
        /// Optional output path. If omitted, the export bundle is printed to stdout.
        #[arg(long)]
        output: Option<PathBuf>,

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

    /// Run PI as an MCP server over stdio.
    McpStdio,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let store = JsonlStore::new(cli.store);
    let engine = GovernanceEngine::new(store);

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
        } => {
            let bundle = engine.retrieve_context(query, project, budget)?;

            match format.as_str() {
                "json" => println!("{}", serde_json::to_string_pretty(&bundle)?),
                "markdown" | "md" => println!("{}", render_markdown(&bundle)),
                other => {
                    anyhow::bail!("unsupported format: {other}. Use `markdown` or `json`.");
                }
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


        Commands::Export {
            output,
            project,
            redacted,
        } => {
            let input = ExportInput { project, redacted };

            match output {
                Some(path) => {
                    let bundle = engine.export_store_to_path(&path, input)?;
                    println!("PI export written: {}", path.display());
                    println!("Schema version: {}", bundle.schema_version);
                    println!("Redacted: {}", bundle.redacted);
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
            backup,
            json,
        } => {
            let report = engine.import_store_from_path(&path, ImportInput { dry_run, backup })?;

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
            let report = engine.doctor()?;

            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                println!("PI Doctor Report");
                println!("Store: {}", report.store_dir);
                println!("Lock: {}", report.lock_path);
                println!("Schema version: {}", report.schema_version);
                println!("Migration needed: {}", report.migration_needed);
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
            let records = engine.list_records(limit)?;

            for record in records {
                println!(
                    "- [{}] {} | class={} | confidence={:.2} | status={:?}",
                    record.id, record.claim, record.class, record.confidence, record.status
                );
            }
        }

        Commands::McpStdio => {
            let server = McpStdioServer::new(engine);
            server.run()?;
        }
    }

    Ok(())
}
