use anyhow::{bail, Context, Result};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::thread::sleep;
use std::time::{Duration, Instant, SystemTime};

#[derive(Debug, Clone)]
pub struct StoreLockConfig {
    pub timeout: Duration,
    pub retry_delay: Duration,
    pub stale_after: Duration,
}

impl Default for StoreLockConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(10),
            retry_delay: Duration::from_millis(50),
            stale_after: Duration::from_secs(120),
        }
    }
}

#[derive(Debug)]
pub struct StoreLockGuard {
    path: PathBuf,
}

impl StoreLockGuard {
    pub fn acquire(path: impl AsRef<Path>, config: StoreLockConfig) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let parent = path
            .parent()
            .context("store lock path has no parent directory")?;

        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create lock parent directory {:?}", parent))?;

        let started_at = Instant::now();

        loop {
            match OpenOptions::new().write(true).create_new(true).open(&path) {
                Ok(mut file) => {
                    writeln!(file, "pid={}", std::process::id())?;
                    writeln!(file, "acquired_at={:?}", SystemTime::now())?;
                    file.sync_all()?;
                    return Ok(Self { path });
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                    maybe_remove_stale_lock(&path, config.stale_after)?;

                    if started_at.elapsed() >= config.timeout {
                        bail!(
                            "store_locked: timed out waiting for PI store lock {:?}; another process may be writing",
                            path
                        );
                    }

                    sleep(config.retry_delay);
                }
                Err(error) => {
                    return Err(error)
                        .with_context(|| format!("failed to acquire store lock {:?}", path));
                }
            }
        }
    }
}

impl Drop for StoreLockGuard {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

fn maybe_remove_stale_lock(path: &Path, stale_after: Duration) -> Result<()> {
    let Ok(metadata) = fs::metadata(path) else {
        return Ok(());
    };

    let Ok(modified_at) = metadata.modified() else {
        return Ok(());
    };

    let Ok(age) = modified_at.elapsed() else {
        return Ok(());
    };

    if age > stale_after {
        fs::remove_file(path)
            .with_context(|| format!("failed to remove stale PI store lock {:?}", path))?;
    }

    Ok(())
}
