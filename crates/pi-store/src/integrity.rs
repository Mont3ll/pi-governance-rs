use pi_governance_core::Record;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RecordIntegrityGroup {
    pub namespace: String,
    pub id: String,
    pub source_ordinals: Vec<usize>,
    pub canonical_ordinal: usize,
    pub removed_rows: usize,
    pub retained_supersedes: Vec<String>,
    pub removed_self_edges: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordIntegrityPlan {
    pub dry_run: bool,
    pub mutation_performed: bool,
    pub migration_needed: bool,
    pub rows_before: usize,
    pub unique_keys_before: usize,
    pub rows_after: usize,
    pub duplicate_groups: usize,
    pub rows_removed: usize,
    pub self_edges_removed: usize,
    pub groups_repaired: usize,
    pub groups: Vec<RecordIntegrityGroup>,
    #[serde(skip)]
    pub records: Vec<Record>,
}

fn retained_supersedes(group: &[(usize, &Record)], id: &str) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut retained = Vec::new();
    for (_, record) in group {
        for value in &record.supersedes {
            if value == id || !seen.insert(value.clone()) {
                continue;
            }
            retained.push(value.clone());
        }
    }
    retained
}

pub fn plan_record_integrity(records: &[Record]) -> RecordIntegrityPlan {
    let mut grouped: BTreeMap<(String, String), Vec<(usize, &Record)>> = BTreeMap::new();
    for (index, record) in records.iter().enumerate() {
        grouped
            .entry((record.namespace.clone(), record.id.clone()))
            .or_default()
            .push((index, record));
    }

    let mut replacements: HashMap<usize, Option<Record>> = HashMap::new();
    let mut groups = Vec::new();
    let mut duplicate_groups = 0;
    let mut rows_removed = 0;
    let mut self_edges_removed = 0;

    for ((namespace, id), group) in &grouped {
        let self_edges = group
            .iter()
            .map(|(_, record)| {
                record
                    .supersedes
                    .iter()
                    .filter(|value| *value == id)
                    .count()
            })
            .sum::<usize>();
        if group.len() == 1 && self_edges == 0 {
            continue;
        }
        if group.len() > 1 {
            duplicate_groups += 1;
        }

        let (canonical_index, canonical_record) =
            group.last().expect("integrity group is non-empty");
        let retained = retained_supersedes(group, id);
        let mut repaired = (*canonical_record).clone();
        repaired.supersedes = retained.clone();
        for (index, _) in group {
            replacements.insert(*index, None);
        }
        replacements.insert(*canonical_index, Some(repaired));

        let removed = group.len() - 1;
        rows_removed += removed;
        self_edges_removed += self_edges;
        groups.push(RecordIntegrityGroup {
            namespace: namespace.clone(),
            id: id.clone(),
            source_ordinals: group.iter().map(|(index, _)| index + 1).collect(),
            canonical_ordinal: canonical_index + 1,
            removed_rows: removed,
            retained_supersedes: retained,
            removed_self_edges: self_edges,
        });
    }

    let repaired_records = records
        .iter()
        .enumerate()
        .filter_map(|(index, record)| match replacements.get(&index) {
            Some(Some(replacement)) => Some(replacement.clone()),
            Some(None) => None,
            None => Some(record.clone()),
        })
        .collect::<Vec<_>>();

    RecordIntegrityPlan {
        dry_run: true,
        mutation_performed: false,
        migration_needed: !groups.is_empty(),
        rows_before: records.len(),
        unique_keys_before: grouped.len(),
        rows_after: repaired_records.len(),
        duplicate_groups,
        rows_removed,
        self_edges_removed,
        groups_repaired: groups.len(),
        groups,
        records: repaired_records,
    }
}
