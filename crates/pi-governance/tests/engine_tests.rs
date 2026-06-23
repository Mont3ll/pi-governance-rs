use pi_core::{EvidenceKind, EvidenceRef, RecordClass, Scope};
use pi_governance::{
    GovernanceEngine, MigrationInput, ProposalInput, ReinforceInput, SupersedeInput, TombstoneInput,
};
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

#[test]
fn belief_revision_supersedes_reinforces_and_tombstones_records() -> anyhow::Result<()> {
    let root = temp_store_dir("belief-revision");
    let engine = GovernanceEngine::new(JsonlStore::new(&root));
    engine.init()?;

    let original = engine.propose_record(
        ProposalInput {
            class: RecordClass::Requirement,
            claim: "Belief revision should be represented through governed patches.".to_string(),
            confidence: 0.70,
            scope: Scope::project("pi-governance-rs"),
            tags: vec!["belief-revision".to_string()],
            evidence_refs: vec![EvidenceRef::new(EvidenceKind::Conversation, "conversation:original")],
            reason: Some("original claim".to_string()),
        },
        true,
        false,
    )?;

    let original_id = original.record_id.expect("original record id");

    let reinforce = engine.reinforce_record(
        ReinforceInput {
            target_id: original_id.clone(),
            evidence_refs: vec![EvidenceRef::new(EvidenceKind::Test, "test:reinforcement")],
            reason: "additional evidence supports the claim".to_string(),
        },
        true,
        false,
    )?;

    assert!(reinforce.applied);

    let reinforced = engine
        .list_records(10)?
        .into_iter()
        .find(|record| record.id == original_id)
        .expect("reinforced record should exist");

    assert!(reinforced.confidence > 0.70);
    assert_eq!(reinforced.evidence.len(), 2);

    let supersede = engine.supersede_record(
        SupersedeInput {
            target_id: original_id.clone(),
            class: RecordClass::Requirement,
            claim: "Belief revision should support reinforcement, supersession, and tombstones."
                .to_string(),
            confidence: 0.82,
            scope: Scope::project("pi-governance-rs"),
            tags: vec!["belief-revision".to_string(), "supersession".to_string()],
            evidence_refs: vec![EvidenceRef::new(EvidenceKind::Conversation, "conversation:supersede")],
            reason: "the claim has been refined after implementation".to_string(),
        },
        true,
        true,
    )?;

    assert!(supersede.applied);
    let replacement_id = supersede.record_id.expect("replacement record id");

    let records = engine.list_records(20)?;
    let old = records
        .iter()
        .find(|record| record.id == original_id)
        .expect("old record should still be auditable");
    let replacement = records
        .iter()
        .find(|record| record.id == replacement_id)
        .expect("replacement record should exist");

    assert!(matches!(old.status, pi_core::RecordStatus::Superseded));
    assert!(replacement.supersedes.contains(&original_id));

    let tombstone = engine.tombstone_record(
        TombstoneInput {
            target_id: replacement_id.clone(),
            evidence_refs: vec![EvidenceRef::new(EvidenceKind::HumanReview, "review:tombstone")],
            reason: "remove refined test record after validating tombstone flow".to_string(),
        },
        true,
        true,
    )?;

    assert!(tombstone.applied);

    let tombstoned = engine
        .list_records(20)?
        .into_iter()
        .find(|record| record.id == replacement_id)
        .expect("replacement should remain in audit history");

    assert!(matches!(tombstoned.status, pi_core::RecordStatus::Tombstoned));

    fs::remove_dir_all(root)?;
    Ok(())
}
