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
    let expected: Value = serde_json::from_slice(
        &fs::read(fixture_path("reconciliation-expected.json")).unwrap(),
    )
    .unwrap();

    assert_eq!(serde_json::to_value(report).unwrap(), expected);
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
