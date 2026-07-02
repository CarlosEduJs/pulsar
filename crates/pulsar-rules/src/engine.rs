use pulsar_core::Diagnostic;
use pulsar_ir::IrGraph;

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
  pub fn run(&self, graph: &IrGraph, source_text: &str, file_path: &str) -> Vec<Diagnostic> {
    let active_rules: Vec<String> = self.rules.iter().map(|r| r.id().to_string()).collect();
    let ctx = RuleContext { graph, source_text, file_path, active_rules: &active_rules };
    let mut all = Vec::new();
    for rule in &self.rules {
      all.extend(rule.run(&ctx));
    }
    all
  }
}

impl Default for RuleEngine {
  fn default() -> Self {
    Self::new()
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::rules::NoSelectStar;
  use pulsar_core::SourceLocation;
  use pulsar_ir::{ColumnRef, IrGraph, OrmArgs, OrmMethod, OrmNode, SQLNode, SqlKind, TableRef};

  fn graph_with_select_star() -> IrGraph {
    let mut graph = IrGraph::new();
    let loc = SourceLocation { file: "test.ts".to_string(), line: 1, column: 1, span: None };
    let sql = SQLNode {
      kind: SqlKind::Select,
      columns: vec![],
      table: Some(TableRef { name: "users".to_string(), alias: None }),
      limit: false,
      where_clause: false,
      location: loc.clone(),
    };
    let orm = OrmNode {
      method: OrmMethod::Select,
      args: OrmArgs { columns: vec![], where_clause: None, limit: None, include: vec![] },
      in_loop: false,
      location: loc,
    };
    let s = graph.add_sql(sql);
    let o = graph.add_orm(orm);
    graph.add_edge(o, s, pulsar_ir::EdgeKind::Generates);
    graph
  }

  fn graph_with_explicit_columns() -> IrGraph {
    let mut graph = IrGraph::new();
    let loc = SourceLocation { file: "test.ts".to_string(), line: 1, column: 1, span: None };
    let sql = SQLNode {
      kind: SqlKind::Select,
      columns: vec![ColumnRef { name: "id".to_string(), table: None }],
      table: Some(TableRef { name: "users".to_string(), alias: None }),
      limit: false,
      where_clause: false,
      location: loc.clone(),
    };
    let orm = OrmNode {
      method: OrmMethod::Select,
      args: OrmArgs {
        columns: vec!["id".to_string()],
        where_clause: None,
        limit: None,
        include: vec![],
      },
      in_loop: false,
      location: loc,
    };
    let s = graph.add_sql(sql);
    let o = graph.add_orm(orm);
    graph.add_edge(o, s, pulsar_ir::EdgeKind::Generates);
    graph
  }

  #[test]
  fn engine_with_no_rules() {
    let engine = RuleEngine::new();
    let graph = graph_with_select_star();
    let diags = engine.run(&graph, "", "test.ts");
    assert!(diags.is_empty());
  }

  #[test]
  fn engine_with_no_rules_default() {
    let engine = RuleEngine::default();
    let graph = graph_with_select_star();
    let diags = engine.run(&graph, "", "test.ts");
    assert!(diags.is_empty());
  }

  #[test]
  fn engine_detects_select_star() {
    let mut engine = RuleEngine::new();
    engine.register(Box::new(NoSelectStar));
    let graph = graph_with_select_star();
    let diags = engine.run(&graph, "", "test.ts");
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].rule_id, "no-select-star");
  }

  #[test]
  fn engine_skips_explicit_columns() {
    let mut engine = RuleEngine::new();
    engine.register(Box::new(NoSelectStar));
    let graph = graph_with_explicit_columns();
    let diags = engine.run(&graph, "", "test.ts");
    assert!(diags.is_empty());
  }

  #[test]
  fn engine_on_empty_graph() {
    let mut engine = RuleEngine::new();
    engine.register(Box::new(NoSelectStar));
    let graph = IrGraph::new();
    let diags = engine.run(&graph, "", "test.ts");
    assert!(diags.is_empty());
  }
}
