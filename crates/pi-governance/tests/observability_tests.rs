use chrono::{Duration, TimeZone, Utc};
use pi_governance_core::{
    Durability, EvidenceKind, EvidenceRef, Patch, PatchStatus, RecallEvent, RecallEventClient,
    RecallEventOperation, Record, RecordClass, Scope, SourceKind, StoreEvent, TrustClass,
};
use pi_governance_engine::{
    analyze_failure_patterns, analyze_memory_quality, analyze_recall_effectiveness,
    analyze_relationship_quality, build_memory_graph, build_store_quality,
    generate_procedure_candidates, GovernanceEngine, MemoryGraphEdgeType,
};

fn record(id: &str, claim: &str) -> Record {
    let mut record = Record::new(
        RecordClass::Workflow,
        claim,
        0.9,
        Scope::global(),
        vec!["workflow".into()],
        vec![EvidenceRef::new(
            EvidenceKind::File,
            "file:/private/secret.txt",
        )],
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
    assert_eq!(
        graph
            .nodes
            .iter()
            .filter(|node| node.node_type.as_str() == "patch")
            .count(),
        1
    );
    assert!(graph
        .edges
        .iter()
        .any(|edge| edge.edge_type == MemoryGraphEdgeType::Supersedes));
    let serialized = serde_json::to_string(&graph).unwrap();
    assert!(!serialized.contains("/private/secret.txt"));
    let node_ids: Vec<_> = graph.nodes.iter().map(|node| node.id.as_str()).collect();
    let sorted = {
        let mut ids = node_ids.clone();
        ids.sort();
        ids
    };
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
    assert!(report.items[0]
        .signals
        .contains(&"missing_evidence".to_string()));
    assert!(!report.items[0]
        .signals
        .iter()
        .any(|signal| signal.contains("live")));
    assert!(report.summary.average_quality < 100);
}

#[test]
fn procedure_and_failure_analysis_are_report_only_and_provenance_backed() {
    let now = Utc.with_ymd_and_hms(2026, 7, 14, 12, 0, 0).unwrap();
    let records = vec![
        record("rec_test", "Run cargo test before release"),
        record("rec_audit", "Run release audit before tagging"),
    ];
    let procedures = generate_procedure_candidates(&records, "default", 2, now);
    assert!(!procedures.mutation_performed);
    assert_eq!(procedures.candidates.len(), 1);
    assert_eq!(procedures.candidates[0].source_record_ids.len(), 2);
    assert!(procedures.candidates[0].review_required);
    assert!(!procedures.candidates[0].pitfalls.is_empty());
    assert_eq!(
        procedures.candidates[0].export_status,
        pi_governance_engine::ProcedureExportStatus::ReviewRequired
    );

    let mut rejected = Patch::propose_record(record("rec_rejected", "rejected workflow"), "unsafe");
    rejected.status = PatchStatus::Rejected;
    let mut deferred = Patch::propose_record(record("rec_deferred", "deferred workflow"), "later");
    deferred.status = PatchStatus::Deferred;
    deferred.updated_at = now - Duration::days(45);
    let failures = analyze_failure_patterns(
        &[rejected, deferred],
        &[StoreEvent::warning("apply failed repeatedly", None)],
        "default",
        30,
        now,
    );
    assert!(!failures.mutation_performed);
    assert!(failures.summary.rejected_patch_count > 0);
    assert!(failures.summary.stale_deferred_patch_count > 0);
    assert!(failures.summary.warning_event_count > 0);
    assert!(failures.summary.event_category_count > 0);
}

#[test]
fn patch_simulation_reports_quality_delta_without_mutating_store() {
    let root = std::env::temp_dir().join(format!(
        "pi-simulation-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let store = pi_governance_store::JsonlStore::new(&root);
    store.init().unwrap();
    store
        .append_record(&record("rec_existing", "existing workflow"))
        .unwrap();
    let proposed = Patch::propose_record(record("rec_proposed", "proposed workflow"), "preview");
    store.append_patch(&proposed).unwrap();
    let before: Vec<_> = [
        "records.jsonl",
        "patches.jsonl",
        "events.jsonl",
        "recall-events.jsonl",
    ]
    .iter()
    .map(|name| (name.to_string(), std::fs::read(root.join(name)).unwrap()))
    .collect();
    let engine = GovernanceEngine::new(store);

    let report = engine.simulate_patch(&proposed.id).unwrap();

    assert_eq!(report.patch_id, proposed.id);
    assert!(!report.mutation_performed);
    assert_eq!(report.snapshot_token.len(), 64);
    assert_eq!(report.predicted_patch_status, PatchStatus::Applied);
    for (name, bytes) in before {
        assert_eq!(
            std::fs::read(root.join(&name)).unwrap(),
            bytes,
            "{name} changed"
        );
    }
}

#[test]
fn recall_effectiveness_and_store_quality_use_recorded_selection_history() {
    let now = Utc.with_ymd_and_hms(2026, 7, 14, 12, 0, 0).unwrap();
    let records = vec![
        record("rec_used", "used workflow"),
        record("rec_never", "never used workflow"),
    ];
    let mut feedback = RecallEvent::new(
        "default",
        RecallEventClient::Cli,
        RecallEventOperation::Feedback,
        "",
        vec!["rec_used".into()],
        0,
        0,
    );
    feedback.outcome = Some(pi_governance_core::RecallEventOutcome::Corrected);
    let events = vec![
        RecallEvent::new(
            "default",
            RecallEventClient::Cli,
            RecallEventOperation::Retrieve,
            "hash",
            vec!["rec_used".into()],
            1200,
            80,
        ),
        feedback,
    ];
    let recall = analyze_recall_effectiveness(&records, &events, "default", now);
    assert_eq!(recall.summary.total_events, 2);
    assert_eq!(recall.summary.corrected_after_recall_count, 1);
    assert_eq!(recall.summary.never_recalled_count, 1);
    let memory = analyze_memory_quality(&records, "default", now);
    let graph = build_memory_graph(&records, &[], &[], "default", 100, 100, now);
    let relationships = analyze_relationship_quality(&graph, &records, now);
    let store = build_store_quality(&memory, &relationships, Some(&recall), 0, 0, now);
    assert!(!store.mutation_performed);
    assert!(store
        .metrics
        .iter()
        .any(|metric| metric.id == "recall_effectiveness"));
    assert!(store.metrics.iter().any(|metric| metric.id == "governance"));
}

#[test]
fn reports_remain_bounded_for_one_thousand_records() {
    let now = Utc.with_ymd_and_hms(2026, 7, 14, 12, 0, 0).unwrap();
    let records: Vec<_> = (0..1000)
        .map(|index| {
            record(
                &format!("rec_{index:04}"),
                &format!("workflow record {index}"),
            )
        })
        .collect();
    let graph = build_memory_graph(&records, &[], &[], "default", 250, 500, now);
    assert!(graph.truncated);
    assert!(graph.nodes.len() <= 250);
    assert!(graph.edges.len() <= 500);
    assert!(!serde_json::to_string(&graph)
        .unwrap()
        .contains("/private/secret.txt"));
    let memory = analyze_memory_quality(&records, "default", now);
    assert_eq!(memory.summary.total_records, 1000);
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
    let mut cycle_a = record("rec_cycle_a", "cycle a");
    cycle_a.supersedes.push("rec_cycle_b".into());
    let mut cycle_b = record("rec_cycle_b", "cycle b");
    cycle_b.supersedes.push("rec_cycle_a".into());
    let records = vec![dangling, orphan, cycle_a, cycle_b];
    let graph = build_memory_graph(&records, &[], &[], "default", 100, 100, now);

    let report = analyze_relationship_quality(&graph, &records, now);

    assert!(!report.mutation_performed);
    assert!(report.summary.dangling_edge_count > 0);
    assert!(report.summary.orphan_memory_count > 0);
    assert!(report.summary.cyclic_memory_pair_count > 0);
    assert!(report
        .relationships
        .iter()
        .any(|item| item.quality_band == pi_governance_engine::RelationshipQualityBand::Broken));
}
