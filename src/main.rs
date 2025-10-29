use crate::app::{App, AppMode};
use crate::background::spawn_cache_updater;
use crate::cache::Cache;
use crate::config::Config;
use anyhow::Result;
use clap::Parser;
use ratatui::{
    Terminal,
    crossterm::terminal::{disable_raw_mode, enable_raw_mode},
    prelude::CrosstermBackend,
};
use std::io::{self, Write};

mod api;
mod app;
mod background;
mod cache;
mod config;
mod models;
mod ui;

#[derive(Parser)]
#[command(name = "trexanh")]
#[command(about = "GitHub Contribution Graph TUI")]
struct Args {
    #[arg(long)]
    cached: bool,

    #[arg(long, hide = true)]
    update_cache: bool,

    username: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let mut config = if Config::exists() {
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

    config.username = args
        .username
        .as_deref()
        .unwrap_or(&config.username)
        .to_string();

    if args.update_cache {
        let mut app = App::new(config.clone(), AppMode::Single);
        app.load().await?;
        if let Some(calendar) = app.calendar {
            let mut cache = Cache::load()?;
            cache.insert(config.username, calendar);
            cache.save()?;
        }
        return Ok(());
    }

    let mut cache = Cache::load()?;

    if args.cached {
        let mut app = App::new(config.clone(), AppMode::Single);

        if let Some(calendar) = cache.get(&config.username) {
            app.calendar = Some(calendar);

            spawn_cache_updater(&config.username)?;
        } else {
            app.load().await?;

            if let Some(ref calendar) = app.calendar {
                cache.insert(config.username, calendar.clone());
                cache.save()?;
            }
        }

        enable_raw_mode()?;
        let stdout = io::stdout();
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        terminal.clear()?;
        terminal.draw(|frame| ui::render(frame, &app))?;
        disable_raw_mode()?;
    } else {
        let mut app = App::new(config.clone(), AppMode::Single);
        app.load().await?;

        if let Some(ref calendar) = app.calendar {
            cache.insert(config.username, calendar.clone());
            cache.save()?;
        }

        enable_raw_mode()?;
        let stdout = io::stdout();
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        terminal.clear()?;
        terminal.draw(|frame| ui::render(frame, &app))?;
        disable_raw_mode()?;
    }

    Ok(())
}
