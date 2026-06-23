use anyhow::{bail, Context, Result};
use chrono::{DateTime, Utc};
use pi_core::{
    validate_patch, validate_record, ContestResolution, ContextBundle, DecisionStatus, EvidenceRef,
    GovernanceDecision, Patch, PatchOperation, PatchStatus, Record, RecordClass, RecordStatus,
    default_namespace, PiConfig, PolicyProfile, RetrievalBudget, RetrievalOptions, SchemaFileAudit, Scope, StoreEvent, CURRENT_SCHEMA_VERSION,
};
use pi_retrieval::{retrieve, retrieve_with_options};
use pi_store::{
    JsonlStore, SchemaMigrationOptions, SchemaMigrationReport, StoreExportBundle,
    StoreExportOptions, StoreImportOptions, StoreImportReport,
};
use serde::Serialize;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct ProposalInput {
    pub namespace: String,
    pub class: RecordClass,
    pub claim: String,
    pub confidence: f32,
    pub scope: Scope,
    pub tags: Vec<String>,
    pub evidence_refs: Vec<EvidenceRef>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone)]
pub struct MigrationInput {
    pub dry_run: bool,
    pub backup: bool,
}

#[derive(Debug, Clone)]
pub struct ExportInput {
    pub namespace: Option<String>,
    pub all_namespaces: bool,
    pub project: Option<String>,
    pub redacted: bool,
}

#[derive(Debug, Clone)]
pub struct ImportInput {
    pub namespace: String,
    pub preserve_namespaces: bool,
    pub dry_run: bool,
    pub backup: bool,
}

#[derive(Debug, Clone)]
pub struct SupersedeInput {
    pub namespace: String,
    pub target_id: String,
    pub class: RecordClass,
    pub claim: String,
    pub confidence: f32,
    pub scope: Scope,
    pub tags: Vec<String>,
    pub evidence_refs: Vec<EvidenceRef>,
    pub reason: String,
}

#[derive(Debug, Clone)]
pub struct TombstoneInput {
    pub namespace: String,
    pub target_id: String,
    pub evidence_refs: Vec<EvidenceRef>,
    pub reason: String,
}

#[derive(Debug, Clone)]
pub struct ReinforceInput {
    pub namespace: String,
    pub target_id: String,
    pub evidence_refs: Vec<EvidenceRef>,
    pub reason: String,
}

#[derive(Debug, Clone)]
pub struct ContestInput {
    pub namespace: String,
    pub target_id: String,
    pub evidence_refs: Vec<EvidenceRef>,
    pub reason: String,
}

#[derive(Debug, Clone)]
pub struct ResolveContestInput {
    pub namespace: String,
    pub target_id: String,
    pub resolution: ContestResolution,
    pub class: Option<RecordClass>,
    pub claim: Option<String>,
    pub confidence: f32,
    pub scope: Scope,
    pub tags: Vec<String>,
    pub evidence_refs: Vec<EvidenceRef>,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProposalResult {
    pub policy_profile: PolicyProfile,
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
pub struct NamespaceSummary {
    pub namespace: String,
    pub records: usize,
    pub active: usize,
    pub contested: usize,
    pub superseded: usize,
    pub tombstoned: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct NamespaceDoctorReport {
    pub namespaces: usize,
    pub default_namespace: String,
    pub records_without_explicit_namespace: usize,
    pub cross_namespace_duplicate_ids: usize,
    pub summaries: Vec<NamespaceSummary>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DoctorReport {
    pub store_dir: String,
    pub current_namespace: String,
    pub namespaces: usize,
    pub records_in_current_namespace: usize,
    pub active_in_current_namespace: usize,
    pub contested_in_current_namespace: usize,
    pub superseded_in_current_namespace: usize,
    pub tombstoned_in_current_namespace: usize,
    pub lock_path: String,
    pub schema_version: u32,
    pub schema_audits: Vec<SchemaFileAudit>,
    pub migration_needed: bool,
    pub total_records: usize,
    pub active_records: usize,
    pub superseded_records: usize,
    pub tombstoned_records: usize,
    pub contested_records: usize,
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

    pub fn config(&self) -> Result<PiConfig> { self.store.load_config() }

    pub fn set_policy(&self, namespace: &str, policy: PolicyProfile) -> Result<PiConfig> {
        let mut config = self.store.load_config()?;
        config.set_policy(namespace, policy);
        self.store.save_config(&config)?;
        Ok(config)
    }

    pub fn effective_policy(&self, namespace: &str) -> Result<PolicyProfile> {
        Ok(self.store.load_config()?.effective_policy(namespace))
    }

    pub fn policy_doctor(&self) -> Result<PiConfig> { self.store.load_config() }

    pub fn policy_explain(operation: &str) -> String {
        let op = operation;
        match op {
            "propose" => "propose: permissive=allow, standard=allow, strict=manual_review".to_string(),
            "reinforce" => "reinforce: permissive=allow, standard=allow, strict=manual_review".to_string(),
            "supersede" => "supersede: permissive=allow with warning, standard=manual_review, strict=manual_review".to_string(),
            "tombstone" => "tombstone: permissive=manual_review, standard=manual_review, strict=manual_review".to_string(),
            "contest" => "contest: permissive=manual_review, standard=manual_review, strict=manual_review".to_string(),
            "resolve-contest" => "resolve-contest: uphold permissive=allow; otherwise manual_review".to_string(),
            "import" => "import: all profiles allow; duplicate and schema safety checks still apply".to_string(),
            _ => format!("unknown operation: {op}"),
        }
    }

    fn apply_policy_profile(mut decision: GovernanceDecision, patch: &Patch, profile: PolicyProfile) -> GovernanceDecision {
        if decision.status == DecisionStatus::Reject { return decision; }
        match profile {
            PolicyProfile::Standard => decision,
            PolicyProfile::Strict => {
                decision.escalate_to_manual(format!("policy profile '{}' required manual review", profile));
                decision
            }
            PolicyProfile::Permissive => {
                match patch.operation {
                    PatchOperation::SupersedeRecord => {
                        if decision.status == DecisionStatus::ManualReview { decision.status = DecisionStatus::Allow; }
                        decision.add_warning("policy profile 'permissive' allowed supersede with warning");
                    }
                    PatchOperation::ResolveContest if patch.contest_resolution == Some(ContestResolution::Uphold) => {
                        if decision.status == DecisionStatus::ManualReview { decision.status = DecisionStatus::Allow; }
                    }
                    _ => {}
                }
                decision
            }
        }
    }

    pub fn propose_record(
        &self,
        input: ProposalInput,
        apply_now: bool,
        force: bool,
    ) -> Result<ProposalResult> {
        self.store.init()?;
        let session = self.store.write_session()?;

        let mut record = Record::new(
            input.class,
            input.claim,
            input.confidence,
            input.scope,
            input.tags,
            input.evidence_refs,
        );
        record.namespace = input.namespace;

        let patch = Patch::propose_record(
            record.clone(),
            input
                .reason
                .unwrap_or_else(|| "proposed by coding agent".to_string()),
        );

        let existing = session.load_records()?;
        let policy_profile = self.effective_policy(&patch.namespace)?;
        let decision = Self::apply_policy_profile(validate_patch(&patch, &existing), &patch, policy_profile);

        if decision.status == DecisionStatus::Reject {
            session.append_patch(&patch.rejected_copy())?;
            session.append_event(&StoreEvent::warning(
                "patch rejected by governance policy",
                Some(patch.id.clone()),
            ))?;

            return Ok(ProposalResult {
                policy_profile,
                patch_id: patch.id,
                record_id: Some(record.id),
                decision,
                queued: false,
                applied: false,
            });
        }

        session.append_patch(&patch)?;

        let mut applied = false;

        if apply_now && decision.can_apply(force) {
            Self::apply_patch_object_locked(&session, &patch, force)?;
            applied = true;
        }

        Ok(ProposalResult {
            policy_profile,
            patch_id: patch.id,
            record_id: Some(record.id),
            decision,
            queued: true,
            applied,
        })
    }

    pub fn supersede_record(
        &self,
        input: SupersedeInput,
        apply_now: bool,
        force: bool,
    ) -> Result<ProposalResult> {
        self.store.init()?;
        let session = self.store.write_session()?;

        let namespace = input.namespace.clone();
        Self::ensure_record_in_namespace_locked(&session, &input.target_id, &namespace)?;
        let mut replacement = Record::new(
            input.class,
            input.claim,
            input.confidence,
            input.scope,
            input.tags,
            input.evidence_refs,
        );
        replacement.namespace = namespace.clone();

        let patch = Patch::supersede_record(input.target_id, replacement.clone(), input.reason);
        let existing = session.load_records()?;
        let policy_profile = self.effective_policy(&patch.namespace)?;
        let decision = Self::apply_policy_profile(validate_patch(&patch, &existing), &patch, policy_profile);

        if decision.status == DecisionStatus::Reject {
            session.append_patch(&patch.rejected_copy())?;
            session.append_event(&StoreEvent::warning(
                "supersede patch rejected by governance policy",
                Some(patch.id.clone()),
            ))?;

            return Ok(ProposalResult {
                policy_profile,
                patch_id: patch.id,
                record_id: Some(replacement.id),
                decision,
                queued: false,
                applied: false,
            });
        }

        session.append_patch(&patch)?;

        let mut applied = false;

        if apply_now && decision.can_apply(force) {
            Self::apply_patch_object_locked(&session, &patch, force)?;
            applied = true;
        }

        Ok(ProposalResult {
            policy_profile,
            patch_id: patch.id,
            record_id: Some(replacement.id),
            decision,
            queued: true,
            applied,
        })
    }

    pub fn tombstone_record(
        &self,
        input: TombstoneInput,
        apply_now: bool,
        force: bool,
    ) -> Result<ProposalResult> {
        self.store.init()?;
        let session = self.store.write_session()?;

        let namespace = input.namespace.clone();
        Self::ensure_record_in_namespace_locked(&session, &input.target_id, &namespace)?;
        let target_id = input.target_id.clone();
        let mut patch = Patch::tombstone_record(input.target_id, input.evidence_refs, input.reason);
        patch.namespace = namespace;
        let existing = session.load_records()?;
        let policy_profile = self.effective_policy(&patch.namespace)?;
        let decision = Self::apply_policy_profile(validate_patch(&patch, &existing), &patch, policy_profile);

        if decision.status == DecisionStatus::Reject {
            session.append_patch(&patch.rejected_copy())?;
            session.append_event(&StoreEvent::warning(
                "tombstone patch rejected by governance policy",
                Some(patch.id.clone()),
            ))?;

            return Ok(ProposalResult {
                policy_profile,
                patch_id: patch.id,
                record_id: Some(target_id),
                decision,
                queued: false,
                applied: false,
            });
        }

        session.append_patch(&patch)?;

        let mut applied = false;

        if apply_now && decision.can_apply(force) {
            Self::apply_patch_object_locked(&session, &patch, force)?;
            applied = true;
        }

        Ok(ProposalResult {
            policy_profile,
            patch_id: patch.id,
            record_id: Some(target_id),
            decision,
            queued: true,
            applied,
        })
    }

    pub fn reinforce_record(
        &self,
        input: ReinforceInput,
        apply_now: bool,
        force: bool,
    ) -> Result<ProposalResult> {
        self.store.init()?;
        let session = self.store.write_session()?;

        let namespace = input.namespace.clone();
        Self::ensure_record_in_namespace_locked(&session, &input.target_id, &namespace)?;
        let target_id = input.target_id.clone();
        let mut patch = Patch::reinforce_record(input.target_id, input.evidence_refs, input.reason);
        patch.namespace = namespace;
        let existing = session.load_records()?;
        let policy_profile = self.effective_policy(&patch.namespace)?;
        let decision = Self::apply_policy_profile(validate_patch(&patch, &existing), &patch, policy_profile);

        if decision.status == DecisionStatus::Reject {
            session.append_patch(&patch.rejected_copy())?;
            session.append_event(&StoreEvent::warning(
                "reinforce patch rejected by governance policy",
                Some(patch.id.clone()),
            ))?;

            return Ok(ProposalResult {
                policy_profile,
                patch_id: patch.id,
                record_id: Some(target_id),
                decision,
                queued: false,
                applied: false,
            });
        }

        session.append_patch(&patch)?;

        let mut applied = false;

        if apply_now && decision.can_apply(force) {
            Self::apply_patch_object_locked(&session, &patch, force)?;
            applied = true;
        }

        Ok(ProposalResult {
            policy_profile,
            patch_id: patch.id,
            record_id: Some(target_id),
            decision,
            queued: true,
            applied,
        })
    }

    pub fn contest_record(
        &self,
        input: ContestInput,
        apply_now: bool,
        force: bool,
    ) -> Result<ProposalResult> {
        self.store.init()?;
        let session = self.store.write_session()?;

        let namespace = input.namespace.clone();
        Self::ensure_record_in_namespace_locked(&session, &input.target_id, &namespace)?;
        let target_id = input.target_id.clone();
        let mut patch = Patch::contest_record(input.target_id, input.evidence_refs, input.reason);
        patch.namespace = namespace;
        let existing = session.load_records()?;
        let policy_profile = self.effective_policy(&patch.namespace)?;
        let decision = Self::apply_policy_profile(validate_patch(&patch, &existing), &patch, policy_profile);

        if decision.status == DecisionStatus::Reject {
            session.append_patch(&patch.rejected_copy())?;
            session.append_event(&StoreEvent::warning(
                "contest patch rejected by governance policy",
                Some(patch.id.clone()),
            ))?;

            return Ok(ProposalResult {
                policy_profile,
                patch_id: patch.id,
                record_id: Some(target_id),
                decision,
                queued: false,
                applied: false,
            });
        }

        session.append_patch(&patch)?;

        let mut applied = false;

        if apply_now && decision.can_apply(force) {
            Self::apply_patch_object_locked(&session, &patch, force)?;
            applied = true;
        }

        Ok(ProposalResult {
            policy_profile,
            patch_id: patch.id,
            record_id: Some(target_id),
            decision,
            queued: true,
            applied,
        })
    }

    pub fn resolve_contest(
        &self,
        input: ResolveContestInput,
        apply_now: bool,
        force: bool,
    ) -> Result<ProposalResult> {
        self.store.init()?;
        let session = self.store.write_session()?;

        let namespace = input.namespace.clone();
        Self::ensure_record_in_namespace_locked(&session, &input.target_id, &namespace)?;
        let target_id = input.target_id.clone();
        let replacement = match input.resolution {
            ContestResolution::Supersede => {
                let class = input
                    .class
                    .context("resolve_contest supersede requires --class")?;
                let claim = input
                    .claim
                    .clone()
                    .context("resolve_contest supersede requires --claim")?;

                Some(Record::new(
                    class,
                    claim,
                    input.confidence,
                    input.scope.clone(),
                    input.tags.clone(),
                    input.evidence_refs.clone(),
                ))
            }
            ContestResolution::Uphold | ContestResolution::Tombstone => None,
        };

        let replacement_id = replacement.as_ref().map(|record| record.id.clone());
        let patch = Patch::resolve_contest(
            input.target_id,
            input.resolution,
            replacement,
            input.evidence_refs,
            input.reason,
        );
        let existing = session.load_records()?;
        let policy_profile = self.effective_policy(&patch.namespace)?;
        let decision = Self::apply_policy_profile(validate_patch(&patch, &existing), &patch, policy_profile);

        if decision.status == DecisionStatus::Reject {
            session.append_patch(&patch.rejected_copy())?;
            session.append_event(&StoreEvent::warning(
                "contest resolution patch rejected by governance policy",
                Some(patch.id.clone()),
            ))?;

            return Ok(ProposalResult {
                policy_profile,
                patch_id: patch.id,
                record_id: replacement_id.or(Some(target_id)),
                decision,
                queued: false,
                applied: false,
            });
        }

        session.append_patch(&patch)?;

        let mut applied = false;

        if apply_now && decision.can_apply(force) {
            Self::apply_patch_object_locked(&session, &patch, force)?;
            applied = true;
        }

        Ok(ProposalResult {
            policy_profile,
            patch_id: patch.id,
            record_id: replacement_id.or(Some(target_id)),
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

    fn ensure_record_in_namespace_locked(
        session: &pi_store::JsonlStoreWriteSession<'_>,
        record_id: &str,
        namespace: &str,
    ) -> Result<()> {
        let records = session.load_records()?;
        if records.iter().any(|record| record.id == record_id && record.namespace == namespace) {
            return Ok(());
        }
        bail!("record not found in namespace '{namespace}'")
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
                    if &record.id == target_id && record.namespace == patch.namespace {
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
                    if &record.id == target_id && record.namespace == patch.namespace {
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
                    if &record.id == target_id && record.namespace == patch.namespace {
                        record.confidence = (record.confidence + 0.05).min(1.0);
                        record.evidence.extend(patch.evidence.clone());
                        record.updated_at = Utc::now();
                    }
                }
            }

            PatchOperation::ContestRecord => {
                let target_id = patch
                    .target_id
                    .as_ref()
                    .context("contest patch missing target_id")?;

                for record in &mut records {
                    if &record.id == target_id && record.namespace == patch.namespace {
                        record.status = RecordStatus::Contested;
                        record.updated_at = Utc::now();
                    }
                }
            }

            PatchOperation::ResolveContest => {
                let target_id = patch
                    .target_id
                    .as_ref()
                    .context("resolve contest patch missing target_id")?;
                let resolution = patch
                    .contest_resolution
                    .as_ref()
                    .context("resolve contest patch missing contest_resolution")?;

                match resolution {
                    ContestResolution::Uphold => {
                        for record in &mut records {
                            if &record.id == target_id && record.namespace == patch.namespace {
                                record.status = RecordStatus::Active;
                                record.updated_at = Utc::now();
                            }
                        }
                    }
                    ContestResolution::Tombstone => {
                        for record in &mut records {
                            if &record.id == target_id && record.namespace == patch.namespace {
                                record.status = RecordStatus::Tombstoned;
                                record.updated_at = Utc::now();
                            }
                        }
                    }
                    ContestResolution::Supersede => {
                        for record in &mut records {
                            if &record.id == target_id && record.namespace == patch.namespace {
                                record.status = RecordStatus::Superseded;
                                record.updated_at = Utc::now();
                            }
                        }

                        let mut replacement = patch
                            .proposed_record
                            .clone()
                            .context("resolve contest supersede patch missing replacement record")?;

                        replacement.supersedes.push(target_id.clone());
                        records.push(replacement);
                    }
                }
            }
        }

        session.overwrite_records_atomic(&records)?;
        session.append_patch(&patch.applied_copy())?;
        session.append_event(&StoreEvent::info("patch applied", Some(patch.id.clone())))?;

        Ok(())
    }

    pub fn migrate_store(&self, input: MigrationInput) -> Result<SchemaMigrationReport> {
        self.store.init()?;

        self.store.migrate_schema_versions(SchemaMigrationOptions {
            dry_run: input.dry_run,
            backup: input.backup,
        })
    }

    pub fn export_store(&self, input: ExportInput) -> Result<StoreExportBundle> {
        self.store.init()?;

        self.store.export_bundle(StoreExportOptions {
            namespace: input.namespace,
            all_namespaces: input.all_namespaces,
            project: input.project,
            redacted: input.redacted,
        })
    }

    pub fn export_store_to_path(
        &self,
        path: &std::path::Path,
        input: ExportInput,
    ) -> Result<StoreExportBundle> {
        self.store.init()?;

        self.store.export_bundle_to_path(
            path,
            StoreExportOptions {
                namespace: input.namespace,
                all_namespaces: input.all_namespaces,
                project: input.project,
                redacted: input.redacted,
            },
        )
    }

    pub fn import_store_from_path(
        &self,
        path: &std::path::Path,
        input: ImportInput,
    ) -> Result<StoreImportReport> {
        self.store.init()?;

        self.store.import_bundle_from_path(
            path,
            StoreImportOptions {
                namespace: input.namespace,
                preserve_namespaces: input.preserve_namespaces,
                dry_run: input.dry_run,
                backup: input.backup,
            },
        )
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

    pub fn retrieve_context_with_options(&self, options: RetrievalOptions) -> Result<ContextBundle> {
        self.store.init()?;
        let records = self.store.load_records()?;
        Ok(retrieve_with_options(&records, options))
    }

    pub fn list_records(&self, limit: usize) -> Result<Vec<Record>> {
        self.list_records_in_namespace(&default_namespace(), limit)
    }

    pub fn list_records_in_namespace(&self, namespace: &str, limit: usize) -> Result<Vec<Record>> {
        self.store.init()?;

        let mut records: Vec<Record> = self.store.load_records()?
            .into_iter()
            .filter(|record| record.namespace == namespace)
            .collect();
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

    pub fn namespace_summaries(&self) -> Result<Vec<NamespaceSummary>> {
        self.store.init()?;
        let records = self.store.load_records()?;
        let mut namespaces: Vec<String> = records.iter().map(|record| record.namespace.clone()).collect();
        namespaces.sort();
        namespaces.dedup();
        Ok(namespaces.into_iter().map(|namespace| {
            let in_ns: Vec<&Record> = records.iter().filter(|record| record.namespace == namespace).collect();
            NamespaceSummary {
                namespace,
                records: in_ns.len(),
                active: in_ns.iter().filter(|record| record.status == RecordStatus::Active).count(),
                contested: in_ns.iter().filter(|record| record.status == RecordStatus::Contested).count(),
                superseded: in_ns.iter().filter(|record| record.status == RecordStatus::Superseded).count(),
                tombstoned: in_ns.iter().filter(|record| record.status == RecordStatus::Tombstoned).count(),
            }
        }).collect())
    }

    pub fn namespace_doctor(&self) -> Result<NamespaceDoctorReport> {
        let summaries = self.namespace_summaries()?;
        let records = self.store.load_records()?;
        let mut by_id: HashMap<String, HashSet<String>> = HashMap::new();
        for record in &records {
            by_id.entry(record.id.clone()).or_default().insert(record.namespace.clone());
        }
        let cross_namespace_duplicate_ids = by_id.values().filter(|namespaces| namespaces.len() > 1).count();
        Ok(NamespaceDoctorReport {
            namespaces: summaries.len(),
            default_namespace: default_namespace(),
            records_without_explicit_namespace: 0,
            cross_namespace_duplicate_ids,
            summaries,
        })
    }

    pub fn doctor(&self) -> Result<DoctorReport> {
        self.doctor_in_namespace(&default_namespace())
    }

    pub fn doctor_in_namespace(&self, namespace: &str) -> Result<DoctorReport> {
        self.store.init()?;

        let records = self.store.load_records()?;
        let namespace_records: Vec<&Record> = records.iter().filter(|record| record.namespace == namespace).collect();
        let namespace_count = self.namespace_summaries()?.len();
        let records_in_current_namespace = namespace_records.len();
        let active_in_current_namespace = namespace_records.iter().filter(|record| record.status == RecordStatus::Active).count();
        let contested_in_current_namespace = namespace_records.iter().filter(|record| record.status == RecordStatus::Contested).count();
        let superseded_in_current_namespace = namespace_records.iter().filter(|record| record.status == RecordStatus::Superseded).count();
        let tombstoned_in_current_namespace = namespace_records.iter().filter(|record| record.status == RecordStatus::Tombstoned).count();
        let patches = self.store.load_patches()?;
        let events = self.store.load_events()?;
        let schema_audits = self.store.audit_schema_versions(CURRENT_SCHEMA_VERSION)?;
        let migration_needed = schema_audits.iter().any(|audit| {
            audit.missing_schema_version > 0 || audit.mismatched_schema_version > 0
        });

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

        let contested_records = records
            .iter()
            .filter(|record| record.status == RecordStatus::Contested)
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
            current_namespace: namespace.to_string(),
            namespaces: namespace_count,
            records_in_current_namespace,
            active_in_current_namespace,
            contested_in_current_namespace,
            superseded_in_current_namespace,
            tombstoned_in_current_namespace,
            lock_path: self.store.lock_path().display().to_string(),
            schema_version: CURRENT_SCHEMA_VERSION,
            schema_audits,
            migration_needed,
            total_records: records.len(),
            active_records,
            superseded_records,
            tombstoned_records,
            contested_records,
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
