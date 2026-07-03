use pulsar_core::{Diagnostic, Severity};
use pulsar_ir::NodeKind;

use crate::rule::{Rule, RuleContext};

pub struct NoQueryInCallback;

impl Rule for NoQueryInCallback {
  fn id(&self) -> &'static str {
    "no-query-in-callback"
  }

  fn docs(&self) -> &'static str {
    "Flags database queries executed inside callbacks.\n\
    \n\
    Running queries inside callbacks (e.g. .then(), .catch(), .finally(), \
    setTimeout, setInterval, .map(), .filter(), .forEach()) often leads \
    to unintended behavior and N+1 problems. Extract the query outside \
    the callback."
  }

  fn run(&self, ctx: &RuleContext) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    for id in ctx.graph.node_indices() {
      match ctx.graph.node(id).expect("node should exist") {
        NodeKind::Orm(orm) if orm.in_callback => {
          diags.push(Diagnostic {
            severity: Severity::Warning,
            message:
              "Database query inside a callback — extract it outside to avoid unintended behavior."
                .to_string(),
            location: orm.location.clone(),
            rule_id: self.id().to_string(),
          });
        }
        NodeKind::Sql(sql) if sql.in_callback => {
          diags.push(Diagnostic {
            severity: Severity::Warning,
            message:
              "SQL query inside a callback — extract it outside to avoid unintended behavior."
                .to_string(),
            location: sql.location.clone(),
            rule_id: self.id().to_string(),
          });
        }
        _ => {}
      }
    }
    diags
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use pulsar_core::SourceLocation;
  use pulsar_ir::{IrGraph, LoopKind, OrmArgs, OrmMethod, OrmNode, SQLNode, SqlKind, TableRef};

  fn make_orm_graph(in_callback: bool) -> IrGraph {
    let mut graph = IrGraph::new();
    let location = SourceLocation { file: "test.ts".to_string(), line: 1, column: 1, span: None };
    let orm = OrmNode {
      method: OrmMethod::Select,
      args: OrmArgs {
        columns: vec!["id".to_string()],
        where_clause: Some("eq(users.id, 1)".to_string()),
        limit: Some(1),
        include: Vec::new(),
      },
      loop_kind: LoopKind::None,
      in_callback,
      missing_await: false,
      location,
    };
    graph.add_orm(orm);
    graph
  }

  fn make_sql_graph(in_callback: bool) -> IrGraph {
    let mut graph = IrGraph::new();
    let location = SourceLocation { file: "test.ts".to_string(), line: 1, column: 1, span: None };
    let sql = SQLNode {
      kind: SqlKind::Select,
      columns: vec![],
      table: Some(TableRef { name: "users".to_string(), alias: None }),
      limit: false,
      where_clause: false,
      in_callback,
      location,
    };
    graph.add_sql(sql);
    graph
  }

  #[test]
  fn flags_orm_in_callback() {
    let graph = make_orm_graph(true);
    let rule = NoQueryInCallback;
    let ctx =
      RuleContext { graph: &graph, source_text: "", file_path: "test.ts", active_rules: &[] };
    let diags = rule.run(&ctx);
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].rule_id, "no-query-in-callback");
    assert_eq!(diags[0].severity, Severity::Warning);
  }

  #[test]
  fn flags_sql_in_callback() {
    let graph = make_sql_graph(true);
    let rule = NoQueryInCallback;
    let ctx =
      RuleContext { graph: &graph, source_text: "", file_path: "test.ts", active_rules: &[] };
    let diags = rule.run(&ctx);
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].rule_id, "no-query-in-callback");
  }

  #[test]
  fn allows_orm_outside_callback() {
    let graph = make_orm_graph(false);
    let rule = NoQueryInCallback;
    let ctx =
      RuleContext { graph: &graph, source_text: "", file_path: "test.ts", active_rules: &[] };
    let diags = rule.run(&ctx);
    assert!(diags.is_empty());
  }

  #[test]
  fn allows_sql_outside_callback() {
    let graph = make_sql_graph(false);
    let rule = NoQueryInCallback;
    let ctx =
      RuleContext { graph: &graph, source_text: "", file_path: "test.ts", active_rules: &[] };
    let diags = rule.run(&ctx);
    assert!(diags.is_empty());
  }

  #[test]
  fn flags_both_when_present() {
    let mut graph = IrGraph::new();
    let loc1 = SourceLocation { file: "test.ts".to_string(), line: 1, column: 1, span: None };
    let loc2 = SourceLocation { file: "test.ts".to_string(), line: 2, column: 1, span: None };
    let orm = OrmNode {
      method: OrmMethod::Select,
      args: OrmArgs {
        columns: vec!["id".to_string()],
        where_clause: None,
        limit: None,
        include: Vec::new(),
      },
      loop_kind: LoopKind::None,
      in_callback: true,
      missing_await: false,
      location: loc1,
    };
    let sql = SQLNode {
      kind: SqlKind::Select,
      columns: vec![],
      table: Some(TableRef { name: "users".to_string(), alias: None }),
      limit: false,
      where_clause: false,
      in_callback: true,
      location: loc2,
    };
    graph.add_orm(orm);
    graph.add_sql(sql);
    let rule = NoQueryInCallback;
    let ctx =
      RuleContext { graph: &graph, source_text: "", file_path: "test.ts", active_rules: &[] };
    let diags = rule.run(&ctx);
    assert_eq!(diags.len(), 2);
  }

  #[test]
  fn empty_graph_no_diagnostics() {
    let graph = IrGraph::new();
    let rule = NoQueryInCallback;
    let ctx =
      RuleContext { graph: &graph, source_text: "", file_path: "test.ts", active_rules: &[] };
    let diags = rule.run(&ctx);
    assert!(diags.is_empty());
  }
}
