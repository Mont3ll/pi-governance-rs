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

#[test]
fn exports_and_imports_portable_bundle_without_overwriting_duplicates() -> anyhow::Result<()> {
    use pi_core::{EvidenceKind, EvidenceRef, Record, RecordClass, Scope};
    use pi_store::{StoreExportOptions, StoreImportOptions};

    let source_root = temp_store_dir("export-source");
    let target_root = temp_store_dir("export-target");

    let source = JsonlStore::new(&source_root);
    source.init()?;

    let record = Record::new(
        RecordClass::Requirement,
        "Portable exports must be importable into a different PI store.",
        0.82,
        Scope::project("pi-governance-rs"),
        vec!["portable".to_string()],
        vec![EvidenceRef::new(EvidenceKind::Test, "test:portable-export")],
    );

    source.append_record(&record)?;

    let bundle = source.export_bundle(StoreExportOptions {
        project: Some("pi-governance-rs".to_string()),
        redacted: false,
    })?;

    assert_eq!(bundle.records.len(), 1);
    assert_eq!(bundle.records[0].id, record.id);

    let target = JsonlStore::new(&target_root);
    target.init()?;

    let dry_run = target.import_bundle(
        bundle.clone(),
        StoreImportOptions {
            dry_run: true,
            backup: true,
        },
    )?;

    assert!(dry_run.changed);
    assert_eq!(dry_run.imported_records, 1);
    assert_eq!(target.load_records()?.len(), 0);

    let imported = target.import_bundle(
        bundle.clone(),
        StoreImportOptions {
            dry_run: false,
            backup: true,
        },
    )?;

    assert!(imported.changed);
    assert_eq!(imported.imported_records, 1);
    assert_eq!(target.load_records()?.len(), 1);

    let duplicate = target.import_bundle(
        bundle,
        StoreImportOptions {
            dry_run: false,
            backup: true,
        },
    )?;

    assert!(!duplicate.changed);
    assert_eq!(duplicate.imported_records, 0);
    assert_eq!(duplicate.skipped_records, 1);
    assert_eq!(target.load_records()?.len(), 1);

    fs::remove_dir_all(source_root)?;
    fs::remove_dir_all(target_root)?;
    Ok(())
}
