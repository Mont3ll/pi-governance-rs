use super::QualityRecommendation;
use crate::graph::{MemoryGraphEdgeType, MemoryGraphReport};
use chrono::{DateTime, Utc};
use pi_governance_core::Record;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap, HashSet};

pub const RELATIONSHIP_QUALITY_METRIC_VERSION: u32 = 2;
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RelationshipQualityBand {
    Strong,
    Medium,
    Weak,
    Broken,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipQualitySummary {
    pub total_edges: usize,
    pub weak_edge_count: usize,
    pub dangling_edge_count: usize,
    pub orphan_memory_count: usize,
    pub dead_end_memory_count: usize,
    pub high_value_hub_count: usize,
    pub cyclic_memory_pair_count: usize,
    pub average_relationship_quality: u32,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipQualityItem {
    pub edge_id: String,
    pub quality_score: u32,
    pub quality_band: RelationshipQualityBand,
    pub signals: Vec<String>,
    pub mutation_performed: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipQualityReport {
    pub schema_version: u32,
    pub metric_version: u32,
    pub generated_at: DateTime<Utc>,
    pub namespace: String,
    pub summary: RelationshipQualitySummary,
    pub relationships: Vec<RelationshipQualityItem>,
    pub recommendations: Vec<QualityRecommendation>,
    pub mutation_performed: bool,
}

fn band(score: u32) -> RelationshipQualityBand {
    if score == 0 {
        RelationshipQualityBand::Broken
    } else if score < 50 {
        RelationshipQualityBand::Weak
    } else if score < 80 {
        RelationshipQualityBand::Medium
    } else {
        RelationshipQualityBand::Strong
    }
}
pub fn analyze_relationship_quality(
    graph: &MemoryGraphReport,
    records: &[Record],
    generated_at: DateTime<Utc>,
) -> RelationshipQualityReport {
    let node_ids: HashSet<_> = graph.nodes.iter().map(|node| node.id.as_str()).collect();
    let mut relationships = Vec::new();
    let mut touching: HashMap<String, Vec<&crate::MemoryGraphEdge>> = HashMap::new();
    for edge in &graph.edges {
        let dangling =
            !node_ids.contains(edge.from.as_str()) || !node_ids.contains(edge.to.as_str());
        let lifecycle = matches!(
            edge.edge_type,
            MemoryGraphEdgeType::Supersedes
                | MemoryGraphEdgeType::TombstonedBy
                | MemoryGraphEdgeType::ContestedBy
                | MemoryGraphEdgeType::ReinforcedBy
        );
        let score = if dangling {
            0
        } else if lifecycle || edge.edge_type == MemoryGraphEdgeType::SupportedBy {
            85
        } else {
            70
        };
        relationships.push(RelationshipQualityItem {
            edge_id: edge.id.clone(),
            quality_score: score,
            quality_band: band(score),
            signals: if dangling {
                vec!["dangling_endpoint".into()]
            } else {
                Vec::new()
            },
            mutation_performed: false,
        });
        if edge.edge_type != MemoryGraphEdgeType::BelongsToNamespace {
            for id in [&edge.from, &edge.to] {
                if id.starts_with("record:") {
                    touching.entry(id.clone()).or_default().push(edge);
                }
            }
        }
    }
    relationships.sort_by(|a, b| {
        a.quality_score
            .cmp(&b.quality_score)
            .then(a.edge_id.cmp(&b.edge_id))
    });
    let mut orphan = 0;
    let mut dead_end = 0;
    let mut hubs = 0;
    for record in records
        .iter()
        .filter(|record| record.namespace == graph.namespace)
    {
        let id = format!("record:{}", record.id);
        let edges = touching.get(&id).cloned().unwrap_or_default();
        if edges.is_empty() && record.evidence.is_empty() {
            orphan += 1;
            continue;
        }
        let useful = edges.iter().any(|edge| {
            matches!(
                edge.edge_type,
                MemoryGraphEdgeType::SupportedBy
                    | MemoryGraphEdgeType::Supersedes
                    | MemoryGraphEdgeType::TombstonedBy
                    | MemoryGraphEdgeType::ReinforcedBy
            )
        });
        if !edges.is_empty() && !useful {
            dead_end += 1;
        }
        let types: BTreeSet<_> = edges.iter().map(|edge| &edge.edge_type).collect();
        if edges.len() >= 4 && types.len() >= 2 {
            hubs += 1;
        }
    }
    let directed: HashSet<_> = graph
        .edges
        .iter()
        .filter(|edge| edge.from.starts_with("record:") && edge.to.starts_with("record:"))
        .map(|edge| (edge.from.as_str(), edge.to.as_str()))
        .collect();
    let mut cycles = HashSet::new();
    for (from, to) in &directed {
        if directed.contains(&(*to, *from)) {
            let mut pair = [*from, *to];
            pair.sort();
            cycles.insert(pair);
        }
    }
    let dangling = relationships
        .iter()
        .filter(|item| {
            item.signals
                .iter()
                .any(|signal| signal == "dangling_endpoint")
        })
        .count();
    let weak = relationships
        .iter()
        .filter(|item| {
            matches!(
                item.quality_band,
                RelationshipQualityBand::Weak | RelationshipQualityBand::Broken
            )
        })
        .count();
    let average = if relationships.is_empty() {
        100
    } else {
        relationships
            .iter()
            .map(|item| item.quality_score)
            .sum::<u32>()
            / relationships.len() as u32
    };
    let recommendations = if weak + orphan + dead_end + cycles.len() > 0 {
        vec![QualityRecommendation { id: "relationship_quality:review".into(), summary: "Review weak memory relationships".into(), reason: format!("{weak} weak edge(s), {orphan} orphan(s), {dead_end} dead end(s), {} cyclic pair(s)", cycles.len()), review_required: true, mutation_performed: false }]
    } else {
        Vec::new()
    };
    RelationshipQualityReport {
        schema_version: 1,
        metric_version: RELATIONSHIP_QUALITY_METRIC_VERSION,
        generated_at,
        namespace: graph.namespace.clone(),
        summary: RelationshipQualitySummary {
            total_edges: relationships.len(),
            weak_edge_count: weak,
            dangling_edge_count: dangling,
            orphan_memory_count: orphan,
            dead_end_memory_count: dead_end,
            high_value_hub_count: hubs,
            cyclic_memory_pair_count: cycles.len(),
            average_relationship_quality: average,
        },
        relationships,
        recommendations,
        mutation_performed: false,
    }
}
