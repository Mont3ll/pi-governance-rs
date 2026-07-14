use super::QualityRecommendation;
use crate::graph::{MemoryGraphEdgeType, MemoryGraphReport};
use chrono::{DateTime, Utc};
use pi_governance_core::Record;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub const RELATIONSHIP_QUALITY_METRIC_VERSION: u32 = 1;
#[derive(Debug, Clone, Serialize, Deserialize)] pub struct RelationshipQualitySummary { pub total_edges: usize, pub dangling_edge_count: usize, pub orphan_memory_count: usize, pub average_relationship_quality: u32 }
#[derive(Debug, Clone, Serialize, Deserialize)] pub struct RelationshipQualityItem { pub edge_id: String, pub quality_score: u32, pub signals: Vec<String>, pub mutation_performed: bool }
#[derive(Debug, Clone, Serialize, Deserialize)] pub struct RelationshipQualityReport { pub schema_version: u32, pub metric_version: u32, pub generated_at: DateTime<Utc>, pub namespace: String, pub summary: RelationshipQualitySummary, pub relationships: Vec<RelationshipQualityItem>, pub recommendations: Vec<QualityRecommendation>, pub mutation_performed: bool }

pub fn analyze_relationship_quality(graph: &MemoryGraphReport, records: &[Record], generated_at: DateTime<Utc>) -> RelationshipQualityReport {
    let node_ids: HashSet<_> = graph.nodes.iter().map(|node| node.id.as_str()).collect();
    let mut relationships = Vec::new();
    for edge in &graph.edges {
        let dangling = !node_ids.contains(edge.from.as_str()) || !node_ids.contains(edge.to.as_str());
        let lifecycle = matches!(edge.edge_type, MemoryGraphEdgeType::Supersedes | MemoryGraphEdgeType::TombstonedBy | MemoryGraphEdgeType::ContestedBy | MemoryGraphEdgeType::ReinforcedBy);
        relationships.push(RelationshipQualityItem { edge_id: edge.id.clone(), quality_score: if dangling { 0 } else if lifecycle { 85 } else { 75 }, signals: if dangling { vec!["dangling_endpoint".into()] } else { Vec::new() }, mutation_performed: false });
    }
    relationships.sort_by(|a, b| a.quality_score.cmp(&b.quality_score).then(a.edge_id.cmp(&b.edge_id)));
    let connected: HashSet<_> = graph.edges.iter().filter(|edge| edge.edge_type != MemoryGraphEdgeType::BelongsToNamespace).flat_map(|edge| [edge.from.as_str(), edge.to.as_str()]).collect();
    let orphan_memory_count = records.iter().filter(|record| record.namespace == graph.namespace && record.evidence.is_empty() && !connected.contains(format!("record:{}", record.id).as_str())).count();
    let dangling_edge_count = relationships.iter().filter(|item| item.signals.contains(&"dangling_endpoint".to_string())).count();
    let average = if relationships.is_empty() { 100 } else { relationships.iter().map(|item| item.quality_score).sum::<u32>() / relationships.len() as u32 };
    let recommendations = if dangling_edge_count + orphan_memory_count > 0 { vec![QualityRecommendation { id: "relationship_quality:review".into(), summary: "Review weak memory relationships".into(), reason: format!("{dangling_edge_count} dangling edge(s), {orphan_memory_count} orphan record(s)"), review_required: true, mutation_performed: false }] } else { Vec::new() };
    RelationshipQualityReport { schema_version: 1, metric_version: RELATIONSHIP_QUALITY_METRIC_VERSION, generated_at, namespace: graph.namespace.clone(), summary: RelationshipQualitySummary { total_edges: relationships.len(), dangling_edge_count, orphan_memory_count, average_relationship_quality: average }, relationships, recommendations, mutation_performed: false }
}
