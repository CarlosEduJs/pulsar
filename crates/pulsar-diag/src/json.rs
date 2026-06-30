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
}
