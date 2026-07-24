use chrono::{DateTime, Utc};
use pi_governance_core::{Patch, PatchOperation, Record, StoreEvent};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const GRAPH_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum MemoryGraphNodeType {
    Record,
    Patch,
    EvidenceRef,
    Namespace,
    Event,
}
impl MemoryGraphNodeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Record => "record",
            Self::Patch => "patch",
            Self::EvidenceRef => "evidence_ref",
            Self::Namespace => "namespace",
            Self::Event => "event",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum MemoryGraphEdgeType {
    SupportedBy,
    Supersedes,
    TombstonedBy,
    ContestedBy,
    ReinforcedBy,
    ProposedByPatch,
    BelongsToNamespace,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryGraphNode {
    pub id: String,
    pub node_type: MemoryGraphNodeType,
    pub label: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryGraphEdge {
    pub id: String,
    pub edge_type: MemoryGraphEdgeType,
    pub from: String,
    pub to: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryGraphReport {
    pub schema_version: u32,
    pub generated_at: DateTime<Utc>,
    pub namespace: String,
    pub nodes: Vec<MemoryGraphNode>,
    pub edges: Vec<MemoryGraphEdge>,
    pub warnings: Vec<String>,
    pub truncated: bool,
    pub mutation_performed: bool,
}

fn edge(edge_type: MemoryGraphEdgeType, from: String, to: String) -> MemoryGraphEdge {
    MemoryGraphEdge {
        id: format!("{:?}:{from}->{to}", edge_type).to_lowercase(),
        edge_type,
        from,
        to,
    }
}

pub fn build_memory_graph(
    records: &[Record],
    patches: &[Patch],
    events: &[StoreEvent],
    namespace: &str,
    max_nodes: usize,
    max_edges: usize,
    generated_at: DateTime<Utc>,
) -> MemoryGraphReport {
    let records: Vec<_> = records
        .iter()
        .filter(|record| record.namespace == namespace)
        .collect();
    let mut latest_patches = BTreeMap::new();
    for patch in patches.iter().filter(|patch| patch.namespace == namespace) {
        latest_patches.insert(patch.id.clone(), patch);
    }
    let mut nodes = vec![MemoryGraphNode {
        id: format!("namespace:{namespace}"),
        node_type: MemoryGraphNodeType::Namespace,
        label: namespace.to_string(),
    }];
    let mut edges = Vec::new();
    for record in &records {
        let record_id = format!("record:{}", record.id);
        nodes.push(MemoryGraphNode {
            id: record_id.clone(),
            node_type: MemoryGraphNodeType::Record,
            label: record.claim.chars().take(80).collect(),
        });
        edges.push(edge(
            MemoryGraphEdgeType::BelongsToNamespace,
            record_id.clone(),
            format!("namespace:{namespace}"),
        ));
        for (index, _) in record.evidence.iter().enumerate() {
            let evidence_id = format!("evidence_ref:{}:{index}", record.id);
            nodes.push(MemoryGraphNode {
                id: evidence_id.clone(),
                node_type: MemoryGraphNodeType::EvidenceRef,
                label: format!("evidence {index}"),
            });
            edges.push(edge(
                MemoryGraphEdgeType::SupportedBy,
                record_id.clone(),
                evidence_id,
            ));
        }
        for target in &record.supersedes {
            edges.push(edge(
                MemoryGraphEdgeType::Supersedes,
                record_id.clone(),
                format!("record:{target}"),
            ));
        }
    }
    for patch in latest_patches.values() {
        let patch_id = format!("patch:{}", patch.id);
        nodes.push(MemoryGraphNode {
            id: patch_id.clone(),
            node_type: MemoryGraphNodeType::Patch,
            label: format!("{:?}", patch.operation),
        });
        if let Some(record) = &patch.proposed_record {
            edges.push(edge(
                MemoryGraphEdgeType::ProposedByPatch,
                format!("record:{}", record.id),
                patch_id.clone(),
            ));
        }
        if let Some(target) = &patch.target_id {
            let kind = match patch.operation {
                PatchOperation::TombstoneRecord => Some(MemoryGraphEdgeType::TombstonedBy),
                PatchOperation::ContestRecord | PatchOperation::ResolveContest => {
                    Some(MemoryGraphEdgeType::ContestedBy)
                }
                PatchOperation::ReinforceRecord => Some(MemoryGraphEdgeType::ReinforcedBy),
                _ => None,
            };
            if let Some(kind) = kind {
                edges.push(edge(kind, format!("record:{target}"), patch_id.clone()));
            }
        }
    }
    let patch_ids: BTreeSet<_> = latest_patches.keys().collect();
    for event in events.iter().filter(|event| {
        event.namespace == namespace
            && event
                .object_id
                .as_ref()
                .is_some_and(|id| patch_ids.contains(id))
    }) {
        let event_id = format!("event:{}", event.id);
        nodes.push(MemoryGraphNode {
            id: event_id,
            node_type: MemoryGraphNodeType::Event,
            label: event.severity.clone(),
        });
    }
    nodes.sort_by(|a, b| a.id.cmp(&b.id));
    nodes.dedup_by(|a, b| a.id == b.id);
    edges.sort_by(|a, b| a.id.cmp(&b.id));
    edges.dedup_by(|a, b| a.id == b.id);
    let truncated = nodes.len() > max_nodes || edges.len() > max_edges;
    nodes.truncate(max_nodes);
    edges.truncate(max_edges);
    let warnings = if truncated {
        vec!["graph output truncated by configured limits".to_string()]
    } else {
        Vec::new()
    };
    MemoryGraphReport {
        schema_version: GRAPH_SCHEMA_VERSION,
        generated_at,
        namespace: namespace.to_string(),
        nodes,
        edges,
        warnings,
        truncated,
        mutation_performed: false,
    }
}
