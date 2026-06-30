/// Severity level of a diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum Severity {
  Error,
  Warning,
  Info,
}

/// Location in source code where a diagnostic was triggered.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct SourceLocation {
  pub file: String,
  pub line: usize,
  pub column: usize,
  pub span: Option<(usize, usize)>,
}

/// A diagnostic emitted by a rule during analysis.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Diagnostic {
  pub severity: Severity,
  pub message: String,
  pub location: SourceLocation,
  pub rule_id: String,
}
