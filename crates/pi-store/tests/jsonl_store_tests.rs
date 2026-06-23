use pi_core::CURRENT_SCHEMA_VERSION;
use pi_store::{JsonlStore, SchemaMigrationOptions};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_store_dir(test_name: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be after unix epoch")
        .as_nanos();

    std::env::temp_dir().join(format!(
        "pi-store-{test_name}-{}-{nonce}",
        std::process::id()
    ))
}

#[test]
fn migrates_legacy_jsonl_schema_versions_with_backup() -> anyhow::Result<()> {
    let root = temp_store_dir("migration");
    fs::create_dir_all(&root)?;

    fs::write(
        root.join("records.jsonl"),
        r#"{"id":"rec_legacy","class":"preference","claim":"Legacy record should be migrated safely.","evidence":[{"kind":"Conversation","uri":"conversation:legacy","note":null}],"confidence":0.8,"status":"active","scope":{"level":"global","key":null},"tags":[],"supersedes":[],"created_at":"2026-01-01T00:00:00Z","updated_at":"2026-01-01T00:00:00Z"}
"#,
    )?;
    fs::write(
        root.join("patches.jsonl"),
        r#"{"id":"patch_legacy","operation":"propose_record","status":"proposed","target_id":null,"proposed_record":null,"evidence":[{"kind":"Conversation","uri":"conversation:legacy","note":null}],"reason":"legacy fixture","created_at":"2026-01-01T00:00:00Z","updated_at":"2026-01-01T00:00:00Z"}
"#,
    )?;
    fs::write(
        root.join("events.jsonl"),
        r#"{"id":"evt_legacy","severity":"info","message":"legacy fixture","object_id":null,"created_at":"2026-01-01T00:00:00Z"}
"#,
    )?;

    let store = JsonlStore::new(&root);

    let dry_run = store.migrate_schema_versions(SchemaMigrationOptions {
        dry_run: true,
        backup: true,
    })?;

    assert!(dry_run.migration_needed);
    assert_eq!(dry_run.changed_files, 3);
    assert!(dry_run.backup.is_none());
    assert!(!fs::read_to_string(root.join("records.jsonl"))?.contains("schema_version"));

    let migrated = store.migrate_schema_versions(SchemaMigrationOptions {
        dry_run: false,
        backup: true,
    })?;

    assert!(migrated.migration_needed);
    assert_eq!(migrated.changed_files, 3);
    assert!(migrated.backup.is_some());
    assert!(fs::read_to_string(root.join("records.jsonl"))?.contains("schema_version"));

    let audits = store.audit_schema_versions(CURRENT_SCHEMA_VERSION)?;
    assert!(audits.iter().all(|audit| audit.missing_schema_version == 0));
    assert!(audits.iter().all(|audit| audit.mismatched_schema_version == 0));

    fs::remove_dir_all(root)?;
    Ok(())
}
