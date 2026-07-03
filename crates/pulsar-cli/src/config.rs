use std::path::Path;

use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
pub struct PulsarConfig {
  #[serde(default)]
  pub settings: Settings,
  /// Database / schema configuration.
  #[serde(default)]
  pub database: DatabaseConfig,
}

#[derive(Debug, Default, Deserialize)]
pub struct Settings {
  /// File or directory *names* to skip by exact match (in addition to .gitignore).
  #[serde(default)]
  pub ignore: Vec<String>,
  /// Enabled rules. Empty means all built-in rules.
  #[serde(default)]
  pub rules: Vec<String>,
}

/// Database / schema configuration.
#[derive(Debug, Default, Deserialize)]
pub struct DatabaseConfig {
  /// Path to a Prisma schema file (e.g. `"./schema.prisma"`).
  #[serde(default)]
  pub schema: Option<String>,
}

impl PulsarConfig {
  /// Loads config from `--config <path>`, or auto-detects `./pulsar.toml`.
  ///
  /// Returns `Ok(Default::default())` if no file exists and no explicit path
  /// was given (caller falls back to all built-in rules).
  pub fn load(explicit: Option<&Path>) -> Result<Self, ConfigError> {
    let path = if let Some(p) = explicit {
      if !p.exists() {
        return Err(ConfigError::NotFound(p.to_string_lossy().to_string()));
      }
      p.to_owned()
    } else {
      let auto = Path::new("pulsar.toml");
      if !auto.exists() {
        return Ok(Self::default());
      }
      auto.to_owned()
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

impl std::error::Error for ConfigError {
  fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
    match self {
      Self::Io(_, e) => Some(e),
      Self::Parse(_, e) => Some(e),
      Self::NotFound(_) => None,
    }
  }
}
