use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const APP_DIR: &str = ".opencan";
const CONFIG_NAME: &str = "config.toml";
const API_KEY_FALLBACK_VARS: [&str; 2] = ["OPEN_API_KEY", "OPENAI_API_KEY"];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub model: String,
    pub base_url: String,
    pub api_key_env: String,
    pub temperature: f32,
    pub system_prompt: String,
    pub memory_file: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            model: "gpt-5-mini".to_string(),
            base_url: "https://api.openai.com/v1".to_string(),
            api_key_env: "OPEN_API_KEY".to_string(),
            temperature: 1.0,
            system_prompt: "You are OpenCan, a pragmatic coding assistant. Prefer concise, correct answers and ask for clarification only when blocked.".to_string(),
            memory_file: "~/.opencan/MEMORY.md".to_string(),
        }
    }
}

pub fn app_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Could not resolve home directory")?;
    Ok(home.join(APP_DIR))
}

pub fn config_path() -> Result<PathBuf> {
    Ok(app_dir()?.join(CONFIG_NAME))
}

pub fn sessions_dir() -> Result<PathBuf> {
    Ok(app_dir()?.join("sessions"))
}

pub fn expand_tilde(path: &str) -> Result<PathBuf> {
    if let Some(stripped) = path.strip_prefix("~/") {
        let home = dirs::home_dir().context("Could not resolve home directory")?;
        return Ok(home.join(stripped));
    }

    Ok(PathBuf::from(path))
}

impl Config {
    pub fn load() -> Result<Self> {
        let path = config_path()?;
        let raw = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config at {}", path.display()))?;
        let config: Self = toml::from_str(&raw).context("Invalid TOML in config file")?;
        Ok(config)
    }

    pub fn save_to_path(&self, path: &Path) -> Result<()> {
        let contents = toml::to_string_pretty(self).context("Failed to serialize config")?;
        fs::write(path, contents)
            .with_context(|| format!("Failed to write config at {}", path.display()))?;
        Ok(())
    }

    pub fn api_key(&self) -> Result<String> {
        let (_, key) = self.resolve_api_key()?;
        Ok(key)
    }

    pub fn resolve_api_key(&self) -> Result<(String, String)> {
        for name in self.api_key_candidates() {
            match env::var(&name) {
                Ok(value) if !value.trim().is_empty() => return Ok((name, value)),
                _ => continue,
            }
        }

        bail!(
            "No API key found. Checked env vars: {}. Export one and retry.",
            self.api_key_candidates().join(", ")
        )
    }

    pub fn memory_path(&self) -> Result<PathBuf> {
        expand_tilde(&self.memory_file)
    }

    pub fn build_system_prompt(&self) -> Result<String> {
        let memory_path = self.memory_path()?;
        let memory = fs::read_to_string(&memory_path).unwrap_or_default();
        if memory.trim().is_empty() {
            return Ok(self.system_prompt.clone());
        }

        Ok(format!(
            "{}\n\nLong-term project memory:\n{}",
            self.system_prompt, memory
        ))
    }
}

impl Config {
    fn api_key_candidates(&self) -> Vec<String> {
        let mut names = vec![self.api_key_env.clone()];
        for fallback in API_KEY_FALLBACK_VARS {
            if !names.iter().any(|name| name == fallback) {
                names.push(fallback.to_string());
            }
        }
        names
    }
}

pub fn ensure_layout(config: &Config) -> Result<()> {
    let app = app_dir()?;
    fs::create_dir_all(&app)
        .with_context(|| format!("Failed to create app dir {}", app.display()))?;

    let sessions = sessions_dir()?;
    fs::create_dir_all(&sessions)
        .with_context(|| format!("Failed to create sessions dir {}", sessions.display()))?;

    let memory_path = config.memory_path()?;
    if let Some(parent) = memory_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create memory dir {}", parent.display()))?;
    }

    if !memory_path.exists() {
        fs::write(&memory_path, "# OpenCan Memory\n")
            .with_context(|| format!("Failed to create memory file {}", memory_path.display()))?;
    }

    Ok(())
}

pub fn write_default_config(
    force: bool,
    model: Option<String>,
    base_url: Option<String>,
    api_key_env: Option<String>,
) -> Result<PathBuf> {
    let mut config = Config::default();
    if let Some(value) = model {
        config.model = value;
    }
    if let Some(value) = base_url {
        config.base_url = value;
    }
    if let Some(value) = api_key_env {
        config.api_key_env = value;
    }

    let app = app_dir()?;
    fs::create_dir_all(&app)
        .with_context(|| format!("Failed to create app dir {}", app.display()))?;

    let path = config_path()?;
    if path.exists() && !force {
        bail!(
            "Config already exists at {}. Re-run with --force to overwrite.",
            path.display()
        );
    }

    config.save_to_path(&path)?;
    ensure_layout(&config)?;
    Ok(path)
}

pub fn append_memory_note(config: &Config, note: &str) -> Result<()> {
    let memory_path = config.memory_path()?;
    if let Some(parent) = memory_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create memory dir {}", parent.display()))?;
    }

    let mut existing = fs::read_to_string(&memory_path).unwrap_or_default();
    if !existing.ends_with('\n') {
        existing.push('\n');
    }

    existing.push_str("- ");
    existing.push_str(note.trim());
    existing.push('\n');

    fs::write(&memory_path, existing)
        .with_context(|| format!("Failed writing memory file {}", memory_path.display()))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn non_tilde_path_is_unchanged() {
        let input = "./tmp/file";
        let output = expand_tilde(input).expect("expand should succeed");
        assert_eq!(output, PathBuf::from(input));
    }
}
