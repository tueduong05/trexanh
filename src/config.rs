use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{env, fs, path::PathBuf};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    pub username: String,
    pub token: String,
}

impl Config {
    fn path() -> Result<PathBuf> {
        let home = env::var("HOME").context("HOME environment variable not set")?;

        Ok(PathBuf::from(home).join(".trexanh").join("config.json"))
    }

    pub fn exists() -> bool {
        Self::path().map(|p| p.exists()).unwrap_or(false)
    }

    pub fn load() -> Result<Self> {
        let path = Self::path()?;
        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config at {}", path.display()))?;
        let config = serde_json::from_str(&content)
            .with_context(|| format!("Invalid JSON in {}", path.display()))?;

        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory {}", parent.display()))?;
        }

        let json = serde_json::to_string_pretty(&self)?;
        fs::write(&path, json)
            .with_context(|| format!("Failed to write config to {}", path.display()))?;

        Ok(())
    }
}
