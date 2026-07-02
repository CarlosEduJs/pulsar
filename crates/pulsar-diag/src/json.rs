use pulsar_core::Diagnostic;

use super::formatter::DiagnosticFormatter;

/// Formats diagnostics as a JSON array.
///
/// Each diagnostic is serialized with severity, `rule_id`, message, and location fields.
pub struct JsonFormatter;

impl DiagnosticFormatter for JsonFormatter {
  fn format(&self, diagnostics: &[Diagnostic], _source_text: &str) -> String {
    serde_json::to_string_pretty(diagnostics).unwrap_or_else(|_| "[]".to_string())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use pulsar_core::{Severity, SourceLocation};

  #[test]
  fn json_output() {
    let diag = Diagnostic {
      severity: Severity::Error,
      message: "test message".to_string(),
      location: SourceLocation {
        file: "test.ts".to_string(),
        line: 1,
        column: 5,
        span: Some((10, 20)),
      },
      rule_id: "test-rule".to_string(),
    };

    let formatter = JsonFormatter;
    let output = formatter.format(&[diag], "");
    assert!(output.contains("\"severity\": \"Error\""));
    assert!(output.contains("\"rule_id\": \"test-rule\""));
    assert!(output.contains("\"file\": \"test.ts\""));
    assert!(output.contains("\"line\": 1"));
  }

  #[test]
  fn json_empty() {
    let formatter = JsonFormatter;
    let output = formatter.format(&[], "");
    assert_eq!(output, "[]");
  }

  #[test]
  fn json_multiple_diagnostics() {
    let diags = vec![
      Diagnostic {
        severity: Severity::Error,
        message: "error msg".to_string(),
        location: SourceLocation {
          file: "a.ts".to_string(),
          line: 1,
          column: 5,
          span: Some((10, 20)),
        },
        rule_id: "rule-one".to_string(),
      },
      Diagnostic {
        severity: Severity::Warning,
        message: "warning msg".to_string(),
        location: SourceLocation { file: "b.ts".to_string(), line: 2, column: 10, span: None },
        rule_id: "rule-two".to_string(),
      },
    ];
    let formatter = JsonFormatter;
    let output = formatter.format(&diags, "");
    assert!(output.contains("\"severity\": \"Error\""));
    assert!(output.contains("\"severity\": \"Warning\""));
    assert!(output.contains("\"rule_id\": \"rule-one\""));
    assert!(output.contains("\"rule_id\": \"rule-two\""));
    assert!(output.contains("\"file\": \"a.ts\""));
    assert!(output.contains("\"file\": \"b.ts\""));
    assert!(output.contains("\"span\""));
  }

  #[test]
  fn json_info_severity() {
    let diag = Diagnostic {
      severity: Severity::Info,
      message: "info msg".to_string(),
      location: SourceLocation { file: "test.ts".to_string(), line: 3, column: 8, span: None },
      rule_id: "info-rule".to_string(),
    };
    let formatter = JsonFormatter;
    let output = formatter.format(&[diag], "");
    assert!(output.contains("\"severity\": \"Info\""));
    assert!(output.contains("\"line\": 3"));
    assert!(output.contains("\"column\": 8"));
  }

  #[test]
  fn json_span_in_output() {
    let diag = Diagnostic {
      severity: Severity::Error,
      message: "msg".to_string(),
      location: SourceLocation {
        file: "t.ts".to_string(),
        line: 1,
        column: 1,
        span: Some((5, 15)),
      },
      rule_id: "r".to_string(),
    };
    let formatter = JsonFormatter;
    let output = formatter.format(&[diag], "");
    assert!(output.contains("\"span\": ["));
    assert!(output.contains('5'));
    assert!(output.contains("15"));
  }
}
