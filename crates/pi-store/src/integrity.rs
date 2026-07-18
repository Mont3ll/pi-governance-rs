use anyhow::{bail, Context, Result};
use pi_governance_core::Record;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;

use crate::backup::{create_store_backup_with_label, StoreBackupReport};
use crate::JsonlStore;

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
    pub fingerprint: String,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordIntegrityApplyReport {
    pub dry_run: bool,
    pub mutation_performed: bool,
    pub migration_needed: bool,
    pub fingerprint: String,
    pub rows_before: usize,
    pub unique_keys_before: usize,
    pub rows_after: usize,
    pub duplicate_groups: usize,
    pub rows_removed: usize,
    pub self_edges_removed: usize,
    pub groups_repaired: usize,
    pub groups: Vec<RecordIntegrityGroup>,
    pub backup: Option<StoreBackupReport>,
    pub report_path: Option<String>,
}

fn records_fingerprint(records: &[Record]) -> String {
    let bytes = serde_json::to_vec(records).expect("Record serialization should be infallible");
    format!("{:x}", Sha256::digest(bytes))
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
        fingerprint: records_fingerprint(records),
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

impl JsonlStore {
    pub fn plan_record_integrity(&self) -> Result<RecordIntegrityPlan> {
        Ok(plan_record_integrity(&self.load_records()?))
    }

    pub fn apply_record_integrity(&self, expected_fingerprint: &str) -> Result<RecordIntegrityApplyReport> {
        let session = self.write_session()?;
        let records = session.load_records()?;
        let plan = plan_record_integrity(&records);
        if plan.fingerprint != expected_fingerprint {
            bail!("integrity preview is stale; run preview again before applying");
        }

        let mut report = RecordIntegrityApplyReport {
            dry_run: false,
            mutation_performed: false,
            migration_needed: plan.migration_needed,
            fingerprint: plan.fingerprint.clone(),
            rows_before: plan.rows_before,
            unique_keys_before: plan.unique_keys_before,
            rows_after: plan.rows_after,
            duplicate_groups: plan.duplicate_groups,
            rows_removed: plan.rows_removed,
            self_edges_removed: plan.self_edges_removed,
            groups_repaired: plan.groups_repaired,
            groups: plan.groups.clone(),
            backup: None,
            report_path: None,
        };
        if !plan.migration_needed {
            return Ok(report);
        }

        report.backup = Some(create_store_backup_with_label(
            self.root(),
            "store-integrity-v1",
            &[("records.jsonl", &self.records_path)],
        )?);
        session.overwrite_records_atomic(&plan.records)?;

        let reports_dir = self.root().join("reports");
        fs::create_dir_all(&reports_dir)
            .with_context(|| format!("failed to create integrity report directory {:?}", reports_dir))?;
        let report_path = reports_dir.join(format!("store-integrity-{}.json", std::process::id()));
        report.mutation_performed = true;
        report.report_path = Some(report_path.display().to_string());
        fs::write(&report_path, format!("{}\n", serde_json::to_string_pretty(&report)?))
            .with_context(|| format!("failed to write integrity report {:?}", report_path))?;
        Ok(report)
    }
}
