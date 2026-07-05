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

#[cfg(test)]
mod tests {
  use super::*;

  // Severity
  // ========

  #[test]
  fn severity_variants_are_distinct() {
    assert_ne!(Severity::Error, Severity::Warning);
    assert_ne!(Severity::Error, Severity::Info);
    assert_ne!(Severity::Warning, Severity::Info);
  }

  #[test]
  fn severity_debug() {
    assert_eq!(format!("{:?}", Severity::Error), "Error");
    assert_eq!(format!("{:?}", Severity::Warning), "Warning");
    assert_eq!(format!("{:?}", Severity::Info), "Info");
  }

  #[test]
  fn severity_clone_copy() {
    let a = Severity::Error;
    let b = a;
    assert_eq!(a, b);
  }

  #[test]
  fn severity_serialize_json() {
    let json = serde_json::to_string(&Severity::Error).unwrap();
    assert_eq!(json, "\"Error\"");
    let json = serde_json::to_string(&Severity::Warning).unwrap();
    assert_eq!(json, "\"Warning\"");
    let json = serde_json::to_string(&Severity::Info).unwrap();
    assert_eq!(json, "\"Info\"");
  }

  #[test]
  fn severity_serialize_roundtrip() {
    for variant in &[Severity::Error, Severity::Warning, Severity::Info] {
      let json = serde_json::to_string(variant).unwrap();
      // Serde Serialize is derived but not Deserialize — just verify output
      assert!(json.starts_with('"'));
    }
  }

  // SourceLocation
  // ==============

  fn sample_loc() -> SourceLocation {
    SourceLocation { file: "test.ts".to_string(), line: 5, column: 10, span: Some((20, 30)) }
  }

  #[test]
  fn source_location_fields() {
    let loc = sample_loc();
    assert_eq!(loc.file, "test.ts");
    assert_eq!(loc.line, 5);
    assert_eq!(loc.column, 10);
    assert_eq!(loc.span, Some((20, 30)));
  }

  #[test]
  fn source_location_no_span() {
    let loc = SourceLocation { file: "a.ts".to_string(), line: 1, column: 1, span: None };
    assert!(loc.span.is_none());
  }

  #[test]
  fn source_location_debug() {
    let loc = sample_loc();
    let dbg = format!("{loc:?}");
    assert!(dbg.contains("test.ts"));
    assert!(dbg.contains("line: 5"));
    assert!(dbg.contains("column: 10"));
    assert!(dbg.contains("span: Some"));
  }

  #[test]
  fn source_location_clone() {
    let a = sample_loc();
    let b = a.clone();
    assert_eq!(a, b);
  }

  #[test]
  fn source_location_equality() {
    let a = sample_loc();
    let b = sample_loc();
    let c = SourceLocation { file: "other.ts".to_string(), ..a.clone() };
    assert_eq!(a, b);
    assert_ne!(a, c);
  }

  #[test]
  fn source_location_serialize_json() {
    let loc = sample_loc();
    let json = serde_json::to_string(&loc).unwrap();
    assert!(json.contains("test.ts"));
    assert!(json.contains("\"line\":5"));
    assert!(json.contains("\"column\":10"));
    assert!(json.contains("\"span\":[20,30]"));
  }

  #[test]
  fn source_location_serialize_no_span() {
    let loc = SourceLocation { file: "x.ts".to_string(), line: 1, column: 2, span: None };
    let json = serde_json::to_string(&loc).unwrap();
    assert!(json.contains("\"span\":null"));
  }

  // Diagnostic
  // ==========

  fn sample_diag() -> Diagnostic {
    Diagnostic {
      severity: Severity::Error,
      message: "Avoid implicit SELECT *.".to_string(),
      location: sample_loc(),
      rule_id: "no-select-star".to_string(),
    }
  }

  #[test]
  fn diagnostic_fields() {
    let diag = sample_diag();
    assert_eq!(diag.severity, Severity::Error);
    assert_eq!(diag.message, "Avoid implicit SELECT *.");
    assert_eq!(diag.location, sample_loc());
    assert_eq!(diag.rule_id, "no-select-star");
  }

  #[test]
  fn diagnostic_debug() {
    let diag = sample_diag();
    let dbg = format!("{diag:?}");
    assert!(dbg.contains("no-select-star"));
    assert!(dbg.contains("Avoid implicit"));
    assert!(dbg.contains("Error"));
  }

  #[test]
  fn diagnostic_clone() {
    let a = sample_diag();
    let b = a.clone();
    assert_eq!(a, b);
  }

  #[test]
  fn diagnostic_equality() {
    let a = sample_diag();
    let b = sample_diag();
    let c = Diagnostic { severity: Severity::Warning, ..a.clone() };
    assert_eq!(a, b);
    assert_ne!(a, c);
  }

  #[test]
  fn diagnostic_serialize_json() {
    let diag = sample_diag();
    let json = serde_json::to_string_pretty(&diag).unwrap();
    assert!(json.contains("\"rule_id\": \"no-select-star\""));
    assert!(json.contains("\"severity\": \"Error\""));
    assert!(json.contains("\"message\": \"Avoid implicit SELECT *.\""));
    assert!(json.contains("\"file\": \"test.ts\""));
    assert!(json.contains("\"line\": 5"));
    assert!(json.contains("\"column\": 10"));
  }

  #[test]
  fn diagnostic_info_severity() {
    let diag = Diagnostic {
      severity: Severity::Info,
      message: "Informational message.".to_string(),
      location: SourceLocation { file: "info.ts".to_string(), line: 3, column: 5, span: None },
      rule_id: "some-rule".to_string(),
    };
    assert_eq!(diag.severity, Severity::Info);
    let json = serde_json::to_string(&diag).unwrap();
    assert!(json.contains("\"Info\""));
  }

  #[test]
  fn diagnostic_warning_severity() {
    let diag = Diagnostic {
      severity: Severity::Warning,
      message: "Warning message.".to_string(),
      location: SourceLocation { file: "warn.ts".to_string(), line: 2, column: 3, span: None },
      rule_id: "warn-rule".to_string(),
    };
    assert_eq!(diag.severity, Severity::Warning);
    let json = serde_json::to_string(&diag).unwrap();
    assert!(json.contains("\"Warning\""));
  }
}
