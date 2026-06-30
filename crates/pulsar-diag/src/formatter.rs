use pulsar_core::Diagnostic;

/// Formats diagnostics for output.
pub trait DiagnosticFormatter {
  /// Formats a slice of diagnostics along with the original source text.
  fn format(&self, diagnostics: &[Diagnostic], source_text: &str) -> String;
}
