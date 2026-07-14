use super::QualityRecommendation;
use chrono::{DateTime, Utc};
use pi_governance_core::{Durability, Record, RecordStatus, SourceKind, TrustClass};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

pub const MEMORY_QUALITY_METRIC_VERSION: u32 = 1;
#[derive(Debug, Clone, Serialize, Deserialize)] pub struct MemoryQualitySummary { pub total_records: usize, pub low_quality_count: usize, pub stale_count: usize, pub duplicate_signal_count: usize, pub average_quality: u32 }
#[derive(Debug, Clone, Serialize, Deserialize)] pub struct MemoryQualityItem { pub memory_id: String, pub quality_score: u32, pub signals: Vec<String>, pub reasons: Vec<String>, pub mutation_performed: bool }
#[derive(Debug, Clone, Serialize, Deserialize)] pub struct MemoryQualityReport { pub schema_version: u32, pub metric_version: u32, pub generated_at: DateTime<Utc>, pub namespace: String, pub summary: MemoryQualitySummary, pub items: Vec<MemoryQualityItem>, pub recommendations: Vec<QualityRecommendation>, pub mutation_performed: bool }

pub fn analyze_memory_quality(records: &[Record], namespace: &str, now: DateTime<Utc>) -> MemoryQualityReport {
    let records: Vec<_> = records.iter().filter(|record| record.namespace == namespace).collect();
    let mut claims: HashMap<String, Vec<String>> = HashMap::new();
    for record in records.iter().filter(|record| record.status == RecordStatus::Active) { claims.entry(record.claim.to_lowercase().split_whitespace().collect::<Vec<_>>().join(" ")).or_default().push(record.id.clone()); }
    let duplicates: HashSet<_> = claims.values().filter(|ids| ids.len() > 1).flatten().cloned().collect();
    let mut items = Vec::new();
    for record in &records {
        let mut score = 100i32; let mut signals = Vec::new(); let mut reasons = Vec::new();
        macro_rules! deduct { ($amount:expr, $signal:expr, $reason:expr) => {{ score -= $amount; signals.push($signal.to_string()); reasons.push($reason.to_string()); }}; }
        if record.confidence < 0.65 { deduct!(25, "low_confidence", "confidence is below 0.65"); } else if record.confidence < 0.8 { deduct!(10, "medium_confidence", "confidence is below 0.80"); }
        if record.evidence.is_empty() { deduct!(25, "missing_evidence", "record has no evidence references"); }
        let age = now.signed_duration_since(record.updated_at).num_days().max(0);
        if age > 180 { deduct!(20, "stale", "record was not updated for more than 180 days"); } else if age > 90 { deduct!(8, "dormant", "record was not updated for more than 90 days"); }
        if duplicates.contains(&record.id) { deduct!(15, "duplicate_claim", "active record duplicates a normalized claim"); }
        match record.status { RecordStatus::Contested => deduct!(25, "contested", "record is contested"), RecordStatus::Superseded => deduct!(30, "superseded", "record is superseded"), RecordStatus::Tombstoned => { score = 0; signals.push("tombstoned".into()); reasons.push("record is tombstoned".into()); }, RecordStatus::Active => {} }
        if record.trust_class == TrustClass::Unknown { deduct!(5, "unknown_trust", "trust class is unknown"); }
        if record.durability == Durability::Unknown { deduct!(5, "unknown_durability", "durability is unknown"); }
        if record.source_kind == SourceKind::Unknown { deduct!(5, "unknown_source", "source kind is unknown"); }
        items.push(MemoryQualityItem { memory_id: record.id.clone(), quality_score: score.clamp(0, 100) as u32, signals, reasons, mutation_performed: false });
    }
    items.sort_by(|a, b| a.quality_score.cmp(&b.quality_score).then(a.memory_id.cmp(&b.memory_id)));
    let average = if items.is_empty() { 100 } else { items.iter().map(|item| item.quality_score).sum::<u32>() / items.len() as u32 };
    let recommendations = items.iter().filter(|item| item.quality_score < 70).map(|item| QualityRecommendation { id: format!("memory_quality:{}", item.memory_id), summary: format!("Review {}", item.memory_id), reason: item.reasons.join("; "), review_required: true, mutation_performed: false }).collect();
    let summary = MemoryQualitySummary { total_records: items.len(), low_quality_count: items.iter().filter(|item| item.quality_score < 70).count(), stale_count: items.iter().filter(|item| item.signals.contains(&"stale".to_string())).count(), duplicate_signal_count: items.iter().filter(|item| item.signals.contains(&"duplicate_claim".to_string())).count(), average_quality: average };
    MemoryQualityReport { schema_version: 1, metric_version: MEMORY_QUALITY_METRIC_VERSION, generated_at: now, namespace: namespace.into(), summary, items, recommendations, mutation_performed: false }
}
