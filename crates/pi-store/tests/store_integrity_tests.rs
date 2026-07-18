use pi_governance_core::{EvidenceKind, EvidenceRef, Record, RecordClass, RecordStatus, Scope};
use pi_governance_store::{plan_record_integrity, JsonlStore};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_store_dir(test_name: &str) -> PathBuf {
    let nonce = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    std::env::temp_dir().join(format!("pi-integrity-{test_name}-{}-{nonce}", std::process::id()))
}

fn record(namespace: &str, id: &str, status: RecordStatus, supersedes: &[&str]) -> Record {
    let mut record = Record::new(
        RecordClass::Requirement,
        format!("{id} claim"),
        0.9,
        Scope::project("demo"),
        vec!["workflow".into()],
        vec![EvidenceRef::new(
            EvidenceKind::Conversation,
            "daily:2026-07-18",
        )],
    );
    record.namespace = namespace.into();
    record.id = id.into();
    record.status = status;
    record.supersedes = supersedes.iter().map(|value| (*value).into()).collect();
    record
}

#[test]
fn keeps_last_namespaced_row_and_merges_non_self_supersedes() {
    let records = vec![
        record(
            "alpha",
            "rec_dup",
            RecordStatus::Superseded,
            &["rec_predecessor"],
        ),
        record("alpha", "rec_other", RecordStatus::Active, &[]),
        record(
            "alpha",
            "rec_dup",
            RecordStatus::Active,
            &["rec_dup", "rec_second"],
        ),
    ];

    let plan = plan_record_integrity(&records);

    assert!(plan.migration_needed);
    assert_eq!(plan.rows_before, 3);
    assert_eq!(plan.unique_keys_before, 2);
    assert_eq!(plan.rows_after, 2);
    assert_eq!(plan.duplicate_groups, 1);
    assert_eq!(plan.rows_removed, 1);
    assert_eq!(plan.self_edges_removed, 1);
    assert_eq!(plan.groups_repaired, 1);
    assert_eq!(plan.groups[0].namespace, "alpha");
    assert_eq!(plan.groups[0].id, "rec_dup");
    assert_eq!(plan.groups[0].canonical_ordinal, 3);
    assert_eq!(
        plan.groups[0].retained_supersedes,
        vec!["rec_predecessor", "rec_second"]
    );
    let repaired = plan
        .records
        .iter()
        .find(|record| record.id == "rec_dup")
        .unwrap();
    assert_eq!(repaired.status, RecordStatus::Active);
    assert_eq!(repaired.supersedes, vec!["rec_predecessor", "rec_second"]);
}

#[test]
fn same_id_in_different_namespaces_is_not_a_duplicate() {
    let records = vec![
        record("alpha", "rec_shared", RecordStatus::Active, &[]),
        record("beta", "rec_shared", RecordStatus::Active, &[]),
    ];

    let plan = plan_record_integrity(&records);

    assert!(!plan.migration_needed);
    assert_eq!(plan.rows_before, 2);
    assert_eq!(plan.unique_keys_before, 2);
    assert_eq!(plan.rows_after, 2);
    assert_eq!(plan.duplicate_groups, 0);
}

#[test]
fn apply_requires_preview_fingerprint_and_creates_backup_and_report() -> anyhow::Result<()> {
    let root = temp_store_dir("apply");
    fs::create_dir_all(&root)?;
    let records = vec![
        record("alpha", "rec_dup", RecordStatus::Superseded, &["rec_previous", "rec_dup"]),
        record("alpha", "rec_dup", RecordStatus::Active, &["rec_dup"]),
    ];
    let original = records.iter().map(serde_json::to_string).collect::<Result<Vec<_>, _>>()?.join("\n") + "\n";
    fs::write(root.join("records.jsonl"), &original)?;
    fs::write(root.join("patches.jsonl"), "")?;
    fs::write(root.join("events.jsonl"), "")?;
    let store = JsonlStore::new(&root);
    let preview = store.plan_record_integrity()?;

    let result = store.apply_record_integrity(&preview.fingerprint)?;

    assert!(result.mutation_performed);
    assert_eq!(result.rows_before, 2);
    assert_eq!(result.rows_after, 1);
    assert_eq!(fs::read_to_string(PathBuf::from(result.backup.as_ref().unwrap().backup_dir.clone()).join("records.jsonl"))?, original);
    assert!(PathBuf::from(result.report_path.as_ref().unwrap()).exists());
    assert!(!store.plan_record_integrity()?.migration_needed);
    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn apply_rejects_stale_fingerprint_without_writing() -> anyhow::Result<()> {
    let root = temp_store_dir("drift");
    fs::create_dir_all(&root)?;
    let records = vec![
        record("alpha", "rec_dup", RecordStatus::Superseded, &["rec_dup"]),
        record("alpha", "rec_dup", RecordStatus::Active, &[]),
    ];
    fs::write(root.join("records.jsonl"), records.iter().map(serde_json::to_string).collect::<Result<Vec<_>, _>>()?.join("\n") + "\n")?;
    let store = JsonlStore::new(&root);
    let preview = store.plan_record_integrity()?;
    let changed = format!("{}{}\n", fs::read_to_string(root.join("records.jsonl"))?, serde_json::to_string(&record("alpha", "rec_new", RecordStatus::Active, &[]))?);
    fs::write(root.join("records.jsonl"), &changed)?;

    let error = store.apply_record_integrity(&preview.fingerprint).unwrap_err();

    assert!(error.to_string().contains("integrity preview is stale"));
    assert_eq!(fs::read_to_string(root.join("records.jsonl"))?, changed);
    assert!(!root.join("backups").exists());
    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn singleton_self_edge_is_repaired_and_second_plan_is_clean() {
    let records = vec![record(
        "alpha",
        "rec_self",
        RecordStatus::Active,
        &["rec_self"],
    )];

    let first = plan_record_integrity(&records);
    let second = plan_record_integrity(&first.records);

    assert!(first.migration_needed);
    assert_eq!(first.rows_removed, 0);
    assert_eq!(first.self_edges_removed, 1);
    assert!(first.records[0].supersedes.is_empty());
    assert!(!second.migration_needed);
    assert_eq!(second.rows_before, second.unique_keys_before);
}
