use std::fmt::Write;

use owo_colors::OwoColorize;
use pulsar_core::{Diagnostic, Severity};

use super::formatter::DiagnosticFormatter;

/// Formats diagnostics in a human-readable, ESLint-inspired style with source context.
///
/// Output example:
/// ```text
///   test.ts:5:10  error  no-select-star  Avoid implicit SELECT *.
///
///     const users = await db.select().from(users)
///                        ^^^^^^
///
/// ✖ 1 problem (1 error, 0 warnings, 0 infos)
/// ```
pub struct PrettyFormatter;

impl DiagnosticFormatter for PrettyFormatter {
  fn format(&self, diagnostics: &[Diagnostic], source_text: &str) -> String {
    let mut out = String::new();

    for diag in diagnostics {
      let _ = write!(
        out,
        "  {}  {}  {}  {}\n\n",
        format_location(diag),
        format_severity(diag.severity),
        diag.rule_id.dimmed(),
        diag.message,
      );

      if let Some(line) = get_line(source_text, diag.location.line) {
        let _ = writeln!(out, "    {line}");

        if let Some((start, end)) = diag.location.span {
          let col = diag.location.column;
          if end > start && col > 0 {
            let width = end.saturating_sub(start);
            let padding = " ".repeat(col.saturating_sub(1));
            let underline = "^".repeat(width.max(1));
            let _ = writeln!(out, "    {padding}{}", underline.red());
          }
        }
      }

      out.push('\n');
    }

    // Summary
    if !diagnostics.is_empty() {
      let (errors, warnings, infos) = count_by_severity(diagnostics);
      let total = diagnostics.len();
      let problem_word = if total == 1 { "problem" } else { "problems" };
      let _ = write!(
        out,
        "{} {} {} ({} {}",
        "✖".red(),
        total,
        problem_word,
        errors,
        if errors == 1 { "error" } else { "errors" },
      );
      let _ = write!(out, ", {} {}", warnings, if warnings == 1 { "warning" } else { "warnings" });
      let _ = write!(out, ", {} {}", infos, if infos == 1 { "info" } else { "infos" });
      let _ = writeln!(out, ")");
    }

    out
  }
}

fn format_location(diag: &Diagnostic) -> String {
  format!("{}:{}:{}", diag.location.file.bold(), diag.location.line, diag.location.column)
}

fn format_severity(severity: Severity) -> String {
  match severity {
    Severity::Error => "error".red().to_string(),
    Severity::Warning => "warning".yellow().to_string(),
    Severity::Info => "info".blue().to_string(),
  }
}

fn get_line(source: &str, line: usize) -> Option<&str> {
  // line is 1-based
  if line == 0 {
    return None;
  }
  source.lines().nth(line - 1)
}

fn count_by_severity(diagnostics: &[Diagnostic]) -> (usize, usize, usize) {
  diagnostics.iter().fold((0, 0, 0), |(e, w, i), d| match d.severity {
    Severity::Error => (e + 1, w, i),
    Severity::Warning => (e, w + 1, i),
    Severity::Info => (e, w, i + 1),
  })
}

#[cfg(test)]
mod tests {
  use super::*;
  use pulsar_core::SourceLocation;

  /// Strips ANSI escape codes for test assertions.
  fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_escape = false;
    for c in s.chars() {
      if in_escape {
        if c == 'm' {
          in_escape = false;
        }
      } else if c == '\x1b' {
        in_escape = true;
      } else {
        out.push(c);
      }
    }
    out
  }

  fn make_diag(
    line: usize,
    column: usize,
    span: Option<(usize, usize)>,
    severity: Severity,
  ) -> Diagnostic {
    Diagnostic {
      severity,
      message: "Avoid implicit SELECT *.".to_string(),
      location: SourceLocation { file: "test.ts".to_string(), line, column, span },
      rule_id: "no-select-star".to_string(),
    }
  }

  #[test]
  fn pretty_with_source_context() {
    let source = "const x = 1;\nconst users = await db.select().from(users);\nconst y = 2;\n";
    let diag = make_diag(2, 12, Some((29, 35)), Severity::Error);
    let formatter = PrettyFormatter;
    let output = formatter.format(&[diag], source);
    let plain = strip_ansi(&output);

    assert!(plain.contains("test.ts:2:12"), "location in output:\n{plain}");
    assert!(plain.contains("error"), "severity in output:\n{plain}");
    assert!(plain.contains("no-select-star"), "rule_id in output:\n{plain}");
    assert!(plain.contains("const users = await db.select().from(users)"));
    assert!(plain.contains("^^^^^^"), "underline in output:\n{plain}");
    assert!(plain.contains("✖"), "summary in output:\n{plain}");
    assert!(plain.contains("1 problem"), "count in output:\n{plain}");
  }

  #[test]
  fn pretty_no_diagnostics() {
    let formatter = PrettyFormatter;
    let output = formatter.format(&[], "");
    assert!(output.is_empty());
  }

  #[test]
  fn pretty_multiple_severities() {
    let source = "a\nb\nc\n";
    let diags =
      vec![make_diag(1, 1, None, Severity::Error), make_diag(2, 1, None, Severity::Warning)];
    let formatter = PrettyFormatter;
    let output = formatter.format(&diags, source);
    let plain = strip_ansi(&output);
    assert!(plain.contains("2 problems"), "count in output:\n{plain}");
    assert!(plain.contains("1 error"), "error in output:\n{plain}");
    assert!(plain.contains("1 warning"), "warning in output:\n{plain}");
  }

  #[test]
  fn pretty_single_problem_grammar() {
    let source = "a\n";
    let diag = make_diag(1, 1, None, Severity::Error);
    let formatter = PrettyFormatter;
    let output = formatter.format(&[diag], source);
    let plain = strip_ansi(&output);
    assert!(plain.contains("1 problem"), "grammar in output:\n{plain}");
    assert!(plain.contains("1 error"), "error in output:\n{plain}");
  }

  #[test]
  fn pretty_info_severity() {
    let source = "a\n";
    let diag = make_diag(1, 1, None, Severity::Info);
    let formatter = PrettyFormatter;
    let output = formatter.format(&[diag], source);
    let plain = strip_ansi(&output);
    assert!(plain.contains("info"), "severity in output:\n{plain}");
    assert!(plain.contains("1 info"), "count in output:\n{plain}");
  }

  #[test]
  fn pretty_warning_severity() {
    let source = "a\n";
    let diag = make_diag(1, 1, None, Severity::Warning);
    let formatter = PrettyFormatter;
    let output = formatter.format(&[diag], source);
    let plain = strip_ansi(&output);
    assert!(plain.contains("warning"), "severity in output:\n{plain}");
    assert!(plain.contains("1 warning"), "count in output:\n{plain}");
  }

  #[test]
  fn pretty_span_at_column_zero() {
    // Column 0 means no underline is drawn (col > 0 check)
    let source = "line one\n";
    let diag = make_diag(1, 0, Some((0, 4)), Severity::Error);
    let formatter = PrettyFormatter;
    let output = formatter.format(&[diag], source);
    let plain = strip_ansi(&output);
    assert!(plain.contains("error"), "severity in output:\n{plain}");
    // underline should NOT be present since col == 0
    assert!(!plain.contains("^^^^"), "no underline at col 0:\n{plain}");
  }

  #[test]
  fn pretty_line_out_of_range() {
    // Line 100 in a 2-line file — no source context shown
    let source = "a\nb\n";
    let diag = make_diag(100, 1, None, Severity::Error);
    let formatter = PrettyFormatter;
    let output = formatter.format(&[diag], source);
    let plain = strip_ansi(&output);
    assert!(plain.contains("error"), "severity in output:\n{plain}");
    assert!(!plain.contains("a\n"), "no source line for out-of-range");
  }

  #[test]
  fn pretty_line_zero_returns_none() {
    // line == 0 should be skipped — get_line returns None
    let source = "a\nb\n";
    let diag = make_diag(0, 1, None, Severity::Error);
    let formatter = PrettyFormatter;
    let output = formatter.format(&[diag], source);
    let plain = strip_ansi(&output);
    assert!(plain.contains("error"), "severity in output:\n{plain}");
    // Should not crash — just no source context
  }

  #[test]
  fn pretty_without_span() {
    let source = "const x = 1;\n";
    let diag = make_diag(1, 5, None, Severity::Error);
    let formatter = PrettyFormatter;
    let output = formatter.format(&[diag], source);
    let plain = strip_ansi(&output);
    assert!(plain.contains("error"), "severity in output:\n{plain}");
    assert!(plain.contains("const x = 1;"), "source line in output:\n{plain}");
    // No underline since span is None
    assert!(!plain.contains("^^^^"), "no underline without span:\n{plain}");
  }

  #[test]
  fn pretty_all_severities_in_summary() {
    let source = "a\nb\nc\n";
    let diags = vec![
      make_diag(1, 1, None, Severity::Error),
      make_diag(2, 1, None, Severity::Warning),
      make_diag(3, 1, None, Severity::Info),
    ];
    let formatter = PrettyFormatter;
    let output = formatter.format(&diags, source);
    let plain = strip_ansi(&output);
    assert!(plain.contains("3 problems"), "count in output:\n{plain}");
    assert!(plain.contains("1 error"), "error in output:\n{plain}");
    assert!(plain.contains("1 warning"), "warning in output:\n{plain}");
    assert!(plain.contains("1 info"), "info in output:\n{plain}");
  }

  #[test]
  fn pretty_empty_source_text_with_diagnostics() {
    let diag = make_diag(1, 1, None, Severity::Error);
    let formatter = PrettyFormatter;
    // Source text is empty but line is 1 — get_line returns None
    let output = formatter.format(&[diag], "");
    let plain = strip_ansi(&output);
    assert!(plain.contains("error"), "severity in output:\n{plain}");
    // No source context since source is empty
  }

  // Regression: Bug #10 — Windows-style line endings (\\r\\n) are handled correctly by Rust's lines()
  #[test]
  fn pretty_with_windows_line_endings() {
    let source = "line one\r\nline two\r\nline three\r\n";
    let diag = make_diag(2, 1, None, Severity::Error);
    let formatter = PrettyFormatter;
    let output = formatter.format(&[diag], source);
    let plain = strip_ansi(&output);
    // Rust's str::lines() properly strips both \\r and \\n for CRLF endings
    assert!(
      plain.contains("line two"),
      "should display 'line two' without \\r suffix, got:\n{plain}"
    );
    assert!(
      !plain.contains("line two\r"),
      "Windows \\r\\n should be handled correctly by Rust's lines() — no \\r in output"
    );
  }

  // Regression: Bug #10 — mixed line endings
  #[test]
  fn pretty_with_mixed_line_endings() {
    let source = "unix line\nwindows line\r\n";
    let diag2 = make_diag(2, 1, None, Severity::Error);
    let formatter = PrettyFormatter;
    let output = formatter.format(&[diag2], source);
    let plain = strip_ansi(&output);
    assert!(plain.contains("windows line"), "should display 'windows line' without \\r suffix");
    assert!(!plain.contains("windows line\r"), "Windows \\r\\n should be handled correctly");
  }
}
