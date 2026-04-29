mod action;
mod app;
mod auth;
mod components;
mod config;
mod imap_client;
mod mail;
mod smtp_client;

use anyhow::Result;
use tracing_appender::rolling;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    // File logging (not stdout — that's the TUI)
    let log_dir = config::Config::path()
        .parent()
        .unwrap_or(std::path::Path::new("."))
        .to_path_buf();
    let file_appender = rolling::never(&log_dir, "crabmail.log");
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("crabmail=info".parse()?))
        .with_writer(file_appender)
        .with_ansi(false)
        .init();

    let config = config::Config::load()?;

    // Panic hook to restore terminal
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        ratatui::restore();
        original_hook(panic_info);
    }));

    let mut terminal = ratatui::init();
    let mut app = app::App::new(config);
    let result = app.run(&mut terminal).await;
    ratatui::restore();
    result
}
