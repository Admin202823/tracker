use anyhow::{Context as _, Result};
use clap::Parser;
use rust_i18n::i18n;

i18n!("locales", fallback = "en");

mod app;
mod config;
mod coordinates;
mod event;
mod group;
mod object;
mod shared_state;
mod tui;
mod utils;
mod widgets;

use app::App;
use config::Config;
use std::path::PathBuf;

#[derive(Parser)]
#[command(version)]
struct Args {
    /// Path to a custom configuration file
    #[arg(long, value_name = "FILE")]
    config: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command-line arguments
    let args = Args::parse();
    
    // Set the application's locale based on the system locale
    let locale = sys_locale::get_locale().unwrap_or_else(|| String::from("en-US"));
    rust_i18n::set_locale(&locale);

    let config = match load_config(args.config) {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Configuration error: {e}");
            std::process::exit(1);
        }
    };
    let mut app = App::with_config(config).context("failed to initialize application")?;
    app.run().await
}

fn load_config(custom_path: Option<std::path::PathBuf>) -> Result<Config> {
    if let Some(path) = custom_path {
        // Mode strict : l'utilisateur a explicitement fourni un fichier
        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("failed to read config file: {}", path.display()))?;

        let config: Config = toml::from_str(&content)
            .with_context(|| format!("failed to parse config file: {}", path.display()))?;

        return Ok(config);
    }

    // Mode normal (fallback)
    let path = std::env::home_dir()
        .context("failed to get home directory")?
        .join(".config/tracker/config.toml");

    if !path.exists() {
        return Ok(Config::default());
    }

    let content = std::fs::read_to_string(&path)?;
    let config: Config = toml::from_str(&content)?;
    Ok(config)
}