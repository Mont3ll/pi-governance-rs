use pi_governance_core::{ContestResolution, Durability, EvidenceKind, EvidenceRef, MemoryLayer, MemoryKind, RecordClass, RuleType, Scope, RecordStatus, SourceKind, TrustClass};
use pi_governance_engine::{
    ContestInput, ExportInput, GovernanceEngine, ImportInput, MigrationInput, ProposalInput, ReinforceInput, ResolveContestInput, SupersedeInput, TombstoneInput,
};
use pi_governance_store::JsonlStore;
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
            namespace: "default".to_string(),
            class: RecordClass::Requirement,
            claim: "Patch visibility should expose proposed and applied states.".to_string(),
            confidence: 0.85,
            scope: Scope::project("pi-governance-rs"),
            tags: vec!["testing".to_string()],
            evidence_refs: vec![EvidenceRef::new(EvidenceKind::Conversation, "conversation:test")],
            reason: Some("engine test".to_string()),
            layer: Some(MemoryLayer::L2Playbook),
            memory_kind: Some(MemoryKind::Instruction),
            rule_type: Some(RuleType::Testing),
            trust_class: TrustClass::DirectUserInstruction,
            durability: Durability::Project,
            source_kind: SourceKind::ManualCli,
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
            namespace: "default".to_string(),
            class: RecordClass::Requirement,
            claim: "Belief revision should be represented through governed patches.".to_string(),
            confidence: 0.70,
            scope: Scope::project("pi-governance-rs"),
            tags: vec!["belief-revision".to_string()],
            evidence_refs: vec![EvidenceRef::new(EvidenceKind::Conversation, "conversation:original")],
            reason: Some("original claim".to_string()),
            layer: Some(MemoryLayer::L2Playbook),
            memory_kind: Some(MemoryKind::Instruction),
            rule_type: Some(RuleType::Convention),
            trust_class: TrustClass::DirectUserInstruction,
            durability: Durability::Project,
            source_kind: SourceKind::ManualCli,
        },
        true,
        false,
    )?;

    let original_id = original.record_id.expect("original record id");

    let reinforce = engine.reinforce_record(
        ReinforceInput {
            namespace: "default".to_string(),
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
            namespace: "default".to_string(),
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

    assert!(matches!(old.status, pi_governance_core::RecordStatus::Superseded));
    assert!(replacement.supersedes.contains(&original_id));

    let tombstone = engine.tombstone_record(
        TombstoneInput {
            namespace: "default".to_string(),
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

    assert!(matches!(tombstoned.status, pi_governance_core::RecordStatus::Tombstoned));

    fs::remove_dir_all(root)?;
    Ok(())
}


#[test]
fn contest_and_resolve_belief_revision_flow() -> anyhow::Result<()> {
    let root = temp_store_dir("contest-resolution");
    let engine = GovernanceEngine::new(JsonlStore::new(&root));
    engine.init()?;

    let original = engine.propose_record(
        ProposalInput {
            namespace: "default".to_string(),
            class: RecordClass::Requirement,
            claim: "Contest workflows should preserve disputed records until review resolution."
                .to_string(),
            confidence: 0.70,
            scope: Scope::project("pi-governance-rs"),
            tags: vec!["contest".to_string()],
            evidence_refs: vec![EvidenceRef::new(EvidenceKind::Conversation, "conversation:contest")],
            reason: Some("original contested claim".to_string()),
            layer: Some(MemoryLayer::L2Playbook),
            memory_kind: Some(MemoryKind::Instruction),
            rule_type: Some(RuleType::Convention),
            trust_class: TrustClass::DirectUserInstruction,
            durability: Durability::Project,
            source_kind: SourceKind::ManualCli,
        },
        true,
        false,
    )?;

    let record_id = original.record_id.expect("record id");

    let contest = engine.contest_record(
        ContestInput {
            namespace: "default".to_string(),
            target_id: record_id.clone(),
            evidence_refs: vec![EvidenceRef::new(EvidenceKind::HumanReview, "review:contest")],
            reason: "reviewer found evidence that disputes this record".to_string(),
        },
        true,
        true,
    )?;

    assert!(contest.applied);

    let contested = engine
        .list_records(20)?
        .into_iter()
        .find(|record| record.id == record_id)
        .expect("contested record should remain present");

    assert!(matches!(contested.status, RecordStatus::Contested));

    let resolved = engine.resolve_contest(
        ResolveContestInput {
            namespace: "default".to_string(),
            target_id: record_id.clone(),
            resolution: ContestResolution::Uphold,
            class: None,
            claim: None,
            confidence: 0.75,
            scope: Scope::project("pi-governance-rs"),
            tags: Vec::new(),
            evidence_refs: vec![EvidenceRef::new(EvidenceKind::HumanReview, "review:uphold")],
            reason: "review confirmed this record remains valid".to_string(),
        },
        true,
        true,
    )?;

    assert!(resolved.applied);

    let upheld = engine
        .list_records(20)?
        .into_iter()
        .find(|record| record.id == record_id)
        .expect("upheld record should remain present");

    assert!(matches!(upheld.status, RecordStatus::Active));

    fs::remove_dir_all(root)?;
    Ok(())
}


#[test]
fn engine_exports_and_imports_portable_bundle() -> anyhow::Result<()> {
    let source_root = temp_store_dir("engine-export-source");
    let target_root = temp_store_dir("engine-export-target");
    let source = GovernanceEngine::new(JsonlStore::new(&source_root));
    let target = GovernanceEngine::new(JsonlStore::new(&target_root));
    source.init()?;
    target.init()?;

    let proposal = source.propose_record(
        ProposalInput {
            namespace: "default".to_string(),
            class: RecordClass::Requirement,
            claim: "Engine export should carry governed records into another store.".to_string(),
            confidence: 0.8,
            scope: Scope::project("pi-governance-rs"),
            tags: vec!["portable".to_string()],
            evidence_refs: vec![EvidenceRef::new(EvidenceKind::Test, "test:engine-export")],
            reason: Some("engine export fixture".to_string()),
            layer: Some(MemoryLayer::L2Playbook),
            memory_kind: Some(MemoryKind::Instruction),
            rule_type: Some(RuleType::Workflow),
            trust_class: TrustClass::DirectUserInstruction,
            durability: Durability::Project,
            source_kind: SourceKind::ManualCli,
        },
        true,
        false,
    )?;

    let bundle = source.export_store(ExportInput {
        namespace: Some("default".to_string()),
        all_namespaces: false,
        project: Some("pi-governance-rs".to_string()),
        redacted: true,
    })?;

    assert!(bundle.redacted);
    assert_eq!(bundle.records.len(), 1);
    assert_eq!(bundle.records[0].evidence[0].uri, "redacted:evidence");

    let dry_run = target.import_store_from_path(
        &write_bundle_fixture(&target_root, &bundle)?,
        ImportInput {
            namespace: "default".to_string(),
            preserve_namespaces: false,
            dry_run: true,
            backup: true,
        },
    )?;

    assert!(dry_run.changed);
    assert_eq!(dry_run.imported_records, 1);
    assert!(target.list_records(20)?.is_empty());

    let import_path = write_bundle_fixture(&target_root, &bundle)?;
    let imported = target.import_store_from_path(
        &import_path,
        ImportInput {
            namespace: "default".to_string(),
            preserve_namespaces: false,
            dry_run: false,
            backup: true,
        },
    )?;

    assert!(imported.changed);
    assert_eq!(imported.imported_records, 1);
    assert_eq!(target.list_records(20)?.len(), 1);
    assert_eq!(target.list_records(20)?[0].id, proposal.record_id.expect("record id"));

    fs::remove_dir_all(source_root)?;
    fs::remove_dir_all(target_root)?;
    Ok(())
}

fn write_bundle_fixture(
    root: &std::path::Path,
    bundle: &pi_governance_store::StoreExportBundle,
) -> anyhow::Result<PathBuf> {
    let path = root.join("bundle.json");
    fs::write(&path, serde_json::to_string_pretty(bundle)?)?;
    Ok(path)
}
