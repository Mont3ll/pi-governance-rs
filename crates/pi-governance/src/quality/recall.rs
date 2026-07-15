use super::QualityRecommendation;
use chrono::{DateTime, Utc};
use pi_governance_core::{RecallEvent, Record, RecordStatus};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const RECALL_EFFECTIVENESS_METRIC_VERSION: u32 = 1;
#[derive(Debug, Clone, Serialize, Deserialize)] pub struct RecallEffectivenessSummary { pub total_events: usize, pub selected_memory_count: usize, pub never_recalled_count: usize, pub average_effectiveness: u32 }
#[derive(Debug, Clone, Serialize, Deserialize)] pub struct RecallMemoryStat { pub memory_id: String, pub selected_count: usize, pub effectiveness_score: u32, pub signals: Vec<String>, pub mutation_performed: bool }
#[derive(Debug, Clone, Serialize, Deserialize)] pub struct RecallEffectivenessReport { pub schema_version: u32, pub metric_version: u32, pub generated_at: DateTime<Utc>, pub namespace: String, pub summary: RecallEffectivenessSummary, pub memory_stats: Vec<RecallMemoryStat>, pub recommendations: Vec<QualityRecommendation>, pub mutation_performed: bool }

pub fn analyze_recall_effectiveness(records: &[Record], events: &[RecallEvent], namespace: &str, generated_at: DateTime<Utc>) -> RecallEffectivenessReport {
    let events: Vec<_> = events.iter().filter(|event| event.namespace == namespace).collect();
    let mut counts = HashMap::new();
    for event in &events { for id in &event.selected_record_ids { *counts.entry(id.clone()).or_insert(0usize) += 1; } }
    let mut memory_stats: Vec<_> = records.iter().filter(|record| record.namespace == namespace).map(|record| {
        let selected_count = counts.get(&record.id).copied().unwrap_or(0);
        let never = selected_count == 0 && record.status == RecordStatus::Active;
        RecallMemoryStat { memory_id: record.id.clone(), selected_count, effectiveness_score: if never { 50 } else { (70 + selected_count.min(6) as u32 * 5).min(100) }, signals: if never { vec!["never_recalled".into()] } else if selected_count >= 3 { vec!["frequently_recalled".into()] } else { Vec::new() }, mutation_performed: false }
    }).collect();
    memory_stats.sort_by(|a, b| a.effectiveness_score.cmp(&b.effectiveness_score).then(a.memory_id.cmp(&b.memory_id)));
    let average = if memory_stats.is_empty() { 100 } else { memory_stats.iter().map(|stat| stat.effectiveness_score).sum::<u32>() / memory_stats.len() as u32 };
    let recommendations = memory_stats.iter().filter(|stat| stat.signals.contains(&"never_recalled".to_string())).map(|stat| QualityRecommendation { id: format!("recall:{}", stat.memory_id), summary: format!("Review recall for {}", stat.memory_id), reason: "active record has never been selected in recorded telemetry".into(), review_required: true, mutation_performed: false }).collect();
    RecallEffectivenessReport { schema_version: 1, metric_version: RECALL_EFFECTIVENESS_METRIC_VERSION, generated_at, namespace: namespace.into(), summary: RecallEffectivenessSummary { total_events: events.len(), selected_memory_count: counts.len(), never_recalled_count: memory_stats.iter().filter(|stat| stat.signals.contains(&"never_recalled".to_string())).count(), average_effectiveness: average }, memory_stats, recommendations, mutation_performed: false }
}
