use anyhow::{bail, Context, Result};
use chrono::{DateTime, Utc};
use pi_core::{Patch, Record, StoreEvent, CURRENT_SCHEMA_VERSION};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

use crate::backup::{create_store_backup, StoreBackupReport};
use crate::jsonl::JsonlStore;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreExportOptions {
    pub namespace: Option<String>,
    pub all_namespaces: bool,
    pub project: Option<String>,
    pub redacted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreImportOptions {
    pub namespace: String,
    pub preserve_namespaces: bool,
    pub dry_run: bool,
    pub backup: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreExportBundle {
    pub schema_version: u32,
    pub exported_at: DateTime<Utc>,
    pub redacted: bool,
    pub namespace: Option<String>,
    pub all_namespaces: bool,
    pub project: Option<String>,
    pub records: Vec<Record>,
    pub patches: Vec<Patch>,
    pub events: Vec<StoreEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreImportReport {
    pub schema_version: u32,
    pub dry_run: bool,
    pub backup_requested: bool,
    pub backup: Option<StoreBackupReport>,
    pub records_in_bundle: usize,
    pub patches_in_bundle: usize,
    pub events_in_bundle: usize,
    pub imported_records: usize,
    pub imported_patches: usize,
    pub imported_events: usize,
    pub skipped_records: usize,
    pub skipped_patches: usize,
    pub skipped_events: usize,
    pub changed: bool,
    pub warnings: Vec<String>,
}

impl JsonlStore {
    pub fn export_bundle(&self, options: StoreExportOptions) -> Result<StoreExportBundle> {
        self.init()?;

        let records = self.load_records()?;
        let patches = self.load_patches()?;
        let events = self.load_events()?;

        let namespace_filter = if options.all_namespaces { None } else { options.namespace.as_deref() };
        let mut selected_records: Vec<Record> = records
            .into_iter()
            .filter(|record| namespace_filter.map(|namespace| record.namespace == namespace).unwrap_or(true))
            .filter(|record| options.project.as_deref().map(|project| record.scope.matches_project_filter(Some(project))).unwrap_or(true))
            .collect();

        let selected_record_ids: HashSet<String> = selected_records
            .iter()
            .map(|record| record.id.clone())
            .collect();

        let mut selected_patches: Vec<Patch> = patches
            .into_iter()
            .filter(|patch| namespace_filter.map(|namespace| patch.namespace == namespace).unwrap_or(true))
            .filter(|patch| {
                patch
                    .target_id
                    .as_ref()
                    .map(|target_id| selected_record_ids.contains(target_id))
                    .unwrap_or(false)
                    || patch
                        .proposed_record
                        .as_ref()
                        .map(|record| {
                            selected_record_ids.contains(&record.id)
                                || options.project.as_deref().map(|project| record.scope.matches_project_filter(Some(project))).unwrap_or(true)
                        })
                        .unwrap_or(false)
                    || options.project.is_none()
            })
            .collect();

        let selected_patch_ids: HashSet<String> = selected_patches
            .iter()
            .map(|patch| patch.id.clone())
            .collect();

        let mut selected_events: Vec<StoreEvent> = events
            .into_iter()
            .filter(|event| namespace_filter.map(|namespace| event.namespace == namespace).unwrap_or(true))
            .filter(|event| {
                options.project.is_none()
                    || event
                        .object_id
                        .as_ref()
                        .map(|object_id| {
                            selected_record_ids.contains(object_id)
                                || selected_patch_ids.contains(object_id)
                        })
                        .unwrap_or(false)
            })
            .collect();

        if options.redacted {
            redact_records(&mut selected_records);
            redact_patches(&mut selected_patches);
            redact_events(&mut selected_events);
        }

        Ok(StoreExportBundle {
            schema_version: CURRENT_SCHEMA_VERSION,
            exported_at: Utc::now(),
            redacted: options.redacted,
            namespace: options.namespace,
            all_namespaces: options.all_namespaces,
            project: options.project,
            records: selected_records,
            patches: selected_patches,
            events: selected_events,
        })
    }

    pub fn export_bundle_to_path(
        &self,
        path: &Path,
        options: StoreExportOptions,
    ) -> Result<StoreExportBundle> {
        let bundle = self.export_bundle(options)?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create export parent dir {:?}", parent))?;
        }

        let json = serde_json::to_string_pretty(&bundle)?;
        fs::write(path, json).with_context(|| format!("failed to write export file {:?}", path))?;

        Ok(bundle)
    }

    pub fn import_bundle_from_path(
        &self,
        path: &Path,
        options: StoreImportOptions,
    ) -> Result<StoreImportReport> {
        let contents = fs::read_to_string(path)
            .with_context(|| format!("failed to read import bundle {:?}", path))?;
        let bundle: StoreExportBundle = serde_json::from_str(&contents)
            .with_context(|| format!("failed to parse import bundle {:?}", path))?;

        self.import_bundle(bundle, options)
    }

    pub fn import_bundle(
        &self,
        bundle: StoreExportBundle,
        options: StoreImportOptions,
    ) -> Result<StoreImportReport> {
        self.init()?;
        let session = self.write_session()?;

        if bundle.schema_version > CURRENT_SCHEMA_VERSION {
            bail!(
                "unsupported_export_schema: bundle schema_version {} is newer than current schema_version {}",
                bundle.schema_version,
                CURRENT_SCHEMA_VERSION
            );
        }

        let mut warnings = Vec::new();

        if bundle.redacted {
            warnings.push("imported bundle is redacted; evidence URIs and event messages may be placeholders".to_string());
        }

        if bundle.schema_version != CURRENT_SCHEMA_VERSION {
            warnings.push(format!(
                "bundle schema_version {} differs from current schema_version {}; run pi migrate after import if needed",
                bundle.schema_version, CURRENT_SCHEMA_VERSION
            ));
        }

        let mut records = session.load_records()?;
        let mut patches = session.load_patches()?;
        let mut events = session.load_events()?;

        let existing_record_ids: HashSet<String> = records.iter().map(|record| record.id.clone()).collect();
        let existing_patch_ids: HashSet<String> = patches.iter().map(|patch| patch.id.clone()).collect();
        let existing_event_ids: HashSet<String> = events.iter().map(|event| event.id.clone()).collect();

        let import_records: Vec<Record> = bundle
            .records
            .iter()
            .filter(|record| !existing_record_ids.contains(&record.id))
            .cloned()
            .map(|mut record| {
                if !options.preserve_namespaces {
                    record.namespace = options.namespace.clone();
                }
                record
            })
            .collect();
        let import_patches: Vec<Patch> = bundle
            .patches
            .iter()
            .filter(|patch| !existing_patch_ids.contains(&patch.id))
            .cloned()
            .map(|mut patch| {
                if !options.preserve_namespaces {
                    patch.namespace = options.namespace.clone();
                    if let Some(record) = &mut patch.proposed_record {
                        record.namespace = options.namespace.clone();
                    }
                }
                patch
            })
            .collect();
        let import_events: Vec<StoreEvent> = bundle
            .events
            .iter()
            .filter(|event| !existing_event_ids.contains(&event.id))
            .cloned()
            .map(|mut event| {
                if !options.preserve_namespaces {
                    event.namespace = options.namespace.clone();
                }
                event
            })
            .collect();

        let skipped_records = bundle.records.len().saturating_sub(import_records.len());
        let skipped_patches = bundle.patches.len().saturating_sub(import_patches.len());
        let skipped_events = bundle.events.len().saturating_sub(import_events.len());
        let changed = !import_records.is_empty() || !import_patches.is_empty() || !import_events.is_empty();

        let mut backup = None;

        if changed && !options.dry_run {
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

            records.extend(import_records.clone());
            patches.extend(import_patches.clone());
            events.extend(import_events.clone());

            session.overwrite_records_atomic(&records)?;
            session.overwrite_patches_atomic(&patches)?;
            session.overwrite_events_atomic(&events)?;
        }

        Ok(StoreImportReport {
            schema_version: CURRENT_SCHEMA_VERSION,
            dry_run: options.dry_run,
            backup_requested: options.backup,
            backup,
            records_in_bundle: bundle.records.len(),
            patches_in_bundle: bundle.patches.len(),
            events_in_bundle: bundle.events.len(),
            imported_records: import_records.len(),
            imported_patches: import_patches.len(),
            imported_events: import_events.len(),
            skipped_records,
            skipped_patches,
            skipped_events,
            changed,
            warnings,
        })
    }
}

fn redact_records(records: &mut [Record]) {
    for record in records {
        for evidence in &mut record.evidence {
            evidence.uri = "redacted:evidence".to_string();
            evidence.note = evidence.note.as_ref().map(|_| "redacted".to_string());
        }
    }
}

fn redact_patches(patches: &mut [Patch]) {
    for patch in patches {
        for evidence in &mut patch.evidence {
            evidence.uri = "redacted:evidence".to_string();
            evidence.note = evidence.note.as_ref().map(|_| "redacted".to_string());
        }

        if let Some(record) = patch.proposed_record.as_mut() {
            redact_records(std::slice::from_mut(record));
        }
    }
}

fn redact_events(events: &mut [StoreEvent]) {
    for event in events {
        event.message = "redacted event message".to_string();
    }
}
