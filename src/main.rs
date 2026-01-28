use crate::app::{App, Focus};
use crate::background::spawn_cache_updater;
use crate::cache::Cache;
use crate::config::Config;
use anyhow::Result;
use clap::Parser;
use ratatui::{
    Terminal, TerminalOptions,
    crossterm::{
        event::{self, Event, KeyCode},
        execute,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    },
    prelude::CrosstermBackend,
};
use std::{
    io::{self},
    process,
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

    #[arg(long)]
    reset: bool,

    username: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let mut config = if Config::exists() && !args.reset {
        Config::load()?
    } else {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let mut config = Config {
            username: "".to_string(),
            token: "".to_string(),
        };
        let mut app = App::new(config.clone());

        loop {
            terminal.draw(|frame| ui::render_input(frame, &app))?;

            if event::poll(Duration::from_millis(200))?
                && let Event::Key(key) = event::read()?
            {
                match key.code {
                    KeyCode::Tab => {
                        app.focus = match app.focus {
                            Focus::Username => Focus::Token,
                            Focus::Token => Focus::Username,
                        };
                    }

                    KeyCode::Enter => {
                        config.username = app.config.username.trim().to_string();
                        config.token = app.config.token.trim().to_string();

                        if !config.username.is_empty() && !config.token.is_empty() {
                            config.save()?;
                            break;
                        }
                    }

                    KeyCode::Char(c) => {
                        let target = match app.focus {
                            Focus::Username => &mut app.config.username,
                            Focus::Token => &mut app.config.token,
                        };
                        target.push(c);
                    }

                    KeyCode::Backspace => {
                        let target = match app.focus {
                            Focus::Username => &mut app.config.username,
                            Focus::Token => &mut app.config.token,
                        };
                        target.pop();
                    }

                    KeyCode::Esc => {
                        disable_raw_mode()?;
                        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
                        process::exit(0);
                    }

                    _ => {}
                }
            }
        }

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

    if let Some(watch_secs) = args.watch {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;

        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        let app = Arc::new(Mutex::new(app));
        let terminal = Arc::new(Mutex::new(terminal));

        {
            let app = app.lock().await;
            let mut term = terminal.lock().await;
            term.draw(|f| ui::render(f, &app))?;
        }

        let interval = Duration::from_secs(watch_secs);
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
                let mut term = terminal_clone.lock().await;
                let _ = term.draw(|f| ui::render(f, &app));
            }
        });

        loop {
            if event::poll(Duration::from_millis(200))?
                && let Event::Key(key) = event::read()?
                && key.code == KeyCode::Char('q')
            {
                break;
            }
        }

        execute!(io::stdout(), LeaveAlternateScreen)?;
        disable_raw_mode()?;
    } else {
        let backend = CrosstermBackend::new(io::stdout());
        let mut terminal = Terminal::with_options(
            backend,
            TerminalOptions {
                viewport: ratatui::Viewport::Inline(13),
            },
        )?;

        terminal.draw(|frame| ui::render(frame, &app))?;

        println!();
    }

    Ok(())
}
