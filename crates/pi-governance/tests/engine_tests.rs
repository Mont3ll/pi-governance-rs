use pi_core::{EvidenceKind, EvidenceRef, RecordClass, Scope};
use pi_governance::{GovernanceEngine, MigrationInput, ProposalInput};
use pi_store::JsonlStore;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_store_dir(test_name: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be after unix epoch")
        .as_nanos();

    std::env::temp_dir().join(format!(
        "pi-engine-{test_name}-{}-{nonce}",
        std::process::id()
    ))
}

#[test]
fn proposes_inspects_and_applies_patch() -> anyhow::Result<()> {
    let root = temp_store_dir("patch-flow");
    let engine = GovernanceEngine::new(JsonlStore::new(&root));
    engine.init()?;

    let proposal = engine.propose_record(
        ProposalInput {
            class: RecordClass::Requirement,
            claim: "Patch visibility should expose proposed and applied states.".to_string(),
            confidence: 0.85,
            scope: Scope::project("pi-governance-rs"),
            tags: vec!["testing".to_string()],
            evidence_refs: vec![EvidenceRef::new(EvidenceKind::Conversation, "conversation:test")],
            reason: Some("engine test".to_string()),
        },
        false,
        false,
    )?;

    let inspection = engine.inspect_patch(&proposal.patch_id)?;
    assert!(inspection.can_apply_without_force);

    let applied = engine.apply_patch_by_id(&proposal.patch_id, false)?;
    assert!(applied.applied);

    let inspection = engine.inspect_patch(&proposal.patch_id)?;
    assert!(!inspection.can_apply_without_force);

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn migration_dry_run_reports_legacy_store_without_rewriting() -> anyhow::Result<()> {
    let root = temp_store_dir("migration-dry-run");
    fs::create_dir_all(&root)?;
    fs::write(
        root.join("events.jsonl"),
        r#"{"id":"evt_legacy","severity":"info","message":"legacy fixture","object_id":null,"created_at":"2026-01-01T00:00:00Z"}
"#,
    )?;

    let engine = GovernanceEngine::new(JsonlStore::new(&root));
    let report = engine.migrate_store(MigrationInput {
        dry_run: true,
        backup: true,
    })?;

    assert!(report.migration_needed);
    assert!(report.backup.is_none());
    assert!(!fs::read_to_string(root.join("events.jsonl"))?.contains("schema_version"));

    fs::remove_dir_all(root)?;
    Ok(())
}
