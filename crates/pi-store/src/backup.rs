use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreBackupReport {
    pub backup_dir: String,
    pub copied_files: Vec<String>,
}

pub(crate) fn create_store_backup(
    root: &Path,
    files: &[(&str, &Path)],
) -> Result<StoreBackupReport> {
    create_store_backup_with_label(root, "schema-v1", files)
}

pub(crate) fn create_store_backup_with_label(
    root: &Path,
    label: &str,
    files: &[(&str, &Path)],
) -> Result<StoreBackupReport> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system clock is before UNIX_EPOCH")?
        .as_nanos();

    let backup_dir = root
        .join("backups")
        .join(format!("{label}-{timestamp}-pid{}", std::process::id()));

    fs::create_dir_all(root.join("backups"))
        .with_context(|| format!("failed to create backup root for {:?}", backup_dir))?;
    fs::create_dir(&backup_dir)
        .with_context(|| format!("failed to create unique backup directory {:?}", backup_dir))?;

    let mut copied_files = Vec::new();

    for (file_name, source_path) in files {
        if !source_path.exists() {
            continue;
        }

        let target_path: PathBuf = backup_dir.join(file_name);
        fs::copy(source_path, &target_path).with_context(|| {
            format!(
                "failed to copy backup file {:?} to {:?}",
                source_path, target_path
            )
        })?;
        copied_files.push(file_name.to_string());
    }

    Ok(StoreBackupReport {
        backup_dir: backup_dir.display().to_string(),
        copied_files,
    })
}
