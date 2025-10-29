use crate::models::ContributionCalendar;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, env, fs, path::PathBuf};

#[derive(Serialize, Deserialize)]
pub struct CachedEntry {
    pub calendar: ContributionCalendar,
    pub fetched_at: DateTime<Utc>,
}

#[derive(Default, Serialize, Deserialize)]
struct CacheData {
    pub entries: HashMap<String, CachedEntry>,
}

pub struct Cache {
    data: CacheData,
}

impl Cache {
    fn path() -> Result<PathBuf> {
        let home = env::var("HOME").context("HOME environment variable not set")?;

        Ok(PathBuf::from(home).join(".trexanh").join("cache.json"))
    }

    pub fn load() -> Result<Self> {
        let path = Self::path()?;

        if !path.exists() {
            return Ok(Self {
                data: CacheData::default(),
            });
        }

        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read cache at {}", path.display()))?;

        let data: CacheData = serde_json::from_str(&content)
            .with_context(|| format!("Invalid JSON in {}", path.display()))?;

        Ok(Self { data })
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory {}", parent.display()))?;
        }

        let json = serde_json::to_string_pretty(&self.data)?;
        fs::write(&path, json)
            .with_context(|| format!("Failed to write cache to {}", path.display()))?;

        Ok(())
    }

    pub fn get(&self, username: &str) -> Option<ContributionCalendar> {
        self.data
            .entries
            .get(username)
            .map(|entry| entry.calendar.clone())
    }

    pub fn insert(&mut self, username: String, calendar: ContributionCalendar) {
        self.data.entries.insert(
            username,
            CachedEntry {
                calendar,
                fetched_at: Utc::now(),
            },
        );
    }
}
