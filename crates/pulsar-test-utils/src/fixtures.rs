use std::path::{Path, PathBuf};

/// Default path to the test fixtures directory, relative to workspace root.
pub const DEFAULT_FIXTURES_DIR: &str = "test/fixtures";

/// Returns the path to the fixtures directory.
///
/// Resolves from the workspace root by trying common relative paths.
/// Can be overridden by setting the `PULSAR_FIXTURES` environment variable.
#[must_use]
pub fn fixtures_dir() -> PathBuf {
  if let Ok(dir) = std::env::var("PULSAR_FIXTURES") {
    return PathBuf::from(dir);
  }

  let candidates: [&Path; 3] = [
    Path::new(DEFAULT_FIXTURES_DIR),  // from workspace root
    Path::new("../../test/fixtures"), // from crates/*/
    Path::new("../test/fixtures"),    // from crates/*/src/
  ];

  for p in &candidates {
    if p.is_dir() {
      return p.to_path_buf();
    }
  }

  PathBuf::from(DEFAULT_FIXTURES_DIR)
}

/// Reads a fixture file as a string.
///
/// `path` is relative to the fixtures directory (e.g., `"basic.ts"` or
/// `"no-missing-limit/query-without-limit.ts"`).
///
/// # Panics
///
/// Panics if the file cannot be read.
pub fn read_fixture(path: &str) -> String {
  let full_path = fixtures_dir().join(path);
  std::fs::read_to_string(&full_path)
    .unwrap_or_else(|e| panic!("failed to read fixture `{}`: {e}", full_path.display()))
}

/// Returns the path to a fixture file.
///
/// `path` is relative to the fixtures directory.
#[must_use]
pub fn fixture_path(path: &str) -> PathBuf {
  fixtures_dir().join(path)
}

/// Reads a fixture from a rule-specific subdirectory.
///
/// Example: `read_rule_fixture("no-missing-limit", "query-without-limit.ts")`
pub fn read_rule_fixture(rule_name: &str, file_name: &str) -> String {
  read_fixture(&format!("{rule_name}/{file_name}"))
}

// Known fixtures
// ==============

pub fn basic_ts() -> String {
  read_fixture("basic.ts")
}

pub fn clean_ts() -> String {
  read_fixture("clean.ts")
}

pub fn no_issues_ts() -> String {
  read_fixture("no-issues.ts")
}

pub fn with_where_ts() -> String {
  read_fixture("with-where.ts")
}

pub fn with_limit_ts() -> String {
  read_fixture("with-limit.ts")
}

pub fn mixed_star_explicit_ts() -> String {
  read_fixture("mixed-star-explicit.ts")
}

pub fn invalid_syntax_ts() -> String {
  read_fixture("invalid-syntax.ts")
}

// Schema fixtures
// ===============

pub fn schema_prisma() -> String {
  read_fixture("schema/schema.prisma")
}

pub fn schema_pulsar_toml() -> String {
  read_fixture("schema/pulsar.toml")
}

pub fn schema_path() -> PathBuf {
  fixture_path("schema/schema.prisma")
}
