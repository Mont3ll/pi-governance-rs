use pi_core::{EvidenceKind, EvidenceRef, Record, RecordClass, RecordStatus, RetrievalFormat, RetrievalOptions, Scope};
use pi_retrieval::retrieve_with_options;

fn rec(class: RecordClass, claim: &str, project: Option<&str>, tags: Vec<&str>, confidence: f32, status: RecordStatus) -> Record {
    let mut record = Record::new(
        class,
        claim,
        confidence,
        project.map(Scope::project).unwrap_or_else(Scope::global),
        tags.into_iter().map(str::to_string).collect(),
        vec![EvidenceRef::new(EvidenceKind::Conversation, "test:evidence")],
    );
    record.status = status;
    record
}

fn opts(query: &str) -> RetrievalOptions {
    RetrievalOptions {
        query: query.to_string(),
        project: Some("pi-governance-rs".to_string()),
        budget: 1200,
        format: RetrievalFormat::Json,
        explain: true,
        classes: Vec::new(),
        include_global: true,
        include_contested: false,
        min_confidence: None,
    }
}

#[test]
fn retrieval_excludes_non_active_records_by_default() {
    let records = vec![
        rec(RecordClass::Requirement, "belief revision active", Some("pi-governance-rs"), vec!["belief"], 0.8, RecordStatus::Active),
        rec(RecordClass::Requirement, "belief revision tombstoned", Some("pi-governance-rs"), vec!["belief"], 0.8, RecordStatus::Tombstoned),
        rec(RecordClass::Requirement, "belief revision superseded", Some("pi-governance-rs"), vec!["belief"], 0.8, RecordStatus::Superseded),
        rec(RecordClass::Requirement, "belief revision contested", Some("pi-governance-rs"), vec!["belief"], 0.8, RecordStatus::Contested),
    ];
    let bundle = retrieve_with_options(&records, opts("belief revision"));
    assert_eq!(bundle.records.len(), 1);
    assert_eq!(bundle.records[0].record.status, RecordStatus::Active);
}

#[test]
fn retrieval_includes_contested_when_requested() {
    let records = vec![rec(RecordClass::Requirement, "belief revision contested", Some("pi-governance-rs"), vec!["belief"], 0.8, RecordStatus::Contested)];
    let mut options = opts("belief revision");
    options.include_contested = true;
    let bundle = retrieve_with_options(&records, options);
    assert_eq!(bundle.records.len(), 1);
    assert_eq!(bundle.records[0].record.status, RecordStatus::Contested);
}

#[test]
fn class_and_confidence_filters_limit_results() {
    let records = vec![
        rec(RecordClass::Requirement, "belief revision strong", Some("pi-governance-rs"), vec!["belief"], 0.9, RecordStatus::Active),
        rec(RecordClass::Workflow, "belief revision workflow", Some("pi-governance-rs"), vec!["belief"], 0.9, RecordStatus::Active),
        rec(RecordClass::Requirement, "belief revision weak", Some("pi-governance-rs"), vec!["belief"], 0.4, RecordStatus::Active),
    ];
    let mut options = opts("belief revision");
    options.classes = vec![RecordClass::Requirement];
    options.min_confidence = Some(0.6);
    let bundle = retrieve_with_options(&records, options);
    assert_eq!(bundle.records.len(), 1);
    assert_eq!(bundle.records[0].record.claim, "belief revision strong");
}

#[test]
fn higher_query_tag_project_match_ranks_above_lower_match() {
    let records = vec![
        rec(RecordClass::Observation, "unrelated note", None, vec![], 0.9, RecordStatus::Active),
        rec(RecordClass::Requirement, "belief revision workflow for deterministic retrieval", Some("pi-governance-rs"), vec!["belief", "revision"], 0.7, RecordStatus::Active),
    ];
    let bundle = retrieve_with_options(&records, opts("belief revision"));
    assert_eq!(bundle.records[0].record.claim, "belief revision workflow for deterministic retrieval");
}

#[test]
fn json_explain_output_includes_score_breakdown() {
    let records = vec![rec(RecordClass::Requirement, "belief revision active", Some("pi-governance-rs"), vec!["belief"], 0.8, RecordStatus::Active)];
    let bundle = retrieve_with_options(&records, opts("belief revision"));
    let json = serde_json::to_value(&bundle).unwrap();
    assert!(json["records"][0]["score_breakdown"].is_object() || json["records"][0]["breakdown"].is_object());
}
