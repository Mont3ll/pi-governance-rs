use super::QualityRecommendation;
use chrono::{DateTime, Utc};
use pi_governance_core::{RecallEvent, RecallEventOutcome, Record, RecordStatus};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const RECALL_EFFECTIVENESS_METRIC_VERSION: u32 = 2;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecallEffectivenessSummary {
    pub total_events: usize,
    pub selected_memory_count: usize,
    pub never_recalled_count: usize,
    pub corrected_after_recall_count: usize,
    pub exclusion_count: usize,
    pub average_effectiveness: u32,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecallMemoryStat {
    pub memory_id: String,
    pub selected_count: usize,
    pub correction_count: usize,
    pub ignored_count: usize,
    pub effectiveness_score: u32,
    pub signals: Vec<String>,
    pub mutation_performed: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecallEffectivenessReport {
    pub schema_version: u32,
    pub metric_version: u32,
    pub generated_at: DateTime<Utc>,
    pub namespace: String,
    pub summary: RecallEffectivenessSummary,
    pub memory_stats: Vec<RecallMemoryStat>,
    pub exclusion_reasons: std::collections::BTreeMap<String, usize>,
    pub recommendations: Vec<QualityRecommendation>,
    pub mutation_performed: bool,
}

pub fn analyze_recall_effectiveness(
    records: &[Record],
    events: &[RecallEvent],
    namespace: &str,
    generated_at: DateTime<Utc>,
) -> RecallEffectivenessReport {
    let events: Vec<_> = events
        .iter()
        .filter(|event| event.namespace == namespace)
        .collect();
    let mut selected = HashMap::new();
    let mut corrected = HashMap::new();
    let mut ignored = HashMap::new();
    let mut exclusion_reasons = std::collections::BTreeMap::new();
    for event in &events {
        for (reason, count) in &event.excluded_reason_counts {
            *exclusion_reasons.entry(reason.clone()).or_insert(0) += count;
        }
        for id in &event.selected_record_ids {
            *selected.entry(id.clone()).or_insert(0usize) += 1;
            if event.outcome == Some(RecallEventOutcome::Corrected) {
                *corrected.entry(id.clone()).or_insert(0usize) += 1;
            }
            if event.outcome == Some(RecallEventOutcome::Ignored) {
                *ignored.entry(id.clone()).or_insert(0usize) += 1;
            }
        }
    }
    let mut memory_stats: Vec<_> = records
        .iter()
        .filter(|record| record.namespace == namespace)
        .map(|record| {
            let selected_count = selected.get(&record.id).copied().unwrap_or(0);
            let correction_count = corrected.get(&record.id).copied().unwrap_or(0);
            let ignored_count = ignored.get(&record.id).copied().unwrap_or(0);
            let never = selected_count == 0 && record.status == RecordStatus::Active;
            let mut signals = Vec::new();
            if never {
                signals.push("never_recalled".into());
            }
            if correction_count > 0 {
                signals.push("recalled_then_corrected".into());
            }
            if ignored_count > 0 {
                signals.push("recalled_then_ignored".into());
            }
            if selected_count >= 3 && correction_count == 0 {
                signals.push("frequently_recalled".into());
            }
            let score = (if never {
                50
            } else {
                70 + selected_count.min(6) as i32 * 5
            } - correction_count.min(2) as i32 * 20
                - ignored_count.min(2) as i32 * 10)
                .clamp(0, 100) as u32;
            RecallMemoryStat {
                memory_id: record.id.clone(),
                selected_count,
                correction_count,
                ignored_count,
                effectiveness_score: score,
                signals,
                mutation_performed: false,
            }
        })
        .collect();
    memory_stats.sort_by(|a, b| {
        a.effectiveness_score
            .cmp(&b.effectiveness_score)
            .then(a.memory_id.cmp(&b.memory_id))
    });
    let average = if memory_stats.is_empty() {
        100
    } else {
        memory_stats
            .iter()
            .map(|stat| stat.effectiveness_score)
            .sum::<u32>()
            / memory_stats.len() as u32
    };
    let recommendations = memory_stats
        .iter()
        .filter(|stat| stat.effectiveness_score < 70)
        .map(|stat| QualityRecommendation {
            id: format!("recall:{}", stat.memory_id),
            summary: format!("Review recall for {}", stat.memory_id),
            reason: stat.signals.join(", "),
            review_required: true,
            mutation_performed: false,
        })
        .collect();
    let exclusion_count = exclusion_reasons.values().sum();
    RecallEffectivenessReport {
        schema_version: 1,
        metric_version: RECALL_EFFECTIVENESS_METRIC_VERSION,
        generated_at,
        namespace: namespace.into(),
        summary: RecallEffectivenessSummary {
            total_events: events.len(),
            selected_memory_count: selected.len(),
            never_recalled_count: memory_stats
                .iter()
                .filter(|stat| stat.signals.iter().any(|s| s == "never_recalled"))
                .count(),
            corrected_after_recall_count: memory_stats
                .iter()
                .filter(|stat| stat.correction_count > 0)
                .count(),
            exclusion_count,
            average_effectiveness: average,
        },
        memory_stats,
        exclusion_reasons,
        recommendations,
        mutation_performed: false,
    }
}
