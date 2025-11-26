use crate::app::App;
use crate::background::spawn_cache_updater;
use crate::cache::Cache;
use crate::config::Config;
use anyhow::Result;
use clap::Parser;
use ratatui::{
    Terminal,
    crossterm::{
        event::{self, Event, KeyCode},
        terminal::{disable_raw_mode, enable_raw_mode},
    },
    prelude::CrosstermBackend,
};
use std::{
    io::{self, Write},
    sync::Arc,
    time::Duration,
};
use tokio::{sync::Mutex, time::sleep};

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
    #[arg(long, hide = true)]
    update_cache: bool,

    #[arg(long)]
    cached: bool,
    #[arg(long)]
    watch: Option<u64>,

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
        let mut app = App::new(config.clone());
        app.load().await?;
        if let Some(calendar) = app.calendar {
            let mut cache = Cache::load()?;
            cache.insert(config.username, calendar);
            cache.save()?;
        }
        return Ok(());
    }

    let mut app = App::new(config.clone());
    let mut cache = Cache::load()?;

    if args.cached {
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

    if args.watch.is_some() {
        let app = Arc::new(Mutex::new(app));
        let terminal = Arc::new(Mutex::new(terminal));

        {
            let app = app.lock().await;
            let mut terminal = terminal.lock().await;
            terminal.draw(|f| ui::render(f, &app))?;
        }

        let interval = Duration::from_secs(args.watch.unwrap());

        let app_clone = Arc::clone(&app);
        let terminal_clone = Arc::clone(&terminal);

        tokio::spawn(async move {
            loop {
                sleep(interval).await;

                {
                    let mut app = app_clone.lock().await;
                    let _ = app.refresh().await;
                }

                let app = app_clone.lock().await;
                let mut terminal = terminal_clone.lock().await;
                let _ = terminal.draw(|f| ui::render(f, &app));
            }
        });

        loop {
            if event::poll(Duration::from_millis(200))? {
                match event::read()? {
                    Event::Key(key) => {
                        if key.code == KeyCode::Char('q') {
                            break;
                        }
                    }
                    Event::Resize(_, _) => {
                        let app = app.lock().await;
                        let mut terminal = terminal.lock().await;
                        terminal.draw(|f| ui::render(f, &app))?;
                    }
                    _ => {}
                }
            }
        }
    } else {
        terminal.draw(|frame| ui::render(frame, &app))?;
    }

    disable_raw_mode()?;
    Ok(())
}
