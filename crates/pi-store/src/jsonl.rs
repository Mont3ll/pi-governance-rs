use anyhow::{Context, Result};
use pi_governance_core::{Patch, Record, SchemaFileAudit, StoreEvent};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use crate::lock::{StoreLockConfig, StoreLockGuard};

#[derive(Debug, Clone)]
pub struct JsonlStore {
    pub(crate) root: PathBuf,
    pub(crate) records_path: PathBuf,
    pub(crate) patches_path: PathBuf,
    pub(crate) events_path: PathBuf,
    pub(crate) lock_path: PathBuf,
}

#[derive(Debug)]
pub struct JsonlStoreWriteSession<'a> {
    store: &'a JsonlStore,
    _guard: StoreLockGuard,
}

impl JsonlStore {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        let root = root.into();

        Self {
            records_path: root.join("records.jsonl"),
            patches_path: root.join("patches.jsonl"),
            events_path: root.join("events.jsonl"),
            lock_path: root.join("store.lock"),
            root,
        }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn lock_path(&self) -> &Path {
        &self.lock_path
    }

    pub fn init(&self) -> Result<()> {
        fs::create_dir_all(&self.root)
            .with_context(|| format!("failed to create store dir {:?}", self.root))?;

        for path in [&self.records_path, &self.patches_path, &self.events_path] {
            if !path.exists() {
                File::create(path).with_context(|| format!("failed to create {:?}", path))?;
            }
        }

        Ok(())
    }

    pub fn write_session(&self) -> Result<JsonlStoreWriteSession<'_>> {
        self.init()?;
        let guard = StoreLockGuard::acquire(&self.lock_path, StoreLockConfig::default())?;

        Ok(JsonlStoreWriteSession {
            store: self,
            _guard: guard,
        })
    }

    pub fn load_records(&self) -> Result<Vec<Record>> {
        self.read_jsonl(&self.records_path)
    }

    pub fn load_patches(&self) -> Result<Vec<Patch>> {
        self.read_jsonl(&self.patches_path)
    }

    pub fn load_events(&self) -> Result<Vec<StoreEvent>> {
        self.read_jsonl(&self.events_path)
    }

    pub fn append_record(&self, record: &Record) -> Result<()> {
        let session = self.write_session()?;
        session.append_record(record)
    }

    pub fn append_patch(&self, patch: &Patch) -> Result<()> {
        let session = self.write_session()?;
        session.append_patch(patch)
    }

    pub fn append_event(&self, event: &StoreEvent) -> Result<()> {
        let session = self.write_session()?;
        session.append_event(event)
    }

    pub fn overwrite_records_atomic(&self, records: &[Record]) -> Result<()> {
        let session = self.write_session()?;
        session.overwrite_records_atomic(records)
    }

    pub fn audit_schema_versions(&self, current_schema_version: u32) -> Result<Vec<SchemaFileAudit>> {
        Ok(vec![
            self.audit_jsonl_schema("records.jsonl", &self.records_path, current_schema_version)?,
            self.audit_jsonl_schema("patches.jsonl", &self.patches_path, current_schema_version)?,
            self.audit_jsonl_schema("events.jsonl", &self.events_path, current_schema_version)?,
        ])
    }

    fn read_jsonl<T: DeserializeOwned>(&self, path: &Path) -> Result<Vec<T>> {
        read_jsonl(path)
    }

    fn append_jsonl<T: Serialize>(&self, path: &Path, value: &T) -> Result<()> {
        append_jsonl(path, value)
    }

    fn write_jsonl_atomic<T: Serialize>(&self, path: &Path, values: &[T]) -> Result<()> {
        write_jsonl_atomic(path, values)
    }

    fn audit_jsonl_schema(
        &self,
        file_name: impl Into<String>,
        path: &Path,
        current_schema_version: u32,
    ) -> Result<SchemaFileAudit> {
        let mut audit = SchemaFileAudit::clean(file_name);

        if !path.exists() {
            return Ok(audit);
        }

        let file = File::open(path).with_context(|| format!("failed to open {:?}", path))?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line.with_context(|| format!("failed to read {:?}", path))?;

            if line.trim().is_empty() {
                continue;
            }

            audit.entries += 1;

            let Ok(value) = serde_json::from_str::<Value>(&line) else {
                audit.invalid_json_lines += 1;
                continue;
            };

            match value.get("schema_version").and_then(Value::as_u64) {
                None => audit.missing_schema_version += 1,
                Some(version) if version != current_schema_version as u64 => {
                    audit.mismatched_schema_version += 1;
                }
                Some(_) => {}
            }
        }

        Ok(audit)
    }
}

impl<'a> JsonlStoreWriteSession<'a> {
    pub fn load_records(&self) -> Result<Vec<Record>> {
        self.store.load_records()
    }

    pub fn load_patches(&self) -> Result<Vec<Patch>> {
        self.store.load_patches()
    }

    pub fn load_events(&self) -> Result<Vec<StoreEvent>> {
        self.store.load_events()
    }

    pub fn append_record(&self, record: &Record) -> Result<()> {
        self.store.append_jsonl(&self.store.records_path, record)
    }

    pub fn append_patch(&self, patch: &Patch) -> Result<()> {
        self.store.append_jsonl(&self.store.patches_path, patch)
    }

    pub fn append_event(&self, event: &StoreEvent) -> Result<()> {
        self.store.append_jsonl(&self.store.events_path, event)
    }

    pub fn overwrite_records_atomic(&self, records: &[Record]) -> Result<()> {
        self.store
            .write_jsonl_atomic(&self.store.records_path, records)
    }

    pub fn overwrite_patches_atomic(&self, patches: &[Patch]) -> Result<()> {
        self.store
            .write_jsonl_atomic(&self.store.patches_path, patches)
    }

    pub fn overwrite_events_atomic(&self, events: &[StoreEvent]) -> Result<()> {
        self.store
            .write_jsonl_atomic(&self.store.events_path, events)
    }
}

fn read_jsonl<T: DeserializeOwned>(path: &Path) -> Result<Vec<T>> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let file = File::open(path).with_context(|| format!("failed to open {:?}", path))?;
    let reader = BufReader::new(file);
    let mut output = Vec::new();

    for line in reader.lines() {
        let line = line.with_context(|| format!("failed to read {:?}", path))?;

        if line.trim().is_empty() {
            continue;
        }

        let value = serde_json::from_str::<T>(&line)
            .with_context(|| format!("invalid JSONL entry in {:?}", path))?;

        output.push(value);
    }

    Ok(output)
}

fn append_jsonl<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create parent dir {:?}", parent))?;
    }

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .with_context(|| format!("failed to append to {:?}", path))?;

    let line = serde_json::to_string(value)?;
    writeln!(file, "{line}")?;
    file.sync_all()?;

    Ok(())
}

fn write_jsonl_atomic<T: Serialize>(path: &Path, values: &[T]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create parent dir {:?}", parent))?;
    }

    let tmp_path = path.with_extension("jsonl.tmp");

    {
        let mut file = File::create(&tmp_path)
            .with_context(|| format!("failed to write temp file {:?}", tmp_path))?;

        for value in values {
            let line = serde_json::to_string(value)?;
            writeln!(file, "{line}")?;
        }

        file.sync_all()?;
    }

    fs::rename(&tmp_path, path)
        .with_context(|| format!("failed to atomically replace {:?}", path))?;

    Ok(())
}
