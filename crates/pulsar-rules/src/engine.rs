use pulsar_core::Diagnostic;

use crate::rule::{Rule, RuleContext};

/// Orchestrates rule execution against the IR graph.
pub struct RuleEngine {
  rules: Vec<Box<dyn Rule>>,
}

impl RuleEngine {
  /// Creates a new empty rule engine.
  #[must_use]
  pub fn new() -> Self {
    Self { rules: Vec::new() }
  }

  /// Registers a rule for execution.
  pub fn register(&mut self, rule: Box<dyn Rule>) {
    self.rules.push(rule);
  }

  /// Runs all registered rules and returns all collected diagnostics.
  #[must_use]
  pub fn run(&self, ctx: &RuleContext) -> Vec<Diagnostic> {
    let mut all = Vec::new();
    for rule in &self.rules {
      all.extend(rule.run(ctx));
    }
    all
  }
}

impl Default for RuleEngine {
  fn default() -> Self {
    Self::new()
  }
}
