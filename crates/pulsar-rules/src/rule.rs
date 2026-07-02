use pulsar_core::Diagnostic;
use pulsar_ir::IrGraph;

/// Context provided to every rule during execution.
pub struct RuleContext<'a> {
  /// The unified dependency graph.
  pub graph: &'a IrGraph,
  /// The original source text being analyzed.
  pub source_text: &'a str,
  /// The file path of the source being analyzed.
  pub file_path: &'a str,
}

/// A lint rule that inspects the IR graph and produces diagnostics.
pub trait Rule {
  /// Unique identifier for this rule (e.g. `"no-select-star"`).
  fn id(&self) -> &'static str;

  /// Human-readable documentation for the rule.
  fn docs(&self) -> &'static str {
    ""
  }

  /// Run the rule against the given context and return any diagnostics found.
  fn run(&self, ctx: &RuleContext) -> Vec<Diagnostic>;
}
