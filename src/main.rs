use crate::config::Config;
use anyhow::Result;
use std::{io, io::Write};

mod api;
mod app;
mod config;
mod models;
mod ui;

#[tokio::main]
async fn main() -> Result<()> {
    let config = if Config::exists() {
        Config::load()?
    } else {
        let mut username = String::new();
        let mut token = String::new();

        print!("GitHub username: ");
        io::stdout().flush()?;
        io::stdin().read_line(&mut username)?;
        print!("GitHub token: ");
        io::stdout().flush()?;
        io::stdin().read_line(&mut token)?;

        let config = Config {
            username: username.trim().to_string(),
            token: token.trim().to_string(),
        };

        config.save()?;
        config
    };

    Ok(())
}
