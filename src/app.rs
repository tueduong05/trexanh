use crate::api::fetch_contributions;
use crate::config::Config;
use crate::models::ContributionCalendar;
use anyhow::Result;
use std::time::{Duration, Instant};

#[derive(PartialEq)]
pub enum AppMode {
    Single,
    Watch,
}

pub struct App {
    pub config: Config,
    pub mode: AppMode,
    pub calendar: Option<ContributionCalendar>,
    pub last_update: Option<Instant>,
}

impl App {
    pub fn new(config: Config, mode: AppMode) -> Self {
        Self {
            config,
            calendar: None,
            mode,
            last_update: None,
        }
    }

    pub async fn load(&mut self) -> Result<()> {
        let calendar = fetch_contributions(&self.config.token, &self.config.username).await?;
        self.calendar = Some(calendar);
        self.last_update = Some(Instant::now());
        Ok(())
    }

    pub async fn refresh(&mut self) -> Result<()> {
        if self.mode == AppMode::Watch
            && self
                .last_update
                .map(|t| t.elapsed() > Duration::from_secs(300))
                .unwrap_or(true)
        {
            self.load().await?;
        }
        Ok(())
    }
}
