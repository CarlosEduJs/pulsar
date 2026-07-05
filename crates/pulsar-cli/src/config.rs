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

#[cfg(test)]
mod tests {
  use super::*;

  // Deserialization from TOML
  // =========================

  #[test]
  fn empty_toml_uses_defaults() {
    let config: PulsarConfig = toml::from_str("").unwrap();
    assert!(config.settings.ignore.is_empty());
    assert!(config.settings.rules.is_empty());
    assert!(config.database.schema.is_none());
  }

  #[test]
  fn settings_ignore() {
    let toml = r#"
[settings]
ignore = ["node_modules", "dist"]
"#;
    let config: PulsarConfig = toml::from_str(toml).unwrap();
    assert_eq!(config.settings.ignore, vec!["node_modules", "dist"]);
    assert!(config.settings.rules.is_empty());
  }

  #[test]
  fn settings_rules() {
    let toml = r#"
[settings]
rules = ["no-select-star", "no-missing-limit"]
"#;
    let config: PulsarConfig = toml::from_str(toml).unwrap();
    assert_eq!(config.settings.rules, vec!["no-select-star", "no-missing-limit"]);
  }

  #[test]
  fn database_schema_path() {
    let toml = r#"
[database]
schema = "./prisma/schema.prisma"
"#;
    let config: PulsarConfig = toml::from_str(toml).unwrap();
    assert_eq!(config.database.schema, Some("./prisma/schema.prisma".to_string()));
  }

  #[test]
  fn full_config() {
    let toml = r#"
[settings]
ignore = ["node_modules"]
rules = ["no-select-star"]

[database]
schema = "./schema.prisma"
"#;
    let config: PulsarConfig = toml::from_str(toml).unwrap();
    assert_eq!(config.settings.ignore, vec!["node_modules"]);
    assert_eq!(config.settings.rules, vec!["no-select-star"]);
    assert_eq!(config.database.schema, Some("./schema.prisma".to_string()));
  }

  #[test]
  fn partial_config_only_settings() {
    let toml = r#"
[settings]
ignore = ["dist"]
"#;
    let config: PulsarConfig = toml::from_str(toml).unwrap();
    assert_eq!(config.settings.ignore, vec!["dist"]);
    assert!(config.database.schema.is_none());
  }

  #[test]
  fn partial_config_only_database() {
    let toml = r#"
[database]
schema = "./custom.prisma"
"#;
    let config: PulsarConfig = toml::from_str(toml).unwrap();
    assert!(config.settings.ignore.is_empty());
    assert_eq!(config.database.schema, Some("./custom.prisma".to_string()));
  }

  #[test]
  fn ignore_defaults_to_empty() {
    let config: PulsarConfig = toml::from_str("").unwrap();
    assert!(config.settings.ignore.is_empty());
  }

  #[test]
  fn rules_defaults_to_empty() {
    let config: PulsarConfig = toml::from_str("").unwrap();
    assert!(config.settings.rules.is_empty());
  }

  #[test]
  fn schema_defaults_to_none() {
    let config: PulsarConfig = toml::from_str("").unwrap();
    assert!(config.database.schema.is_none());
  }

  #[test]
  fn invalid_toml_returns_parse_error() {
    let result: Result<PulsarConfig, toml::de::Error> = toml::from_str("[[[");
    assert!(result.is_err());
  }

  // PulsarConfig::load & ConfigError
  // ================================

  #[test]
  fn load_explicit_not_found() {
    let result = PulsarConfig::load(Some(Path::new("/tmp/nonexistent-pulsar-file.toml")));
    match result {
      Err(ConfigError::NotFound(p)) => assert!(p.contains("nonexistent-pulsar-file")),
      _ => panic!("expected ConfigError::NotFound"),
    }
  }

  #[test]
  fn load_explicit_path() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("pulsar.toml");
    std::fs::write(
      &config_path,
      r#"
[settings]
rules = ["no-select-star"]
"#,
    )
    .unwrap();

    let result = PulsarConfig::load(Some(&config_path));
    let config = result.unwrap();
    assert_eq!(config.settings.rules, vec!["no-select-star"]);
    assert!(config.database.schema.is_none());
  }

  #[test]
  fn load_explicit_invalid_toml() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("pulsar.toml");
    std::fs::write(&config_path, "[[[invalid toml").unwrap();

    let result = PulsarConfig::load(Some(&config_path));
    match result {
      Err(ConfigError::Parse(..)) => {}
      _ => panic!("expected ConfigError::Parse"),
    }
  }

  #[test]
  fn load_explicit_path_with_all_fields() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("pulsar.toml");
    std::fs::write(
      &config_path,
      r#"
[settings]
ignore = ["node_modules"]
rules = ["no-select-star", "no-missing-limit"]

[database]
schema = "./prisma/schema.prisma"
"#,
    )
    .unwrap();

    let config = PulsarConfig::load(Some(&config_path)).unwrap();
    assert_eq!(config.settings.ignore, vec!["node_modules"]);
    assert_eq!(config.settings.rules, vec!["no-select-star", "no-missing-limit"]);
    assert_eq!(config.database.schema, Some("./prisma/schema.prisma".to_string()));
  }

  // ConfigError Display
  // ===================

  #[test]
  fn config_error_not_found_display() {
    let err = ConfigError::NotFound("my-config.toml".to_string());
    assert_eq!(err.to_string(), "config file not found: my-config.toml");
  }

  #[test]
  fn config_error_io_display() {
    let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
    let err = ConfigError::Io("config.toml".to_string(), io_err);
    let msg = err.to_string();
    assert!(msg.contains("config.toml"));
    assert!(msg.contains("failed to read"));
  }

  #[test]
  fn config_error_parse_display() {
    // Produce a parse error from real invalid TOML
    let parse_err: Result<PulsarConfig, toml::de::Error> = toml::from_str("[[[");
    let parse_err = parse_err.unwrap_err();
    let err = ConfigError::Parse("config.toml".to_string(), parse_err);
    let msg = err.to_string();
    assert!(msg.contains("config.toml"));
    assert!(msg.contains("failed to parse"));
  }
}
