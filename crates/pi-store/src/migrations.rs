use anyhow::{Context, Result};
use pi_governance_core::CURRENT_SCHEMA_VERSION;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

use crate::backup::{create_store_backup, StoreBackupReport};
use crate::jsonl::JsonlStore;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaMigrationOptions {
    pub dry_run: bool,
    pub backup: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaMigrationFileReport {
    pub file_name: String,
    pub entries: usize,
    pub changed_entries: usize,
    pub root_schema_version_added: usize,
    pub root_schema_version_updated: usize,
    pub nested_schema_version_added: usize,
    pub nested_schema_version_updated: usize,
    pub invalid_json_lines: usize,
    pub rewritten: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaMigrationReport {
    pub schema_version: u32,
    pub dry_run: bool,
    pub backup_requested: bool,
    pub backup: Option<StoreBackupReport>,
    pub migration_needed: bool,
    pub changed_files: usize,
    pub changed_entries: usize,
    pub invalid_json_lines: usize,
    pub files: Vec<SchemaMigrationFileReport>,
}

impl JsonlStore {
    pub fn migrate_schema_versions(
        &self,
        options: SchemaMigrationOptions,
    ) -> Result<SchemaMigrationReport> {
        let _session = if options.dry_run { None } else { Some(self.write_session()?) };

        let mut files = Vec::new();
        let targets = [
            ("records.jsonl", self.records_path.as_path(), JsonlKind::Records),
            ("patches.jsonl", self.patches_path.as_path(), JsonlKind::Patches),
            ("events.jsonl", self.events_path.as_path(), JsonlKind::Events),
        ];

        for (file_name, path, kind) in targets {
            files.push(plan_file_migration(
                file_name,
                path,
                kind,
                CURRENT_SCHEMA_VERSION,
            )?);
        }

        let migration_needed = files.iter().any(|file| file.changed_entries > 0);
        let changed_files = files.iter().filter(|file| file.changed_entries > 0).count();
        let changed_entries = files.iter().map(|file| file.changed_entries).sum();
        let invalid_json_lines = files.iter().map(|file| file.invalid_json_lines).sum();

        let mut backup = None;

        if migration_needed && !options.dry_run {
            if options.backup {
                backup = Some(create_store_backup(
                    &self.root,
                    &[
                        ("records.jsonl", self.records_path.as_path()),
                        ("patches.jsonl", self.patches_path.as_path()),
                        ("events.jsonl", self.events_path.as_path()),
                    ],
                )?);
            }

            for (file_name, path, kind) in targets {
                rewrite_file_with_schema_version(
                    file_name,
                    path,
                    kind,
                    CURRENT_SCHEMA_VERSION,
                )?;
            }

            for file in &mut files {
                if file.changed_entries > 0 {
                    file.rewritten = true;
                }
            }
        }

        Ok(SchemaMigrationReport {
            schema_version: CURRENT_SCHEMA_VERSION,
            dry_run: options.dry_run,
            backup_requested: options.backup,
            backup,
            migration_needed,
            changed_files,
            changed_entries,
            invalid_json_lines,
            files,
        })
    }
}

#[derive(Debug, Clone, Copy)]
enum JsonlKind {
    Records,
    Patches,
    Events,
}

#[derive(Debug, Default)]
struct ChangeCounter {
    root_added: usize,
    root_updated: usize,
    nested_added: usize,
    nested_updated: usize,
}

impl ChangeCounter {
    fn changed(&self) -> bool {
        self.root_added > 0
            || self.root_updated > 0
            || self.nested_added > 0
            || self.nested_updated > 0
    }
}

fn plan_file_migration(
    file_name: impl Into<String>,
    path: &Path,
    kind: JsonlKind,
    schema_version: u32,
) -> Result<SchemaMigrationFileReport> {
    let file_name = file_name.into();

    if !path.exists() {
        return Ok(SchemaMigrationFileReport {
            file_name,
            entries: 0,
            changed_entries: 0,
            root_schema_version_added: 0,
            root_schema_version_updated: 0,
            nested_schema_version_added: 0,
            nested_schema_version_updated: 0,
            invalid_json_lines: 0,
            rewritten: false,
        });
    }

    let file = File::open(path).with_context(|| format!("failed to open {:?}", path))?;
    let reader = BufReader::new(file);

    let mut report = SchemaMigrationFileReport {
        file_name,
        entries: 0,
        changed_entries: 0,
        root_schema_version_added: 0,
        root_schema_version_updated: 0,
        nested_schema_version_added: 0,
        nested_schema_version_updated: 0,
        invalid_json_lines: 0,
        rewritten: false,
    };

    for line in reader.lines() {
        let line = line.with_context(|| format!("failed to read {:?}", path))?;

        if line.trim().is_empty() {
            continue;
        }

        report.entries += 1;

        let Ok(mut value) = serde_json::from_str::<Value>(&line) else {
            report.invalid_json_lines += 1;
            continue;
        };

        let mut counter = ChangeCounter::default();
        migrate_value(&mut value, kind, schema_version, &mut counter);

        if counter.changed() {
            report.changed_entries += 1;
            report.root_schema_version_added += counter.root_added;
            report.root_schema_version_updated += counter.root_updated;
            report.nested_schema_version_added += counter.nested_added;
            report.nested_schema_version_updated += counter.nested_updated;
        }
    }

    Ok(report)
}

fn rewrite_file_with_schema_version(
    file_name: &str,
    path: &Path,
    kind: JsonlKind,
    schema_version: u32,
) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }

    let file = File::open(path).with_context(|| format!("failed to open {:?}", path))?;
    let reader = BufReader::new(file);
    let mut output_lines = Vec::new();

    for line in reader.lines() {
        let line = line.with_context(|| format!("failed to read {:?}", path))?;

        if line.trim().is_empty() {
            output_lines.push(line);
            continue;
        }

        match serde_json::from_str::<Value>(&line) {
            Ok(mut value) => {
                let mut counter = ChangeCounter::default();
                migrate_value(&mut value, kind, schema_version, &mut counter);
                output_lines.push(serde_json::to_string(&value)?);
            }
            Err(_) => {
                // Preserve invalid lines byte-for-byte rather than losing user data.
                output_lines.push(line);
            }
        }
    }

    write_raw_lines_atomic(path, &output_lines)
        .with_context(|| format!("failed to rewrite {file_name} during schema migration"))
}

fn migrate_value(
    value: &mut Value,
    kind: JsonlKind,
    schema_version: u32,
    counter: &mut ChangeCounter,
) {
    ensure_schema_version(value, schema_version, &mut counter.root_added, &mut counter.root_updated);

    match kind {
        JsonlKind::Records => {
            migrate_evidence_array(value, "evidence", schema_version, counter);
        }
        JsonlKind::Patches => {
            migrate_evidence_array(value, "evidence", schema_version, counter);
            if let Some(proposed_record) = value.get_mut("proposed_record") {
                ensure_schema_version(
                    proposed_record,
                    schema_version,
                    &mut counter.nested_added,
                    &mut counter.nested_updated,
                );
                migrate_evidence_array(proposed_record, "evidence", schema_version, counter);
            }
        }
        JsonlKind::Events => {}
    }
}

fn migrate_evidence_array(
    value: &mut Value,
    key: &str,
    schema_version: u32,
    counter: &mut ChangeCounter,
) {
    let Some(object) = value.as_object_mut() else {
        return;
    };

    let Some(evidence) = object.get_mut(key).and_then(Value::as_array_mut) else {
        return;
    };

    for evidence_ref in evidence {
        ensure_schema_version(
            evidence_ref,
            schema_version,
            &mut counter.nested_added,
            &mut counter.nested_updated,
        );
    }
}

fn ensure_schema_version(
    value: &mut Value,
    schema_version: u32,
    added: &mut usize,
    updated: &mut usize,
) {
    let Some(object) = value.as_object_mut() else {
        return;
    };

    match object.get("schema_version").and_then(Value::as_u64) {
        None => {
            object.insert("schema_version".to_string(), Value::from(schema_version));
            *added += 1;
        }
        Some(existing) if existing != schema_version as u64 => {
            object.insert("schema_version".to_string(), Value::from(schema_version));
            *updated += 1;
        }
        Some(_) => {}
    }
}

fn write_raw_lines_atomic(path: &Path, lines: &[String]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create parent dir {:?}", parent))?;
    }

    let tmp_path = path.with_extension("jsonl.tmp");

    {
        let mut file = File::create(&tmp_path)
            .with_context(|| format!("failed to write temp file {:?}", tmp_path))?;

        for line in lines {
            writeln!(file, "{line}")?;
        }

        file.sync_all()?;
    }

    fs::rename(&tmp_path, path)
        .with_context(|| format!("failed to atomically replace {:?}", path))?;

    Ok(())
}
