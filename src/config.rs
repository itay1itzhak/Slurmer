use color_eyre::eyre::WrapErr;
use color_eyre::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

const ENV_SLURM_LOGS_DIR: &str = "SLURMER_SLURM_LOGS_DIR";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SlurmerConfig {
    pub slurm_logs_dir: Option<String>,
}

pub fn load_config() -> Result<SlurmerConfig> {
    let path = config_file_path()?;
    if !path.exists() {
        return Ok(SlurmerConfig::default());
    }

    let raw = fs::read_to_string(&path).wrap_err("failed reading config file")?;
    let cfg: SlurmerConfig = toml::from_str(&raw).wrap_err("failed parsing config toml")?;
    Ok(cfg)
}

pub fn save_config(cfg: &SlurmerConfig) -> Result<()> {
    let path = config_file_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).wrap_err("failed creating config directory")?;
    }
    let raw = toml::to_string_pretty(cfg).wrap_err("failed serializing config toml")?;
    fs::write(&path, raw).wrap_err("failed writing config file")?;
    Ok(())
}

pub fn resolve_slurm_logs_dir(cfg: &SlurmerConfig) -> Option<PathBuf> {
    if let Ok(v) = std::env::var(ENV_SLURM_LOGS_DIR) {
        let trimmed = v.trim();
        if !trimmed.is_empty() {
            return Some(PathBuf::from(trimmed));
        }
    }

    cfg.slurm_logs_dir
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
}

fn config_file_path() -> Result<PathBuf> {
    let base = xdg_config_home()?;
    Ok(base.join("slurmer").join("config.toml"))
}

fn xdg_config_home() -> Result<PathBuf> {
    if let Ok(v) = std::env::var("XDG_CONFIG_HOME") {
        let p = PathBuf::from(v);
        if !p.as_os_str().is_empty() {
            return Ok(p);
        }
    }

    let home = std::env::var("HOME").wrap_err("HOME is not set")?;
    Ok(Path::new(&home).join(".config"))
}

