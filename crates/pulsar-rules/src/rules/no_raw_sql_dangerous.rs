use pulsar_core::{Diagnostic, Severity};
use pulsar_ir::NodeKind;

use crate::rule::{Rule, RuleContext};

pub struct NoRawSqlDangerous;

impl Rule for NoRawSqlDangerous {
  fn id(&self) -> &'static str {
    "no-raw-sql-dangerous"
  }

  fn docs(&self) -> &'static str {
    "Flags raw SQL usage detected in the codebase.\n\
    \n\
    Raw SQL bypasses the ORM's type safety, migration tracking, and query \
    building. Use Drizzle's query builder instead. Raw SQL with string \
    interpolation is especially dangerous (SQL injection risk)."
  }

  fn run(&self, ctx: &RuleContext) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    for id in ctx.graph.node_indices() {
      if let NodeKind::RawSql(raw) = ctx.graph.node(id).expect("node should exist") {
        let (severity, message) = if raw.has_interpolation {
          (
            Severity::Error,
            "Raw SQL with interpolation detected — SQL injection risk. Use parameterized queries instead."
              .to_string(),
          )
        } else {
          (
            Severity::Warning,
            "Raw SQL detected — prefer Drizzle query builder for type safety.".to_string(),
          )
        };
        diags.push(Diagnostic {
          severity,
          message,
          location: raw.location.clone(),
          rule_id: self.id().to_string(),
        });
      }
    }
    diags
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use pulsar_core::SourceLocation;
  use pulsar_ir::{IrGraph, RawSqlKind, RawSqlNode};

  fn make_graph(kind: RawSqlKind, has_interpolation: bool) -> IrGraph {
    let mut graph = IrGraph::new();
    let location = SourceLocation { file: "test.ts".to_string(), line: 1, column: 1, span: None };
    let raw = RawSqlNode { kind, has_interpolation, location };
    graph.add_raw_sql(raw);
    graph
  }

  #[test]
  fn flags_tagged_template_with_interpolation_as_error() {
    let graph = make_graph(RawSqlKind::TaggedTemplate, true);
    let rule = NoRawSqlDangerous;
    let ctx =
      RuleContext { graph: &graph, source_text: "", file_path: "test.ts", active_rules: &[] };
    let diags = rule.run(&ctx);
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].rule_id, "no-raw-sql-dangerous");
    assert_eq!(diags[0].severity, Severity::Error);
  }

  #[test]
  fn flags_tagged_template_without_interpolation_as_warning() {
    let graph = make_graph(RawSqlKind::TaggedTemplate, false);
    let rule = NoRawSqlDangerous;
    let ctx =
      RuleContext { graph: &graph, source_text: "", file_path: "test.ts", active_rules: &[] };
    let diags = rule.run(&ctx);
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].severity, Severity::Warning);
  }

  #[test]
  fn flags_db_raw_method_with_interpolation_as_error() {
    let graph = make_graph(RawSqlKind::DbRawMethod, true);
    let rule = NoRawSqlDangerous;
    let ctx =
      RuleContext { graph: &graph, source_text: "", file_path: "test.ts", active_rules: &[] };
    let diags = rule.run(&ctx);
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].severity, Severity::Error);
  }

  #[test]
  fn flags_db_raw_method_without_interpolation_as_warning() {
    let graph = make_graph(RawSqlKind::DbRawMethod, false);
    let rule = NoRawSqlDangerous;
    let ctx =
      RuleContext { graph: &graph, source_text: "", file_path: "test.ts", active_rules: &[] };
    let diags = rule.run(&ctx);
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].severity, Severity::Warning);
  }

  #[test]
  fn empty_graph_no_diagnostics() {
    let graph = IrGraph::new();
    let rule = NoRawSqlDangerous;
    let ctx =
      RuleContext { graph: &graph, source_text: "", file_path: "test.ts", active_rules: &[] };
    let diags = rule.run(&ctx);
    assert!(diags.is_empty());
  }
}
