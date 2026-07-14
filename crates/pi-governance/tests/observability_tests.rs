use chrono::{Duration, TimeZone, Utc};
use pi_governance_core::{
    Durability, EvidenceKind, EvidenceRef, Patch, Record, RecordClass, Scope,
    SourceKind, StoreEvent, TrustClass,
};
use pi_governance_engine::{
    analyze_memory_quality, analyze_relationship_quality, build_memory_graph, MemoryGraphEdgeType,
};

fn record(id: &str, claim: &str) -> Record {
    let mut record = Record::new(
        RecordClass::Workflow,
        claim,
        0.9,
        Scope::global(),
        vec!["workflow".into()],
        vec![EvidenceRef::new(EvidenceKind::File, "file:/private/secret.txt")],
    );
    record.id = id.into();
    record.trust_class = TrustClass::HumanReview;
    record.durability = Durability::LongTerm;
    record.source_kind = SourceKind::CodebaseAnalysis;
    record
}

#[test]
fn graph_is_deterministic_current_state_and_does_not_expose_evidence_uri() {
    let old = record("rec_old", "old workflow");
    let mut replacement = record("rec_new", "new workflow");
    replacement.supersedes.push(old.id.clone());
    let proposed = Patch::propose_record(replacement.clone(), "replace workflow");
    let applied = proposed.applied_copy();
    let events = vec![StoreEvent::info("patch applied", Some(proposed.id.clone()))];
    let now = Utc.with_ymd_and_hms(2026, 7, 14, 12, 0, 0).unwrap();

    let graph = build_memory_graph(
        &[replacement, old],
        &[proposed, applied],
        &events,
        "default",
        100,
        100,
        now,
    );

    assert!(!graph.mutation_performed);
    assert_eq!(graph.nodes.iter().filter(|node| node.node_type.as_str() == "patch").count(), 1);
    assert!(graph.edges.iter().any(|edge| edge.edge_type == MemoryGraphEdgeType::Supersedes));
    let serialized = serde_json::to_string(&graph).unwrap();
    assert!(!serialized.contains("/private/secret.txt"));
    let node_ids: Vec<_> = graph.nodes.iter().map(|node| node.id.as_str()).collect();
    let sorted = { let mut ids = node_ids.clone(); ids.sort(); ids };
    assert_eq!(node_ids, sorted);
}

#[test]
fn memory_quality_scores_governance_signals_without_claiming_evidence_liveness() {
    let now = Utc.with_ymd_and_hms(2026, 7, 14, 12, 0, 0).unwrap();
    let healthy = record("rec_healthy", "review before release");
    let mut weak = record("rec_weak", "review before release");
    weak.confidence = 0.4;
    weak.evidence.clear();
    weak.trust_class = TrustClass::Unknown;
    weak.durability = Durability::Unknown;
    weak.source_kind = SourceKind::Unknown;
    weak.updated_at = now - Duration::days(200);

    let report = analyze_memory_quality(&[healthy, weak], "default", now);

    assert!(!report.mutation_performed);
    assert_eq!(report.items[0].memory_id, "rec_weak");
    assert!(report.items[0].signals.contains(&"missing_evidence".to_string()));
    assert!(!report.items[0].signals.iter().any(|signal| signal.contains("live")));
    assert!(report.summary.average_quality < 100);
}

#[test]
fn relationship_quality_reports_dangling_and_orphan_relationships() {
    let now = Utc.with_ymd_and_hms(2026, 7, 14, 12, 0, 0).unwrap();
    let mut dangling = record("rec_dangling", "superseding workflow");
    dangling.supersedes.push("rec_missing".into());
    let orphan = {
        let mut value = record("rec_orphan", "orphan workflow");
        value.evidence.clear();
        value
    };
    let records = vec![dangling, orphan];
    let graph = build_memory_graph(&records, &[], &[], "default", 100, 100, now);

    let report = analyze_relationship_quality(&graph, &records, now);

    assert!(!report.mutation_performed);
    assert!(report.summary.dangling_edge_count > 0);
    assert!(report.summary.orphan_memory_count > 0);
}
