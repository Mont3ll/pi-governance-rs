use pi_governance_core::{RecallEvent, RecallEventClient, RecallEventOperation, CURRENT_SCHEMA_VERSION};
use pi_governance_store::{JsonlStore, SchemaMigrationOptions};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn recall_telemetry_is_disabled_by_default_and_uses_a_separate_jsonl_stream() {
    let root = temp_store_dir("recall-telemetry");
    let store = JsonlStore::new(&root);
    store.init().unwrap();
    assert!(!store.load_config().unwrap().recall_telemetry.enabled);
    assert!(store.load_recall_events().unwrap().is_empty());

    let event = RecallEvent::new("default", RecallEventClient::Cli, RecallEventOperation::Retrieve, "query-hash", vec!["rec_1".into()], 1200, 80);
    assert!(!store.record_recall_event(&event).unwrap());
    assert!(store.load_recall_events().unwrap().is_empty());

    let mut config = store.load_config().unwrap();
    config.recall_telemetry.enabled = true;
    config.recall_telemetry.max_events = 1;
    store.save_config(&config).unwrap();
    assert!(store.record_recall_event(&event).unwrap());
    let second = RecallEvent::new("default", RecallEventClient::Cli, RecallEventOperation::RecallXray, "second-hash", vec!["rec_2".into()], 1200, 90);
    assert!(store.record_recall_event(&second).unwrap());
    let events = store.load_recall_events().unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].query_hash, "second-hash");
    assert!(root.join("recall-events.jsonl").exists());
    let export = store.export_bundle(pi_governance_store::StoreExportOptions { namespace: Some("default".into()), all_namespaces: false, project: None, redacted: false }).unwrap();
    assert!(!serde_json::to_string(&export).unwrap().contains("second-hash"));
}

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
fn imports_javascript_bundle_and_preserves_auxiliary_sections() -> anyhow::Result<()> {
    use pi_governance_store::{StoreExportOptions, StoreImportOptions};
    let root = temp_store_dir("js-portable");
    let store = JsonlStore::new(&root);
    store.init()?;
    let path = root.join("js-bundle.json");
    let bundle = serde_json::json!({
        "schema_version": 1, "format": "pi-governance",
        "producer": {"name":"pi-persistent-intelligence","version":"0.13.0"},
        "exported_at":"2026-07-15T00:00:00Z", "redacted":false,
        "redaction":{"enabled":false,"fields_checked":[],"fields_redacted":[],"notes":[]},
        "namespace":"interop-test", "all_namespaces":false, "project":null,
        "records":[
            {"id":"mem_js","namespace":"interop-test","class":"workflow","layer":"l2_playbook","claim":"Preserve portable memory.","status":"active","memory_kind":"instruction","rule_type":"workflow","trust_class":"direct_user_instruction","durability":"project","source_kind":"manual_cli","confidence":0.9,"evidence_ids":["ev_js"],"evidence":[{"kind":"conversation","uri":"daily/2026-07-15.md"}],"scope":{"level":"project","key":"demo"},"tags":["interop"],"supersedes":[],"created_at":"2026-07-15","updated_at":"2026-07-15"},
            {"id":"mem_domain","namespace":"interop-test","class":"workflow","layer":"l2_playbook","claim":"Preserve domain scope metadata.","status":"active","memory_kind":"instruction","confidence":0.8,"evidence":[],"scope":{"level":"domain","key":"release"},"tags":["interop"],"supersedes":[],"created_at":"2026-07-15","updated_at":"2026-07-15"}
        ],
        "patches":[
            {"id":"cap_js","status":"proposed","operation":"propose_record","claim":"Review imported candidate.","layer":"l2_playbook","memory_kind":"instruction","rule_type":"workflow","tags":["interop"]},
            {"schema_version":1,"namespace":"interop-test","id":"patch_reinforce","operation":"reinforce_record","status":"applied","target_id":"mem_js","proposed_record":null,"contest_resolution":null,"evidence":[],"reason":"Historical reinforcement","created_at":"2026-07-15T00:00:00Z","updated_at":"2026-07-15T00:00:00Z"}
        ],
        "evidence":[{"id":"ev_js","created_at":"2026-07-15T00:00:00Z","source_summary":"Portable evidence"}],
        "sessions":[{"id":"session_js","namespace":"interop-test","layer":"l3_session","text":"#decision preserve sessions","created_at":"2026-07-15T00:00:00Z","source_kind":"session_entry"}],
        "inquiries":[{"id":"inq_js","created_at":"2026-07-15T00:00:00Z","question":"Preserve inquiry?","status":"open"}],
        "reinforcement":[{"id":"rein_js","memory_id":"mem_js","timestamp":"2026-07-15T00:00:00Z","outcome":"explicit_reinforcement"}],
        "tombstones":[{"id":"tomb_js","deleted_record_id":"mem_deleted","deleted_at":"2026-07-15T00:00:00Z","deletion_mode":"audit_preserving","deletion_reason":"user_requested","content_removed":true}],
        "events":[]
    });
    fs::write(&path, serde_json::to_vec_pretty(&bundle)?)?;
    let report = store.import_bundle_from_path(&path, StoreImportOptions { namespace:"interop-test".into(), preserve_namespaces:true, dry_run:false, backup:true })?;
    assert_eq!(report.imported_records, 2);
    assert!(store.load_records()?.iter().any(|record| record.id == "mem_domain" && record.tags.iter().any(|tag| tag == "domain:release")));
    assert_eq!(report.imported_patches, 2);
    assert_eq!(report.imported_events, 5);
    let exported = store.export_bundle(StoreExportOptions { namespace:Some("interop-test".into()), all_namespaces:false, project:None, redacted:false })?;
    assert_eq!(exported.evidence.len(), 1);
    assert_eq!(exported.sessions.len(), 1);
    assert_eq!(exported.inquiries.len(), 1);
    assert_eq!(exported.reinforcement.len(), 1);
    assert_eq!(exported.tombstones.len(), 1);
    let redacted = store.export_bundle(StoreExportOptions { namespace:Some("interop-test".into()), all_namespaces:false, project:None, redacted:true })?;
    assert!(redacted.sessions.is_empty());
    assert_eq!(redacted.evidence[0].get("source_summary").and_then(serde_json::Value::as_str), Some("redacted"));
    let duplicate = store.import_bundle_from_path(&path, StoreImportOptions { namespace:"interop-test".into(), preserve_namespaces:true, dry_run:false, backup:true })?;
    assert_eq!(duplicate.imported_records, 0);
    assert_eq!(duplicate.imported_patches, 0);
    assert_eq!(duplicate.imported_events, 0);
    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn exports_and_imports_portable_bundle_without_overwriting_duplicates() -> anyhow::Result<()> {
    use pi_governance_core::{EvidenceKind, EvidenceRef, Record, RecordClass, Scope};
    use pi_governance_store::{StoreExportOptions, StoreImportOptions};

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
        namespace: Some("default".to_string()),
        all_namespaces: false,
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
            namespace: "default".to_string(),
            preserve_namespaces: false,
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
            namespace: "default".to_string(),
            preserve_namespaces: false,
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
            namespace: "default".to_string(),
            preserve_namespaces: false,
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
