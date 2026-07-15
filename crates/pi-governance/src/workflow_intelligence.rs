use chrono::{DateTime, Duration, Utc};
use pi_governance_core::{MemoryLayer, Patch, PatchStatus, Record, RecordClass, RecordStatus, RuleType, StoreEvent};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)] #[serde(rename_all = "snake_case")] pub enum ProcedureExportStatus { NotExported, ReviewRequired, Approved, Exported }
#[derive(Debug, Clone, Serialize, Deserialize)] pub struct ProcedureCandidate { pub id: String, pub title: String, pub when_to_use: String, pub steps: Vec<String>, pub pitfalls: Vec<String>, pub verification_steps: Vec<String>, pub source_record_ids: Vec<String>, pub evidence_reference_count: usize, pub confidence: f32, pub suggested_skill_name: String, pub export_status: ProcedureExportStatus, pub review_required: bool, pub mutation_performed: bool }
#[derive(Debug, Clone, Serialize, Deserialize)] pub struct ProcedureCandidateReport { pub schema_version: u32, pub generated_at: DateTime<Utc>, pub namespace: String, pub candidates: Vec<ProcedureCandidate>, pub skipped_record_count: usize, pub mutation_performed: bool }

pub fn generate_procedure_candidates(records: &[Record], namespace: &str, min_source_records: usize, generated_at: DateTime<Utc>) -> ProcedureCandidateReport {
    let mut groups: BTreeMap<String, Vec<&Record>> = BTreeMap::new(); let mut skipped = 0;
    for record in records.iter().filter(|record| record.namespace == namespace) {
        let eligible = record.status == RecordStatus::Active && record.layer == MemoryLayer::L2Playbook && (record.class == RecordClass::Workflow || record.rule_type == Some(RuleType::Workflow) || record.rule_type == Some(RuleType::Testing) || record.rule_type == Some(RuleType::Tool));
        if !eligible { skipped += 1; continue; }
        let key = record.tags.first().cloned().unwrap_or_else(|| "workflow".into()); groups.entry(key).or_default().push(record);
    }
    let mut candidates = Vec::new();
    for (group, mut source) in groups { if source.len() < min_source_records.max(2) { skipped += source.len(); continue; } source.sort_by(|a, b| a.id.cmp(&b.id)); let ids: Vec<_> = source.iter().map(|record| record.id.clone()).collect(); let steps: Vec<_> = source.iter().map(|record| record.claim.clone()).collect(); let verification_steps = steps.iter().filter(|step| step.to_lowercase().contains("test") || step.to_lowercase().contains("verify") || step.to_lowercase().contains("audit")).cloned().collect(); let confidence = source.iter().map(|record| record.confidence).sum::<f32>() / source.len() as f32; let evidence_reference_count = source.iter().map(|record| record.evidence.len()).sum(); candidates.push(ProcedureCandidate { id: format!("procedure:{}:{}", namespace, group), title: format!("{} workflow", group), when_to_use: format!("Use when the governed {group} workflow applies."), steps, pitfalls: vec!["Review ordering and applicability before execution.".into(), "Do not treat this report as canonical memory or an installed skill.".into()], verification_steps, source_record_ids: ids, evidence_reference_count, confidence, suggested_skill_name: format!("{}-workflow", group.to_lowercase().replace(|c: char| !c.is_ascii_alphanumeric(), "-")), export_status: ProcedureExportStatus::ReviewRequired, review_required: true, mutation_performed: false }); }
    ProcedureCandidateReport { schema_version: 1, generated_at, namespace: namespace.into(), candidates, skipped_record_count: skipped, mutation_performed: false }
}

#[derive(Debug, Clone, Serialize, Deserialize)] pub struct FailureAnalysisSummary { pub rejected_patch_count: usize, pub stale_deferred_patch_count: usize, pub warning_event_count: usize, pub event_category_count: usize }
#[derive(Debug, Clone, Serialize, Deserialize)] pub struct FailurePattern { pub id: String, pub category: String, pub affected_object_ids: Vec<String>, pub recommendation: String, pub review_required: bool }
#[derive(Debug, Clone, Serialize, Deserialize)] pub struct FailureAnalysisReport { pub schema_version: u32, pub generated_at: DateTime<Utc>, pub namespace: String, pub summary: FailureAnalysisSummary, pub patterns: Vec<FailurePattern>, pub mutation_performed: bool }

pub fn analyze_failure_patterns(patches: &[Patch], events: &[StoreEvent], namespace: &str, stale_deferred_days: i64, generated_at: DateTime<Utc>) -> FailureAnalysisReport {
    let mut latest = BTreeMap::new(); for patch in patches.iter().filter(|patch| patch.namespace == namespace) { latest.insert(patch.id.clone(), patch); }
    let rejected: Vec<_> = latest.values().filter(|patch| patch.status == PatchStatus::Rejected).map(|patch| patch.id.clone()).collect();
    let stale_before = generated_at - Duration::days(stale_deferred_days.max(1));
    let deferred: Vec<_> = latest.values().filter(|patch| patch.status == PatchStatus::Deferred && patch.updated_at < stale_before).map(|patch| patch.id.clone()).collect();
    let mut warning_categories: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for event in events.iter().filter(|event| event.namespace == namespace && event.severity != "info") { warning_categories.entry(event.category.clone()).or_default().extend(event.object_id.clone()); }
    let mut patterns = Vec::new();
    if !rejected.is_empty() { patterns.push(FailurePattern { id: "failure:rejected_patches".into(), category: "rejected_patches".into(), affected_object_ids: rejected.clone(), recommendation: "Review repeated rejection reasons before proposing similar memory changes.".into(), review_required: true }); }
    if !deferred.is_empty() { patterns.push(FailurePattern { id: "failure:stale_deferred".into(), category: "stale_deferred_patches".into(), affected_object_ids: deferred.clone(), recommendation: "Apply, reject, or explicitly re-defer stale patches after review.".into(), review_required: true }); }
    for (category, ids) in &warning_categories { patterns.push(FailurePattern { id: format!("failure:event:{category}"), category: category.clone(), affected_object_ids: ids.clone(), recommendation: format!("Inspect recurring {category} events and their affected objects."), review_required: true }); }
    FailureAnalysisReport { schema_version: 1, generated_at, namespace: namespace.into(), summary: FailureAnalysisSummary { rejected_patch_count: rejected.len(), stale_deferred_patch_count: deferred.len(), warning_event_count: events.iter().filter(|event| event.namespace == namespace && event.severity != "info").count(), event_category_count: warning_categories.len() }, patterns, mutation_performed: false }
}
