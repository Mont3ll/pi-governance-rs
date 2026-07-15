use super::{MemoryQualityReport, RecallEffectivenessReport, RelationshipQualityReport};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)] pub struct StoreQualityMetric { pub id: String, pub score: Option<u32>, pub signals: Vec<String> }
#[derive(Debug, Clone, Serialize, Deserialize)] pub struct StoreQualityReport { pub schema_version: u32, pub metric_version: u32, pub generated_at: DateTime<Utc>, pub overall_score: u32, pub metrics: Vec<StoreQualityMetric>, pub mutation_performed: bool }
pub fn build_store_quality(memory: &MemoryQualityReport, relationships: &RelationshipQualityReport, recall: Option<&RecallEffectivenessReport>, pending_patches: usize, runtime_warnings: usize, generated_at: DateTime<Utc>) -> StoreQualityReport {
    let mut metrics = vec![
        StoreQualityMetric { id: "memory_quality".into(), score: Some(memory.summary.average_quality), signals: Vec::new() },
        StoreQualityMetric { id: "relationship_quality".into(), score: Some(relationships.summary.average_relationship_quality), signals: Vec::new() },
        StoreQualityMetric { id: "recall_effectiveness".into(), score: recall.map(|report| report.summary.average_effectiveness), signals: if recall.is_none() { vec!["unavailable".into()] } else { Vec::new() } },
        StoreQualityMetric { id: "inbox".into(), score: Some(if pending_patches == 0 { 100 } else if pending_patches <= 5 { 85 } else { 60 }), signals: if pending_patches > 0 { vec!["pending_patches".into()] } else { Vec::new() } },
        StoreQualityMetric { id: "runtime".into(), score: Some(if runtime_warnings == 0 { 100 } else if runtime_warnings <= 2 { 85 } else { 60 }), signals: if runtime_warnings > 0 { vec!["runtime_warnings".into()] } else { Vec::new() } },
    ];
    let scores: Vec<_> = metrics.iter().filter_map(|metric| metric.score).collect();
    let overall_score = if scores.is_empty() { 100 } else { scores.iter().sum::<u32>() / scores.len() as u32 };
    StoreQualityReport { schema_version: 1, metric_version: 1, generated_at, overall_score, metrics: std::mem::take(&mut metrics), mutation_performed: false }
}
