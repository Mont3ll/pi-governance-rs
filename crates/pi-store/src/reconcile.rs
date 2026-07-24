use crate::StoreExportBundle;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReconciliationSection {
    pub source_only_ids: Vec<String>,
    pub destination_only_ids: Vec<String>,
    pub matching_ids: Vec<String>,
    pub divergent_ids: Vec<String>,
    pub source_duplicate_ids: Vec<String>,
    pub destination_duplicate_ids: Vec<String>,
    pub conflicting_duplicate_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconciliationArtifactCount {
    pub source: usize,
    pub destination: usize,
    pub delta: isize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconciliationReport {
    pub dry_run: bool,
    pub mutation_performed: bool,
    pub source_identity: BTreeMap<String, Value>,
    pub destination_identity: BTreeMap<String, Value>,
    pub sections: BTreeMap<String, ReconciliationSection>,
    pub artifact_counts: BTreeMap<String, ReconciliationArtifactCount>,
    pub mapping_changes: Vec<Value>,
    pub redaction_omissions: Vec<String>,
    pub warnings: Vec<String>,
}

const SECTION_NAMES: [&str; 8] = [
    "records",
    "patches",
    "evidence",
    "inquiries",
    "sessions",
    "reinforcement",
    "events",
    "tombstones",
];

fn set_array_key(key: Option<&str>) -> bool {
    matches!(
        key,
        Some(
            "tags"
                | "evidence"
                | "evidence_ids"
                | "supersedes"
                | "superseded_by"
                | "related_memory_ids"
                | "related_evidence_ids"
                | "record_ids"
                | "sessions_touched"
                | "fields_checked"
                | "fields_redacted"
        )
    )
}

fn stable_string(value: &Value) -> String {
    serde_json::to_string(value).expect("semantic JSON serialization should be infallible")
}

fn normalize_value(value: Value, key: Option<&str>) -> Value {
    match value {
        Value::Array(values) => {
            let mut normalized: Vec<Value> = values
                .into_iter()
                .map(|value| normalize_value(value, None))
                .collect();
            if set_array_key(key) {
                normalized.sort_by_key(stable_string);
            }
            Value::Array(normalized)
        }
        Value::Object(values) => {
            let mut normalized = Map::new();
            let sorted: BTreeMap<String, Value> = values.into_iter().collect();
            for (child_key, child) in sorted {
                normalized.insert(child_key.clone(), normalize_value(child, Some(&child_key)));
            }
            Value::Object(normalized)
        }
        primitive => primitive,
    }
}

#[derive(Default)]
struct GroupedArtifacts {
    groups: BTreeMap<String, Vec<Value>>,
    duplicate_ids: Vec<String>,
    conflicting_ids: Vec<String>,
}

fn group_artifacts(values: Vec<Value>) -> GroupedArtifacts {
    let mut grouped = GroupedArtifacts::default();
    for value in values {
        let Some(id) = value.get("id").and_then(Value::as_str).map(str::to_string) else {
            continue;
        };
        grouped
            .groups
            .entry(id)
            .or_default()
            .push(normalize_value(value, None));
    }
    for (id, group) in &grouped.groups {
        if group.len() < 2 {
            continue;
        }
        grouped.duplicate_ids.push(id.clone());
        if group
            .iter()
            .map(stable_string)
            .collect::<BTreeSet<_>>()
            .len()
            > 1
        {
            grouped.conflicting_ids.push(id.clone());
        }
    }
    grouped
}

fn compare_section(
    source_values: Vec<Value>,
    destination_values: Vec<Value>,
) -> ReconciliationSection {
    let source = group_artifacts(source_values);
    let destination = group_artifacts(destination_values);
    let source_ids: BTreeSet<String> = source.groups.keys().cloned().collect();
    let destination_ids: BTreeSet<String> = destination.groups.keys().cloned().collect();
    let mut matching_ids = Vec::new();
    let mut divergent_ids = Vec::new();
    for id in source_ids.intersection(&destination_ids) {
        let source_forms: BTreeSet<String> = source.groups[id].iter().map(stable_string).collect();
        let destination_forms: BTreeSet<String> =
            destination.groups[id].iter().map(stable_string).collect();
        if source_forms == destination_forms {
            matching_ids.push(id.clone());
        } else {
            divergent_ids.push(id.clone());
        }
    }
    ReconciliationSection {
        source_only_ids: source_ids.difference(&destination_ids).cloned().collect(),
        destination_only_ids: destination_ids.difference(&source_ids).cloned().collect(),
        matching_ids,
        divergent_ids,
        source_duplicate_ids: source.duplicate_ids,
        destination_duplicate_ids: destination.duplicate_ids,
        conflicting_duplicate_ids: source
            .conflicting_ids
            .into_iter()
            .chain(destination.conflicting_ids)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect(),
    }
}

fn identity(bundle: &StoreExportBundle) -> BTreeMap<String, Value> {
    BTreeMap::from([
        (
            "format".to_string(),
            json!(bundle.format.as_deref().unwrap_or("pi-governance")),
        ),
        (
            "namespace".to_string(),
            json!(bundle.namespace.as_deref().unwrap_or("default")),
        ),
        (
            "producer".to_string(),
            json!(bundle
                .producer
                .as_ref()
                .map(|producer| producer.name.as_str())
                .unwrap_or("unknown")),
        ),
    ])
}

fn section_values(bundle: &StoreExportBundle, section: &str) -> Vec<Value> {
    match section {
        "records" => bundle
            .records
            .iter()
            .map(|value| serde_json::to_value(value).unwrap())
            .collect(),
        "patches" => bundle
            .patches
            .iter()
            .map(|value| serde_json::to_value(value).unwrap())
            .collect(),
        "evidence" => bundle.evidence.clone(),
        "inquiries" => bundle.inquiries.clone(),
        "sessions" => bundle.sessions.clone(),
        "reinforcement" => bundle.reinforcement.clone(),
        "events" => bundle
            .events
            .iter()
            .map(|value| serde_json::to_value(value).unwrap())
            .collect(),
        "tombstones" => bundle.tombstones.clone(),
        _ => Vec::new(),
    }
}

pub fn reconcile_bundles(
    source: &StoreExportBundle,
    destination: &StoreExportBundle,
) -> ReconciliationReport {
    let mut sections = BTreeMap::new();
    let mut artifact_counts = BTreeMap::new();
    for section in SECTION_NAMES {
        let source_values = section_values(source, section);
        let destination_values = section_values(destination, section);
        artifact_counts.insert(
            section.to_string(),
            ReconciliationArtifactCount {
                source: source_values.len(),
                destination: destination_values.len(),
                delta: source_values.len() as isize - destination_values.len() as isize,
            },
        );
        sections.insert(
            section.to_string(),
            compare_section(source_values, destination_values),
        );
    }
    ReconciliationReport {
        dry_run: true,
        mutation_performed: false,
        source_identity: identity(source),
        destination_identity: identity(destination),
        sections,
        artifact_counts,
        mapping_changes: Vec::new(),
        redaction_omissions: Vec::new(),
        warnings: source
            .warnings
            .iter()
            .chain(&destination.warnings)
            .cloned()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect(),
    }
}
