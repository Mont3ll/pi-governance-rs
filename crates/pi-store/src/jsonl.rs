use anyhow::{Context, Result};
use pi_core::{Patch, Record, StoreEvent};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct JsonlStore {
    root: PathBuf,
    records_path: PathBuf,
    patches_path: PathBuf,
    events_path: PathBuf,
}

impl JsonlStore {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        let root = root.into();

        Self {
            records_path: root.join("records.jsonl"),
            patches_path: root.join("patches.jsonl"),
            events_path: root.join("events.jsonl"),
            root,
        }
    }

    pub fn root(&self) -> &Path {
        &self.root
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
        self.append_jsonl(&self.records_path, record)
    }

    pub fn append_patch(&self, patch: &Patch) -> Result<()> {
        self.append_jsonl(&self.patches_path, patch)
    }

    pub fn append_event(&self, event: &StoreEvent) -> Result<()> {
        self.append_jsonl(&self.events_path, event)
    }

    pub fn overwrite_records_atomic(&self, records: &[Record]) -> Result<()> {
        self.write_jsonl_atomic(&self.records_path, records)
    }

    fn read_jsonl<T: DeserializeOwned>(&self, path: &Path) -> Result<Vec<T>> {
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

    fn append_jsonl<T: Serialize>(&self, path: &Path, value: &T) -> Result<()> {
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

    fn write_jsonl_atomic<T: Serialize>(&self, path: &Path, values: &[T]) -> Result<()> {
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
}
