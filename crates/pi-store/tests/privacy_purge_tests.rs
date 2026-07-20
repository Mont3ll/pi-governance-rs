use chrono::Utc;
use pi_governance_core::{EvidenceKind, EvidenceRef, Record, RecordClass, RecordStatus, Scope};
use pi_governance_store::{apply_record_privacy_purge, plan_record_privacy_purge, JsonlStore};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_store_dir() -> PathBuf {
    let nonce = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    std::env::temp_dir().join(format!("pi-privacy-purge-{}-{nonce}", std::process::id()))
}

fn secret_record(status: RecordStatus) -> Record {
    let mut record = Record::new(
        RecordClass::Workflow,
        "legacy secret sk-prohibited-value",
        0.9,
        Scope::project("secret-project-token"),
        vec!["secret-tag".into()],
        vec![EvidenceRef::new(EvidenceKind::Conversation, "secret:evidence")],
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
    store.append_record(&secret_record(RecordStatus::Superseded)).unwrap();
    let before = fs::read(root.join("records.jsonl")).unwrap();

    let preview = plan_record_privacy_purge(&store, "alpha", "mem_secret", "privacy cleanup").unwrap();
    assert!(preview.dry_run);
    assert!(!preview.mutation_performed);
    assert!(preview.target_found);
    assert_eq!(fs::read(root.join("records.jsonl")).unwrap(), before);

    let applied = apply_record_privacy_purge(&store, "alpha", "mem_secret", "privacy cleanup", &preview.fingerprint).unwrap();
    assert!(applied.mutation_performed);
    assert!(applied.backup.as_ref().is_some_and(|backup| fs::metadata(&backup.backup_dir).is_ok()));
    assert!(applied.report_path.as_ref().is_some_and(|path| fs::metadata(path).is_ok()));
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

    let second = plan_record_privacy_purge(&store, "alpha", "mem_secret", "privacy cleanup").unwrap();
    assert!(second.already_purged);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn privacy_purge_rejects_stale_fingerprint_without_mutation() {
    let root = temp_store_dir();
    let store = JsonlStore::new(&root);
    store.init().unwrap();
    store.append_record(&secret_record(RecordStatus::Active)).unwrap();
    let preview = plan_record_privacy_purge(&store, "alpha", "mem_secret", "privacy cleanup").unwrap();
    store.append_record(&secret_record(RecordStatus::Active)).unwrap();
    let before = fs::read(root.join("records.jsonl")).unwrap();
    assert!(apply_record_privacy_purge(&store, "alpha", "mem_secret", "privacy cleanup", &preview.fingerprint).is_err());
    assert_eq!(fs::read(root.join("records.jsonl")).unwrap(), before);
    fs::remove_dir_all(root).unwrap();
}
