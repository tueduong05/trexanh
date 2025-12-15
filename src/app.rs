use crate::api::fetch_contributions;
use crate::config::Config;
use crate::models::ContributionCalendar;
use anyhow::Result;

#[derive(PartialEq)]
pub enum Focus {
    Username,
    Token,
}

pub struct App {
    pub config: Config,
    pub focus: Focus,
    pub calendar: Option<ContributionCalendar>,
}

impl App {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            focus: Focus::Username,
            calendar: None,
        }
    }

    pub async fn load(&mut self) -> Result<()> {
        let calendar = fetch_contributions(&self.config.token, &self.config.username).await?;
        self.calendar = Some(calendar);
        Ok(())
    }

    pub async fn refresh(&mut self) -> Result<()> {
        self.load().await?;
        Ok(())
    }
}
