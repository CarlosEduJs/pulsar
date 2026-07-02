use std::path::Path;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct PulsarConfig {
  #[serde(default)]
  pub settings: Settings,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
  /// Directories/files to ignore (in addition to .gitignore).
  #[serde(default)]
  pub ignore: Vec<String>,
  /// Enabled rules. Empty means all built-in rules.
  #[serde(default)]
  pub rules: Vec<String>,
}

impl Default for PulsarConfig {
  fn default() -> Self {
    Self { settings: Settings::default() }
  }
}

impl Default for Settings {
  fn default() -> Self {
    Self { ignore: vec![], rules: vec![] }
  }
}

impl PulsarConfig {
  /// Loads config from `--config <path>`, or auto-detects `./pulsar.toml`.
  ///
  /// Returns `Ok(None)` if no file exists and no explicit path was given
  /// (caller falls back to defaults).
  pub fn load(explicit: Option<&Path>) -> Result<Self, ConfigError> {
    let path = match explicit {
      Some(p) => {
        if !p.exists() {
          return Err(ConfigError::NotFound(p.to_string_lossy().to_string()));
        }
        p.to_owned()
      }
      None => {
        let auto = Path::new("pulsar.toml");
        if !auto.exists() {
          return Ok(Self::default());
        }
        auto.to_owned()
      }
    };

    let contents = std::fs::read_to_string(&path)
      .map_err(|e| ConfigError::Io(path.to_string_lossy().to_string(), e))?;

    toml::from_str(&contents).map_err(|e| ConfigError::Parse(path.to_string_lossy().to_string(), e))
  }
}

#[derive(Debug)]
pub enum ConfigError {
  NotFound(String),
  Io(String, std::io::Error),
  Parse(String, toml::de::Error),
}

impl std::fmt::Display for ConfigError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::NotFound(p) => write!(f, "config file not found: {p}"),
      Self::Io(p, e) => write!(f, "failed to read {p}: {e}"),
      Self::Parse(p, e) => write!(f, "failed to parse {p}: {e}"),
    }
  }
}

impl std::error::Error for ConfigError {}
