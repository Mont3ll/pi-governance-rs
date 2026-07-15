use crate::{analyze_memory_quality, analyze_recall_effectiveness, analyze_relationship_quality, build_memory_graph, build_store_quality, GovernanceEngine};
use anyhow::{bail, Context, Result};
use chrono::{DateTime, Utc};
use pi_governance_core::{validate_patch, ContestResolution, Patch, PatchOperation, PatchStatus, Record, RecordStatus};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize)] pub struct SimulationScorecard { pub memory_quality: u32, pub relationship_quality: u32, pub store_quality: u32, pub record_count: usize, pub active_record_count: usize }
#[derive(Debug, Clone, Serialize, Deserialize)] pub struct PatchSimulationReport { pub schema_version: u32, pub generated_at: DateTime<Utc>, pub patch_id: String, pub snapshot_token: String, pub predicted_patch_status: PatchStatus, pub affected_record_ids: Vec<String>, pub before: SimulationScorecard, pub after: SimulationScorecard, pub memory_quality_delta: i32, pub relationship_quality_delta: i32, pub store_quality_delta: i32, pub review_required: bool, pub mutation_performed: bool }

pub fn transition_records(records: &[Record], patch: &Patch, now: DateTime<Utc>) -> Result<(Vec<Record>, Vec<String>)> {
    let mut output = records.to_vec(); let mut affected = Vec::new();
    match patch.operation {
        PatchOperation::ProposeRecord => { let record = patch.proposed_record.clone().context("propose_record patch missing proposed_record")?; if output.iter().any(|existing| existing.id == record.id) { bail!("record_conflict: record {} already exists", record.id); } affected.push(record.id.clone()); output.push(record); }
        PatchOperation::SupersedeRecord => { let target = patch.target_id.as_ref().context("supersede patch missing target_id")?; for record in &mut output { if &record.id == target && record.namespace == patch.namespace { record.status = RecordStatus::Superseded; record.updated_at = now; } } let mut replacement = patch.proposed_record.clone().context("supersede patch missing replacement record")?; replacement.supersedes.push(target.clone()); affected.extend([target.clone(), replacement.id.clone()]); output.push(replacement); }
        PatchOperation::TombstoneRecord => update_status(&mut output, patch, RecordStatus::Tombstoned, now, &mut affected)?,
        PatchOperation::ContestRecord => update_status(&mut output, patch, RecordStatus::Contested, now, &mut affected)?,
        PatchOperation::ReinforceRecord => { let target = patch.target_id.as_ref().context("reinforce patch missing target_id")?; for record in &mut output { if &record.id == target && record.namespace == patch.namespace { record.confidence = (record.confidence + 0.05).min(1.0); record.evidence.extend(patch.evidence.clone()); record.updated_at = now; } } affected.push(target.clone()); }
        PatchOperation::ResolveContest => { let target = patch.target_id.as_ref().context("resolve contest patch missing target_id")?; match patch.contest_resolution.as_ref().context("resolve contest patch missing contest_resolution")? { ContestResolution::Uphold => update_status(&mut output, patch, RecordStatus::Active, now, &mut affected)?, ContestResolution::Tombstone => update_status(&mut output, patch, RecordStatus::Tombstoned, now, &mut affected)?, ContestResolution::Supersede => { update_status(&mut output, patch, RecordStatus::Superseded, now, &mut affected)?; let mut replacement = patch.proposed_record.clone().context("resolve contest supersede patch missing replacement record")?; replacement.supersedes.push(target.clone()); affected.push(replacement.id.clone()); output.push(replacement); } } }
    }
    affected.sort(); affected.dedup(); Ok((output, affected))
}
fn update_status(records: &mut [Record], patch: &Patch, status: RecordStatus, now: DateTime<Utc>, affected: &mut Vec<String>) -> Result<()> { let target = patch.target_id.as_ref().context("patch missing target_id")?; for record in records { if &record.id == target && record.namespace == patch.namespace { record.status = status.clone(); record.updated_at = now; } } affected.push(target.clone()); Ok(()) }

impl GovernanceEngine {
    pub fn simulate_patch(&self, patch_id: &str) -> Result<PatchSimulationReport> {
        let records = self.store().load_records()?; let patches = self.store().load_patches()?; let events = self.store().load_events()?; let recall_events = self.store().load_recall_events()?;
        let patch = patches.iter().rev().find(|patch| patch.id == patch_id).context("patch not found")?;
        if patch.status != PatchStatus::Proposed { bail!("patch_not_pending: only proposed patches can be simulated"); }
        let decision = validate_patch(patch, &records); if !decision.can_apply(true) { bail!("patch cannot be simulated: {:?}", decision.reasons); }
        let now = Utc::now();
        let snapshot_token = format!("{:x}", Sha256::digest(serde_json::to_vec(&(&records, patch))?));
        let (predicted, affected) = transition_records(&records, patch, now)?;
        let score = |items: &[Record]| { let memory = analyze_memory_quality(items, &patch.namespace, now); let graph = build_memory_graph(items, &patches, &events, &patch.namespace, 5000, 10000, now); let relationships = analyze_relationship_quality(&graph, items, now); let recall = analyze_recall_effectiveness(items, &recall_events, &patch.namespace, now); let store = build_store_quality(&memory, &relationships, Some(&recall), 0, 0, now); SimulationScorecard { memory_quality: memory.summary.average_quality, relationship_quality: relationships.summary.average_relationship_quality, store_quality: store.overall_score, record_count: items.len(), active_record_count: items.iter().filter(|record| record.status == RecordStatus::Active).count() } };
        let before = score(&records); let after = score(&predicted);
        Ok(PatchSimulationReport { schema_version: 1, generated_at: now, patch_id: patch.id.clone(), snapshot_token, predicted_patch_status: PatchStatus::Applied, affected_record_ids: affected, memory_quality_delta: after.memory_quality as i32 - before.memory_quality as i32, relationship_quality_delta: after.relationship_quality as i32 - before.relationship_quality as i32, store_quality_delta: after.store_quality as i32 - before.store_quality as i32, before, after, review_required: true, mutation_performed: false })
    }
}
