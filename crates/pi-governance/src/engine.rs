use anyhow::{bail, Context, Result};
use chrono::{DateTime, Utc};
use pi_core::{
    validate_patch, validate_record, ContextBundle, DecisionStatus, EvidenceRef,
    GovernanceDecision, Patch, PatchOperation, PatchStatus, Record, RecordClass, RecordStatus,
    RetrievalBudget, SchemaFileAudit, Scope, StoreEvent, CURRENT_SCHEMA_VERSION,
};
use pi_retrieval::retrieve;
use pi_store::JsonlStore;
use serde::Serialize;
use std::collections::{HashMap, HashSet};

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
pub struct ApplyPatchResult {
    pub patch_id: String,
    pub applied: bool,
    pub latest_status_before: Option<PatchStatus>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PatchSummary {
    pub patch_id: String,
    pub operation: PatchOperation,
    pub latest_status: PatchStatus,
    pub target_id: Option<String>,
    pub proposed_record_id: Option<String>,
    pub proposed_record_class: Option<RecordClass>,
    pub proposed_record_claim: Option<String>,
    pub reason: String,
    pub evidence_count: usize,
    pub history_entries: usize,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PatchInspection {
    pub summary: PatchSummary,
    pub current_decision: Option<GovernanceDecision>,
    pub can_apply_without_force: bool,
    pub can_apply_with_force: bool,
    pub history: Vec<Patch>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DoctorReport {
    pub store_dir: String,
    pub lock_path: String,
    pub schema_version: u32,
    pub schema_audits: Vec<SchemaFileAudit>,
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
        let session = self.store.write_session()?;

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

        let existing = session.load_records()?;
        let decision = validate_patch(&patch, &existing);

        if decision.status == DecisionStatus::Reject {
            session.append_patch(&patch.rejected_copy())?;
            session.append_event(&StoreEvent::warning(
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

        session.append_patch(&patch)?;

        let mut applied = false;

        if apply_now {
            Self::apply_patch_object_locked(&session, &patch, force)?;
            applied = true;
        }

        Ok(ProposalResult {
            patch_id: patch.id,
            record_id: Some(record.id),
            decision,
            queued: true,
            applied,
        })
    }

    pub fn apply_patch_by_id(&self, patch_id: &str, force: bool) -> Result<ApplyPatchResult> {
        self.store.init()?;
        let session = self.store.write_session()?;

        let patches = session.load_patches()?;
        let history: Vec<&Patch> = patches.iter().filter(|patch| patch.id == patch_id).collect();

        let Some(latest) = history.last().copied() else {
            bail!(
                "patch_not_found: no patch history found for id {patch_id}; check the store path and patch id"
            );
        };

        let latest_status_before = latest.status.clone();

        if latest.status != PatchStatus::Proposed {
            bail!(
                "patch_not_pending: patch {patch_id} cannot be applied because its latest status is {:?}; only proposed patches can be applied",
                latest.status
            );
        }

        Self::apply_patch_object_locked(&session, latest, force)?;

        Ok(ApplyPatchResult {
            patch_id: patch_id.to_string(),
            applied: true,
            latest_status_before: Some(latest_status_before),
            message: "patch applied".to_string(),
        })
    }

    fn apply_patch_object_locked(
        session: &pi_store::JsonlStoreWriteSession<'_>,
        patch: &Patch,
        force: bool,
    ) -> Result<()> {
        let mut records = session.load_records()?;
        let decision = validate_patch(patch, &records);

        if !decision.can_apply(force) {
            bail!(
                "patch_requires_review: patch {} cannot be applied without force/manual review: {:?} — {:?}",
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
                    bail!("record_conflict: record {} already exists", record.id);
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

        session.overwrite_records_atomic(&records)?;
        session.append_patch(&patch.applied_copy())?;
        session.append_event(&StoreEvent::info("patch applied", Some(patch.id.clone())))?;

        Ok(())
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

    pub fn list_patches(&self, limit: usize) -> Result<Vec<PatchSummary>> {
        self.store.init()?;

        let patches = self.store.load_patches()?;
        let mut seen = HashSet::new();
        let mut summaries = Vec::new();

        for latest in patches.iter().rev() {
            if !seen.insert(latest.id.clone()) {
                continue;
            }

            let history_entries = patches
                .iter()
                .filter(|patch| patch.id == latest.id)
                .count();

            summaries.push(Self::summarize_patch(latest, history_entries));

            if summaries.len() >= limit {
                break;
            }
        }

        Ok(summaries)
    }

    pub fn inspect_patch(&self, patch_id: &str) -> Result<PatchInspection> {
        self.store.init()?;

        let records = self.store.load_records()?;
        let patches = self.store.load_patches()?;
        let history: Vec<Patch> = patches
            .into_iter()
            .filter(|patch| patch.id == patch_id)
            .collect();

        let Some(latest) = history.last() else {
            bail!(
                "patch_not_found: no patch history found for id {patch_id}; check the store path and patch id"
            );
        };

        let summary = Self::summarize_patch(latest, history.len());

        let current_decision = if latest.status == PatchStatus::Proposed {
            Some(validate_patch(latest, &records))
        } else {
            None
        };

        let can_apply_without_force = current_decision
            .as_ref()
            .map(|decision| decision.can_apply(false))
            .unwrap_or(false);

        let can_apply_with_force = current_decision
            .as_ref()
            .map(|decision| decision.can_apply(true))
            .unwrap_or(false);

        Ok(PatchInspection {
            summary,
            current_decision,
            can_apply_without_force,
            can_apply_with_force,
            history,
        })
    }

    pub fn doctor(&self) -> Result<DoctorReport> {
        self.store.init()?;

        let records = self.store.load_records()?;
        let patches = self.store.load_patches()?;
        let events = self.store.load_events()?;
        let schema_audits = self.store.audit_schema_versions(CURRENT_SCHEMA_VERSION)?;

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

        for audit in &schema_audits {
            if audit.missing_schema_version > 0 {
                warnings.push(format!(
                    "{} has {} entrie(s) without schema_version; they will load with the current default but should be migrated later",
                    audit.file_name, audit.missing_schema_version
                ));
            }

            if audit.mismatched_schema_version > 0 {
                warnings.push(format!(
                    "{} has {} entrie(s) with a non-current schema_version",
                    audit.file_name, audit.mismatched_schema_version
                ));
            }

            if audit.invalid_json_lines > 0 {
                errors.push(format!(
                    "{} has {} invalid JSONL entrie(s)",
                    audit.file_name, audit.invalid_json_lines
                ));
            }
        }

        Ok(DoctorReport {
            store_dir: self.store.root().display().to_string(),
            lock_path: self.store.lock_path().display().to_string(),
            schema_version: CURRENT_SCHEMA_VERSION,
            schema_audits,
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

    fn summarize_patch(patch: &Patch, history_entries: usize) -> PatchSummary {
        let proposed_record = patch.proposed_record.as_ref();

        PatchSummary {
            patch_id: patch.id.clone(),
            operation: patch.operation.clone(),
            latest_status: patch.status.clone(),
            target_id: patch.target_id.clone(),
            proposed_record_id: proposed_record.map(|record| record.id.clone()),
            proposed_record_class: proposed_record.map(|record| record.class.clone()),
            proposed_record_claim: proposed_record.map(|record| record.claim.clone()),
            reason: patch.reason.clone(),
            evidence_count: patch.evidence.len(),
            history_entries,
            created_at: patch.created_at.clone(),
            updated_at: patch.updated_at.clone(),
        }
    }
}
