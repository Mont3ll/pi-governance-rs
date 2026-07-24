use anyhow::{Context, Result};
use pi_governance_core::PiConfig;
use std::fs;

use crate::jsonl::JsonlStore;

impl JsonlStore {
    pub fn config_path(&self) -> std::path::PathBuf {
        self.root().join("config.json")
    }

    pub fn load_config(&self) -> Result<PiConfig> {
        self.init()?;
        let path = self.config_path();
        if !path.exists() {
            return Ok(PiConfig::default());
        }
        let contents =
            fs::read_to_string(&path).with_context(|| format!("failed to read {:?}", path))?;
        serde_json::from_str(&contents).with_context(|| format!("failed to parse {:?}", path))
    }

    pub fn save_config(&self, config: &PiConfig) -> Result<()> {
        self.init()?;
        let path = self.config_path();
        let tmp = path.with_extension("json.tmp");
        fs::write(&tmp, serde_json::to_string_pretty(config)?)?;
        fs::rename(&tmp, &path).with_context(|| format!("failed to write {:?}", path))
    }
}
