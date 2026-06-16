use anyhow::Result;
use clap::{Parser, Subcommand};
use pi_core::{EvidenceKind, EvidenceRef, RecordClass, Scope};
use pi_governance::{GovernanceEngine, ProposalInput};
use pi_mcp::McpStdioServer;
use pi_retrieval::render_markdown;
use pi_store::JsonlStore;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(
    name = "pi",
    version = "0.2.0",
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

        Commands::Doctor { json } => {
            let report = engine.doctor()?;

            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                println!("PI Doctor Report");
                println!("Store: {}", report.store_dir);
                println!("Records: {}", report.total_records);
                println!("Active: {}", report.active_records);
                println!("Superseded: {}", report.superseded_records);
                println!("Tombstoned: {}", report.tombstoned_records);
                println!("Patches: {}", report.total_patches);
                println!("Latest proposed patches: {}", report.proposed_patches_latest);
                println!("Latest applied patches: {}", report.applied_patches_latest);
                println!("Latest rejected patches: {}", report.rejected_patches_latest);
                println!("Events: {}", report.total_events);

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
