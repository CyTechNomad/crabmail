use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub name: String,
    pub email: String,
    pub imap_host: String,
    #[serde(default = "default_imap_port")]
    pub imap_port: u16,
    pub smtp_host: String,
    #[serde(default = "default_smtp_port")]
    pub smtp_port: u16,
    #[serde(default = "default_true")]
    pub use_tls: bool,
}

fn default_imap_port() -> u16 {
    993
}
fn default_smtp_port() -> u16 {
    587
}
fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub accounts: Vec<Account>,
    #[serde(default)]
    pub auto_refresh_seconds: Option<u64>,
}

impl Config {
    pub fn path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("crabmail")
            .join("config.toml")
    }

    pub fn load() -> Result<Self> {
        let path = Self::path();
        if !path.exists() {
            let config = Config::default();
            config.save()?;
            return Ok(config);
        }
        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config: {}", path.display()))?;
        toml::from_str(&content).with_context(|| "Failed to parse config")
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        fs::write(&path, content)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&path, fs::Permissions::from_mode(0o600));
        }
        Ok(())
    }
}
