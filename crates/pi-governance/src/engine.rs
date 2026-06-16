use anyhow::{bail, Context, Result};
use chrono::Utc;
use pi_core::{
    validate_patch, validate_record, ContextBundle, DecisionStatus, EvidenceRef,
    GovernanceDecision, Patch, PatchOperation, PatchStatus, Record, RecordClass, RecordStatus,
    RetrievalBudget, Scope, StoreEvent,
};
use pi_retrieval::retrieve;
use pi_store::JsonlStore;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ProposalInput {
    pub class: RecordClass,
    pub claim: String,
    pub confidence: f32,
    pub scope: Scope,
    pub tags: Vec<String>,
    pub evidence_refs: Vec<EvidenceRef>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProposalResult {
    pub patch_id: String,
    pub record_id: Option<String>,
    pub decision: GovernanceDecision,
    pub queued: bool,
    pub applied: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct DoctorReport {
    pub store_dir: String,
    pub total_records: usize,
    pub active_records: usize,
    pub superseded_records: usize,
    pub tombstoned_records: usize,
    pub total_patches: usize,
    pub proposed_patches_latest: usize,
    pub applied_patches_latest: usize,
    pub rejected_patches_latest: usize,
    pub total_events: usize,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct GovernanceEngine {
    store: JsonlStore,
}

impl GovernanceEngine {
    pub fn new(store: JsonlStore) -> Self {
        Self { store }
    }

    pub fn init(&self) -> Result<()> {
        self.store.init()
    }

    pub fn propose_record(
        &self,
        input: ProposalInput,
        apply_now: bool,
        force: bool,
    ) -> Result<ProposalResult> {
        self.store.init()?;

        let record = Record::new(
            input.class,
            input.claim,
            input.confidence,
            input.scope,
            input.tags,
            input.evidence_refs,
        );

        let patch = Patch::propose_record(
            record.clone(),
            input
                .reason
                .unwrap_or_else(|| "proposed by coding agent".to_string()),
        );

        let existing = self.store.load_records()?;
        let decision = validate_patch(&patch, &existing);

        if decision.status == DecisionStatus::Reject {
            self.store.append_patch(&patch.rejected_copy())?;
            self.store.append_event(&StoreEvent::warning(
                "patch rejected by governance policy",
                Some(patch.id.clone()),
            ))?;

            return Ok(ProposalResult {
                patch_id: patch.id,
                record_id: Some(record.id),
                decision,
                queued: false,
                applied: false,
            });
        }

        self.store.append_patch(&patch)?;

        let mut applied = false;

        if apply_now {
            applied = self.apply_patch_object(&patch, force)?;
        }

        Ok(ProposalResult {
            patch_id: patch.id,
            record_id: Some(record.id),
            decision,
            queued: true,
            applied,
        })
    }

    pub fn apply_patch_by_id(&self, patch_id: &str, force: bool) -> Result<bool> {
        self.store.init()?;

        let patches = self.store.load_patches()?;

        let Some(patch) = patches
            .iter()
            .rev()
            .find(|patch| patch.id == patch_id && patch.status == PatchStatus::Proposed)
        else {
            bail!("no proposed patch found with id {patch_id}");
        };

        self.apply_patch_object(patch, force)
    }

    fn apply_patch_object(&self, patch: &Patch, force: bool) -> Result<bool> {
        let mut records = self.store.load_records()?;
        let decision = validate_patch(patch, &records);

        if !decision.can_apply(force) {
            bail!(
                "patch {} cannot be applied without review: {:?} — {:?}",
                patch.id,
                decision.status,
                decision.reasons
            );
        }

        match patch.operation {
            PatchOperation::ProposeRecord => {
                let record = patch
                    .proposed_record
                    .clone()
                    .context("propose_record patch missing proposed_record")?;

                if records.iter().any(|existing| existing.id == record.id) {
                    bail!("record {} already exists", record.id);
                }

                records.push(record);
            }

            PatchOperation::SupersedeRecord => {
                let target_id = patch
                    .target_id
                    .as_ref()
                    .context("supersede patch missing target_id")?;

                for record in &mut records {
                    if &record.id == target_id {
                        record.status = RecordStatus::Superseded;
                        record.updated_at = Utc::now();
                    }
                }

                let mut replacement = patch
                    .proposed_record
                    .clone()
                    .context("supersede patch missing replacement record")?;

                replacement.supersedes.push(target_id.clone());
                records.push(replacement);
            }

            PatchOperation::TombstoneRecord => {
                let target_id = patch
                    .target_id
                    .as_ref()
                    .context("tombstone patch missing target_id")?;

                for record in &mut records {
                    if &record.id == target_id {
                        record.status = RecordStatus::Tombstoned;
                        record.updated_at = Utc::now();
                    }
                }
            }

            PatchOperation::ReinforceRecord => {
                let target_id = patch
                    .target_id
                    .as_ref()
                    .context("reinforce patch missing target_id")?;

                for record in &mut records {
                    if &record.id == target_id {
                        record.confidence = (record.confidence + 0.05).min(1.0);
                        record.evidence.extend(patch.evidence.clone());
                        record.updated_at = Utc::now();
                    }
                }
            }
        }

        self.store.overwrite_records_atomic(&records)?;
        self.store.append_patch(&patch.applied_copy())?;
        self.store
            .append_event(&StoreEvent::info("patch applied", Some(patch.id.clone())))?;

        Ok(true)
    }

    pub fn retrieve_context(
        &self,
        query: impl Into<String>,
        project: Option<String>,
        max_tokens: usize,
    ) -> Result<ContextBundle> {
        self.store.init()?;

        let records = self.store.load_records()?;

        Ok(retrieve(
            &records,
            query,
            project,
            RetrievalBudget { max_tokens },
        ))
    }

    pub fn list_records(&self, limit: usize) -> Result<Vec<Record>> {
        self.store.init()?;

        let mut records = self.store.load_records()?;
        records.reverse();
        records.truncate(limit);

        Ok(records)
    }

    pub fn doctor(&self) -> Result<DoctorReport> {
        self.store.init()?;

        let records = self.store.load_records()?;
        let patches = self.store.load_patches()?;
        let events = self.store.load_events()?;

        let mut warnings = Vec::new();
        let mut errors = Vec::new();

        let active_records = records
            .iter()
            .filter(|record| record.status == RecordStatus::Active)
            .count();

        let superseded_records = records
            .iter()
            .filter(|record| record.status == RecordStatus::Superseded)
            .count();

        let tombstoned_records = records
            .iter()
            .filter(|record| record.status == RecordStatus::Tombstoned)
            .count();

        for record in &records {
            let decision = validate_record(record, &[]);

            match decision.status {
                DecisionStatus::Reject => {
                    errors.push(format!(
                        "record {} violates policy: {:?}",
                        record.id, decision.reasons
                    ));
                }
                DecisionStatus::ManualReview => {
                    warnings.push(format!(
                        "record {} would require review if proposed today: {:?}",
                        record.id, decision.reasons
                    ));
                }
                DecisionStatus::Allow => {}
            }
        }

        let mut latest_patch_status: HashMap<String, PatchStatus> = HashMap::new();

        for patch in &patches {
            latest_patch_status.insert(patch.id.clone(), patch.status.clone());
        }

        let proposed_patches_latest = latest_patch_status
            .values()
            .filter(|status| **status == PatchStatus::Proposed)
            .count();

        let applied_patches_latest = latest_patch_status
            .values()
            .filter(|status| **status == PatchStatus::Applied)
            .count();

        let rejected_patches_latest = latest_patch_status
            .values()
            .filter(|status| **status == PatchStatus::Rejected)
            .count();

        if proposed_patches_latest > 0 {
            warnings.push(format!(
                "{proposed_patches_latest} proposed patch(es) awaiting application or review"
            ));
        }

        Ok(DoctorReport {
            store_dir: self.store.root().display().to_string(),
            total_records: records.len(),
            active_records,
            superseded_records,
            tombstoned_records,
            total_patches: patches.len(),
            proposed_patches_latest,
            applied_patches_latest,
            rejected_patches_latest,
            total_events: events.len(),
            warnings,
            errors,
        })
    }
}
