use chrono::Utc;
use pi_governance_core::{
    EvidenceKind, EvidenceRef, Patch, PatchStatus, Record, RecordClass, RecordStatus, Scope,
};
use pi_governance_store::{apply_record_privacy_purge, plan_record_privacy_purge, JsonlStore};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_store_dir() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("pi-privacy-purge-{}-{nonce}", std::process::id()))
}

fn secret_record(status: RecordStatus) -> Record {
    let mut record = Record::new(
        RecordClass::Workflow,
        "legacy secret sk-prohibited-value",
        0.9,
        Scope::project("secret-project-token"),
        vec!["secret-tag".into()],
        vec![EvidenceRef::new(
            EvidenceKind::Conversation,
            "secret:evidence",
        )],
    );
    record.namespace = "alpha".into();
    record.id = "mem_secret".into();
    record.status = status;
    record.updated_at = Utc::now();
    record
}

#[test]
fn privacy_purge_previews_then_redacts_terminal_record_with_backup_and_report() {
    let root = temp_store_dir();
    let store = JsonlStore::new(&root);
    store.init().unwrap();
    store
        .append_record(&secret_record(RecordStatus::Superseded))
        .unwrap();
    let before = fs::read(root.join("records.jsonl")).unwrap();

    let preview =
        plan_record_privacy_purge(&store, "alpha", "mem_secret", "privacy cleanup", &[]).unwrap();
    assert!(preview.dry_run);
    assert!(!preview.mutation_performed);
    assert!(preview.target_found);
    assert_eq!(fs::read(root.join("records.jsonl")).unwrap(), before);

    let applied = apply_record_privacy_purge(
        &store,
        "alpha",
        "mem_secret",
        "privacy cleanup",
        &[],
        &preview.fingerprint,
    )
    .unwrap();
    assert!(applied.mutation_performed);
    assert!(applied
        .backup
        .as_ref()
        .is_some_and(|backup| fs::metadata(&backup.backup_dir).is_ok()));
    assert!(applied
        .report_path
        .as_ref()
        .is_some_and(|path| fs::metadata(path).is_ok()));
    let record = store.load_records().unwrap().pop().unwrap();
    assert_eq!(record.id, "mem_secret");
    assert_eq!(record.namespace, "alpha");
    assert_eq!(record.status, RecordStatus::Tombstoned);
    assert_eq!(record.claim, "[privacy purged]");
    assert!(record.evidence.is_empty());
    assert!(record.tags.is_empty());
    assert_eq!(record.scope, Scope::global());
    let canonical = fs::read_to_string(root.join("records.jsonl")).unwrap();
    assert!(!canonical.contains("sk-prohibited-value"));
    assert!(!canonical.contains("secret-project-token"));

    let second =
        plan_record_privacy_purge(&store, "alpha", "mem_secret", "privacy cleanup", &[]).unwrap();
    assert!(second.already_purged);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn privacy_purge_redacts_correlated_historical_patch_payloads_after_record_was_purged() {
    let root = temp_store_dir();
    let store = JsonlStore::new(&root);
    store.init().unwrap();
    store
        .append_record(&secret_record(RecordStatus::Active))
        .unwrap();
    let first =
        plan_record_privacy_purge(&store, "alpha", "mem_secret", "privacy cleanup", &[]).unwrap();
    apply_record_privacy_purge(
        &store,
        "alpha",
        "mem_secret",
        "privacy cleanup",
        &[],
        &first.fingerprint,
    )
    .unwrap();

    let mut embedded = secret_record(RecordStatus::Active);
    embedded.id = "imported_cap_secret".into();
    let mut patch = Patch::propose_record(embedded, "legacy secret sk-prohibited-value");
    patch.id = "explicit_patch".into();
    patch.status = PatchStatus::Applied;
    store.append_patch(&patch).unwrap();
    let mut independent_record = secret_record(RecordStatus::Active);
    independent_record.id = "independent_record".into();
    independent_record.claim = "independent safe claim".into();
    let mut independent_patch = Patch::propose_record(independent_record, "independent reason");
    independent_patch.id = "cap_secret".into();
    store.append_patch(&independent_patch).unwrap();

    let explicit_patch_ids = vec!["explicit_patch".to_string()];
    let preview = plan_record_privacy_purge(
        &store,
        "alpha",
        "mem_secret",
        "privacy cleanup",
        &explicit_patch_ids,
    )
    .unwrap();
    assert!(!preview.already_purged);
    let applied = apply_record_privacy_purge(
        &store,
        "alpha",
        "mem_secret",
        "privacy cleanup",
        &explicit_patch_ids,
        &preview.fingerprint,
    )
    .unwrap();
    assert!(applied.mutation_performed);
    let canonical = fs::read_to_string(root.join("patches.jsonl")).unwrap();
    assert!(!canonical.contains("sk-prohibited-value"));
    let patches = store.load_patches().unwrap();
    let patch = patches
        .iter()
        .find(|patch| patch.id == "explicit_patch")
        .unwrap();
    assert_eq!(patch.reason, "[privacy purged]");
    assert!(patch.evidence.is_empty());
    let proposed = patch.proposed_record.as_ref().unwrap();
    assert_eq!(proposed.claim, "[privacy purged]");
    assert_eq!(proposed.status, RecordStatus::Tombstoned);
    let independent = patches
        .iter()
        .find(|patch| patch.id == "cap_secret")
        .unwrap();
    assert_eq!(independent.reason, "independent reason");
    assert_eq!(
        independent.proposed_record.as_ref().unwrap().claim,
        "independent safe claim"
    );
    assert!(applied.backup.as_ref().is_some_and(|backup| {
        fs::metadata(PathBuf::from(&backup.backup_dir).join("patches.jsonl")).is_ok()
    }));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn privacy_purge_rejects_stale_fingerprint_without_mutation() {
    let root = temp_store_dir();
    let store = JsonlStore::new(&root);
    store.init().unwrap();
    store
        .append_record(&secret_record(RecordStatus::Active))
        .unwrap();
    let preview =
        plan_record_privacy_purge(&store, "alpha", "mem_secret", "privacy cleanup", &[]).unwrap();
    store
        .append_record(&secret_record(RecordStatus::Active))
        .unwrap();
    let before = fs::read(root.join("records.jsonl")).unwrap();
    assert!(apply_record_privacy_purge(
        &store,
        "alpha",
        "mem_secret",
        "privacy cleanup",
        &[],
        &preview.fingerprint
    )
    .is_err());
    assert_eq!(fs::read(root.join("records.jsonl")).unwrap(), before);
    fs::remove_dir_all(root).unwrap();
}
