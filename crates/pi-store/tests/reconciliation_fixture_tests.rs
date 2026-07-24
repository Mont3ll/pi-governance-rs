use pi_governance_core::RecordStatus;
use pi_governance_store::{reconcile_bundles, StoreExportBundle};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/pi-governance-conformance")
        .join(name)
}

fn bundle(name: &str) -> StoreExportBundle {
    serde_json::from_slice(&fs::read(fixture_path(name)).unwrap()).unwrap()
}

#[test]
fn reconciliation_fixture_report_matches_shared_contract() {
    let report = reconcile_bundles(&bundle("full-bundle.json"), &bundle("filtered-bundle.json"));
    let expected: Value =
        serde_json::from_slice(&fs::read(fixture_path("reconciliation-expected.json")).unwrap())
            .unwrap();

    assert_eq!(serde_json::to_value(report).unwrap(), expected);
}

#[test]
fn reconciliation_fixture_normalizes_sets_and_envelope_fields_only() {
    let source = bundle("full-bundle.json");
    let mut destination = source.clone();
    destination.exported_at = "2030-01-01T00:00:00Z".parse().unwrap();
    destination.producer.as_mut().unwrap().version = "99.0.0".into();
    destination.records[0].tags.reverse();
    destination.records[0].evidence.reverse();

    let normalized = reconcile_bundles(&source, &destination);
    assert!(normalized.sections["records"].divergent_ids.is_empty());
    assert!(normalized.sections["records"]
        .matching_ids
        .contains(&"rec_match".to_string()));

    destination.records[0].status = RecordStatus::Contested;
    let substantive = reconcile_bundles(&source, &destination);
    assert!(substantive.sections["records"]
        .divergent_ids
        .contains(&"rec_match".to_string()));
}

#[test]
fn reconciliation_fixture_classifies_exact_and_conflicting_duplicates() {
    let source = bundle("duplicate-input.json");
    let mut destination = source.clone();
    destination.records.clear();

    let report = reconcile_bundles(&source, &destination);
    let records = report.sections.get("records").unwrap();

    assert_eq!(
        records.source_duplicate_ids,
        vec!["rec_conflicting_duplicate", "rec_exact_duplicate"]
    );
    assert_eq!(
        records.conflicting_duplicate_ids,
        vec!["rec_conflicting_duplicate"]
    );
}
