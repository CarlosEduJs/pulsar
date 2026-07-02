use pulsar_core::{Diagnostic, Severity};
use pulsar_ir::NodeKind;

use crate::rule::{Rule, RuleContext};

/// Flags SQL SELECT queries without a LIMIT clause.
///
/// Queries without a LIMIT can return unbounded result sets,
/// leading to performance issues and excessive memory usage.
pub struct NoMissingLimit;

impl Rule for NoMissingLimit {
  fn id(&self) -> &'static str {
    "no-missing-limit"
  }

  fn docs(&self) -> &'static str {
    "Flags SQL SELECT queries without a LIMIT clause.\n\
    \n\
    Queries without a LIMIT can return unbounded result sets, \
    leading to performance issues and excessive memory usage."
  }

  fn run(&self, ctx: &RuleContext) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    for id in ctx.graph.node_indices() {
      if let NodeKind::Sql(sql) = ctx.graph.node(id).expect("node should exist") {
        if !sql.limit {
          diags.push(Diagnostic {
            severity: Severity::Warning,
            message: "Query is missing a LIMIT clause.".to_string(),
            location: sql.location.clone(),
            rule_id: self.id().to_string(),
          });
        }
      }
    }
    diags
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use pulsar_core::SourceLocation;
  use pulsar_ir::{ColumnRef, IrGraph, OrmArgs, OrmMethod, OrmNode, SQLNode, SqlKind, TableRef};

  fn make_graph(has_limit: bool) -> IrGraph {
    let mut graph = IrGraph::new();
    let location = SourceLocation { file: "test.ts".to_string(), line: 1, column: 1, span: None };

    let sql = SQLNode {
      kind: SqlKind::Select,
      columns: vec![ColumnRef { name: "id".to_string(), table: None }],
      table: Some(TableRef { name: "users".to_string(), alias: None }),
      limit: has_limit,
      where_clause: false,
      location: location.clone(),
    };

    let orm = OrmNode {
      method: OrmMethod::Select,
      args: OrmArgs {
        columns: vec!["id".to_string()],
        where_clause: None,
        limit: if has_limit { Some(10) } else { None },
        include: Vec::new(),
      },
      in_loop: false,
      location,
    };

    let sql_id = graph.add_sql(sql);
    let orm_id = graph.add_orm(orm);
    graph.add_edge(orm_id, sql_id, pulsar_ir::EdgeKind::Generates);
    graph
  }

  #[test]
  fn flags_missing_limit() {
    let graph = make_graph(false);
    let rule = NoMissingLimit;
    let ctx =
      RuleContext { graph: &graph, source_text: "", file_path: "test.ts", active_rules: &[] };
    let diags = rule.run(&ctx);
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].rule_id, "no-missing-limit");
    assert_eq!(diags[0].severity, Severity::Warning);
  }

  #[test]
  fn allows_limit_present() {
    let graph = make_graph(true);
    let rule = NoMissingLimit;
    let ctx =
      RuleContext { graph: &graph, source_text: "", file_path: "test.ts", active_rules: &[] };
    let diags = rule.run(&ctx);
    assert_eq!(diags.len(), 0);
  }

  #[test]
  fn empty_graph_no_diagnostics() {
    let graph = IrGraph::new();
    let rule = NoMissingLimit;
    let ctx =
      RuleContext { graph: &graph, source_text: "", file_path: "test.ts", active_rules: &[] };
    let diags = rule.run(&ctx);
    assert!(diags.is_empty());
  }
}
