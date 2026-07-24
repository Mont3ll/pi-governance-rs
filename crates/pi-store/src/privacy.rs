use crate::{create_store_backup_with_label, JsonlStore, StoreBackupReport};
use anyhow::{bail, Context, Result};
use chrono::Utc;
use pi_governance_core::{Durability, Record, RecordStatus, Scope, SourceKind, TrustClass};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::fs;

#[derive(Debug, Clone, Serialize)]
pub struct PrivacyPurgePlan {
    pub dry_run: bool,
    pub mutation_performed: bool,
    pub fingerprint: String,
    pub namespace: String,
    pub target_id: String,
    pub target_found: bool,
    pub target_status: Option<RecordStatus>,
    pub already_purged: bool,
    pub reason_sha256: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PrivacyPurgeApplyResult {
    pub dry_run: bool,
    pub mutation_performed: bool,
    pub fingerprint: String,
    pub namespace: String,
    pub target_id: String,
    pub already_purged: bool,
    pub backup: Option<StoreBackupReport>,
    pub report_path: Option<String>,
    pub reason_sha256: String,
}

fn hash(bytes: impl AsRef<[u8]>) -> String {
    format!("{:x}", Sha256::digest(bytes.as_ref()))
}

fn validate_reason(reason: &str) -> Result<()> {
    if reason.trim().len() < 8 {
        bail!("privacy purge requires a meaningful reason");
    }
    Ok(())
}

fn plan_from_records(
    records: &[Record],
    namespace: &str,
    target_id: &str,
    reason: &str,
) -> Result<PrivacyPurgePlan> {
    validate_reason(reason)?;
    let matches: Vec<&Record> = records
        .iter()
        .filter(|record| record.namespace == namespace && record.id == target_id)
        .collect();
    if matches.len() > 1 {
        bail!("privacy purge target has duplicate canonical rows; run store-integrity first");
    }
    let record = matches.first().copied();
    let already_purged = record.is_some_and(|record| {
        record.status == RecordStatus::Tombstoned
            && record.claim == "[privacy purged]"
            && record.evidence.is_empty()
            && record.tags.is_empty()
    });
    let reason_sha256 = hash(reason.trim());
    let fingerprint = hash(serde_json::to_vec(&(
        namespace,
        target_id,
        &reason_sha256,
        records,
    ))?);
    Ok(PrivacyPurgePlan {
        dry_run: true,
        mutation_performed: false,
        fingerprint,
        namespace: namespace.to_string(),
        target_id: target_id.to_string(),
        target_found: record.is_some(),
        target_status: record.map(|record| record.status.clone()),
        already_purged,
        reason_sha256,
    })
}

pub fn plan_record_privacy_purge(
    store: &JsonlStore,
    namespace: &str,
    target_id: &str,
    reason: &str,
) -> Result<PrivacyPurgePlan> {
    plan_from_records(&store.load_records()?, namespace, target_id, reason)
}

pub fn apply_record_privacy_purge(
    store: &JsonlStore,
    namespace: &str,
    target_id: &str,
    reason: &str,
    expected_fingerprint: &str,
) -> Result<PrivacyPurgeApplyResult> {
    let session = store.write_session()?;
    let mut records = session.load_records()?;
    let plan = plan_from_records(&records, namespace, target_id, reason)?;
    if plan.fingerprint != expected_fingerprint {
        bail!("privacy purge preview is stale; run preview again");
    }
    if !plan.target_found {
        bail!("privacy purge target not found");
    }
    let base = PrivacyPurgeApplyResult {
        dry_run: false,
        mutation_performed: false,
        fingerprint: plan.fingerprint.clone(),
        namespace: namespace.to_string(),
        target_id: target_id.to_string(),
        already_purged: plan.already_purged,
        backup: None,
        report_path: None,
        reason_sha256: plan.reason_sha256.clone(),
    };
    if plan.already_purged {
        return Ok(base);
    }

    let backup = create_store_backup_with_label(
        store.root(),
        "privacy-purge-v1",
        &[("records.jsonl", &store.records_path)],
    )?;
    let reports = store.root().join("reports");
    fs::create_dir_all(&reports).with_context(|| format!("failed to create {:?}", reports))?;
    let report_path = reports.join(format!(
        "privacy-purge-{}-{}.json",
        target_id,
        Utc::now().timestamp_millis()
    ));
    fs::write(
        &report_path,
        serde_json::to_vec_pretty(
            &serde_json::json!({"state":"prepared","plan":plan,"backup":backup}),
        )?,
    )?;

    let record = records
        .iter_mut()
        .find(|record| record.namespace == namespace && record.id == target_id)
        .context("privacy purge target disappeared")?;
    record.claim = "[privacy purged]".to_string();
    record.evidence.clear();
    record.tags.clear();
    record.supersedes.clear();
    record.scope = Scope::global();
    record.confidence = 0.0;
    record.status = RecordStatus::Tombstoned;
    record.trust_class = TrustClass::Unknown;
    record.durability = Durability::Unknown;
    record.source_kind = SourceKind::Unknown;
    record.rule_type = None;
    record.updated_at = Utc::now();
    session.overwrite_records_atomic(&records)?;

    let result = PrivacyPurgeApplyResult {
        mutation_performed: true,
        backup: Some(backup),
        report_path: Some(report_path.display().to_string()),
        ..base
    };
    fs::write(
        &report_path,
        serde_json::to_vec_pretty(&serde_json::json!({"state":"committed","result":result}))?,
    )?;
    Ok(result)
}
