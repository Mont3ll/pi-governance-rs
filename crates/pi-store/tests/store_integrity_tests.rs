use pi_governance_core::{EvidenceKind, EvidenceRef, Record, RecordClass, RecordStatus, Scope};
use pi_governance_store::plan_record_integrity;

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
