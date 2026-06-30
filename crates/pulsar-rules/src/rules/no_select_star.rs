use pulsar_core::{Diagnostic, Severity};
use pulsar_ir::NodeKind;

use crate::rule::{Rule, RuleContext};

/// Flags `SELECT *` queries — both implicit (empty column list) and explicit wildcards.
///
/// Using `SELECT *` makes queries fragile and often fetches more data than needed.
/// Always specify the exact columns required.
pub struct NoSelectStar;

impl Rule for NoSelectStar {
  fn id(&self) -> &'static str {
    "no-select-star"
  }

  fn run(&self, ctx: &RuleContext) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    for id in ctx.graph.node_indices() {
      if let NodeKind::Sql(sql) = ctx.graph.node(id).expect("node should exist") {
        if sql.is_select_star() {
          diags.push(Diagnostic {
            severity: Severity::Error,
            message: "Avoid implicit SELECT *. Specify columns explicitly.".to_string(),
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
  use pulsar_ir::{IrGraph, OrmArgs, OrmMethod, OrmNode, SQLNode, SqlKind, TableRef};

  fn make_graph(with_columns: bool) -> IrGraph {
    let mut graph = IrGraph::new();
    let location = SourceLocation { file: "test.ts".to_string(), line: 5, column: 10, span: None };

    let columns = if with_columns {
      vec![pulsar_ir::ColumnRef { name: "id".to_string(), table: None }]
    } else {
      vec![]
    };

    let sql = SQLNode {
      kind: SqlKind::Select,
      columns,
      table: Some(TableRef { name: "users".to_string(), alias: None }),
      limit: false,
      where_clause: false,
      location: location.clone(),
    };

    let orm = OrmNode {
      method: OrmMethod::Select,
      args: OrmArgs { columns: Vec::new(), where_clause: None, limit: None, include: Vec::new() },
      location,
    };

    let sql_id = graph.add_sql(sql);
    let orm_id = graph.add_orm(orm);
    graph.add_edge(orm_id, sql_id, pulsar_ir::EdgeKind::Generates);
    graph
  }

  #[test]
  fn detects_select_star() {
    let graph = make_graph(false);
    let rule = NoSelectStar;
    let ctx = RuleContext { graph: &graph, source_text: "", file_path: "test.ts" };
    let diags = rule.run(&ctx);
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].rule_id, "no-select-star");
    assert_eq!(diags[0].severity, Severity::Error);
  }

  #[test]
  fn allows_explicit_columns() {
    let graph = make_graph(true);
    let rule = NoSelectStar;
    let ctx = RuleContext { graph: &graph, source_text: "", file_path: "test.ts" };
    let diags = rule.run(&ctx);
    assert_eq!(diags.len(), 0);
  }
}
