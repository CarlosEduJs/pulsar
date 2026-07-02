use pulsar_core::{Diagnostic, Severity};
use pulsar_ir::{NodeKind, OrmMethod};

use crate::rule::{Rule, RuleContext};

/// Flags ORM find calls that lack both `.where()` and `.limit()`.
///
/// Unbounded queries (no filter, no limit) can return the entire table,
/// which is almost never intended and often a bug or performance hazard.
pub struct NoUnboundedFind;

impl Rule for NoUnboundedFind {
  fn id(&self) -> &'static str {
    "no-unbounded-find"
  }

  fn docs(&self) -> &'static str {
    "Flags ORM queries that lack both a WHERE clause and a LIMIT.\n\
    \n\
    Unbounded queries can return the entire table, which is almost never \
    intended and often a bug or performance hazard. \
    Always add a .where() filter or a .limit() bound."
  }

  fn run(&self, ctx: &RuleContext) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    for id in ctx.graph.node_indices() {
      if let NodeKind::Orm(orm) = ctx.graph.node(id).expect("node should exist") {
        if !matches!(orm.method, OrmMethod::Select | OrmMethod::FindMany | OrmMethod::FindFirst) {
          continue;
        }
        let has_where = orm.args.where_clause.is_some();
        let has_limit = orm.args.limit.is_some();
        if !has_where && !has_limit {
          diags.push(Diagnostic {
            severity: Severity::Warning,
            message: "Query is unbounded — add a .where() filter or a .limit() bound.".to_string(),
            location: orm.location.clone(),
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
  use pulsar_ir::{IrGraph, LoopKind, OrmArgs, OrmMethod, OrmNode};

  fn make_graph(has_where: bool, has_limit: bool) -> IrGraph {
    let mut graph = IrGraph::new();
    let location = SourceLocation { file: "test.ts".to_string(), line: 1, column: 1, span: None };

    let orm = OrmNode {
      method: OrmMethod::Select,
      args: OrmArgs {
        columns: vec!["id".to_string()],
        where_clause: if has_where { Some("eq(users.id, 1)".to_string()) } else { None },
        limit: if has_limit { Some(10) } else { None },
        include: Vec::new(),
      },
      loop_kind: LoopKind::None,
      in_callback: false,
      location,
    };

    graph.add_orm(orm);
    graph
  }

  #[test]
  fn flags_no_where_no_limit() {
    let graph = make_graph(false, false);
    let rule = NoUnboundedFind;
    let ctx =
      RuleContext { graph: &graph, source_text: "", file_path: "test.ts", active_rules: &[] };
    let diags = rule.run(&ctx);
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].rule_id, "no-unbounded-find");
    assert_eq!(diags[0].severity, Severity::Warning);
  }

  #[test]
  fn allows_where_only() {
    let graph = make_graph(true, false);
    let rule = NoUnboundedFind;
    let ctx =
      RuleContext { graph: &graph, source_text: "", file_path: "test.ts", active_rules: &[] };
    let diags = rule.run(&ctx);
    assert_eq!(diags.len(), 0);
  }

  #[test]
  fn allows_limit_only() {
    let graph = make_graph(false, true);
    let rule = NoUnboundedFind;
    let ctx =
      RuleContext { graph: &graph, source_text: "", file_path: "test.ts", active_rules: &[] };
    let diags = rule.run(&ctx);
    assert_eq!(diags.len(), 0);
  }

  #[test]
  fn allows_where_and_limit() {
    let graph = make_graph(true, true);
    let rule = NoUnboundedFind;
    let ctx =
      RuleContext { graph: &graph, source_text: "", file_path: "test.ts", active_rules: &[] };
    let diags = rule.run(&ctx);
    assert_eq!(diags.len(), 0);
  }

  #[test]
  fn allows_insert_without_where_or_limit() {
    let mut graph = IrGraph::new();
    let location = SourceLocation { file: "test.ts".to_string(), line: 1, column: 1, span: None };
    let orm = OrmNode {
      method: OrmMethod::Insert,
      args: OrmArgs { columns: vec![], where_clause: None, limit: None, include: Vec::new() },
      loop_kind: LoopKind::None,
      in_callback: false,
      location,
    };
    graph.add_orm(orm);
    let rule = NoUnboundedFind;
    let ctx =
      RuleContext { graph: &graph, source_text: "", file_path: "test.ts", active_rules: &[] };
    let diags = rule.run(&ctx);
    assert_eq!(diags.len(), 0, "Insert should not be flagged as unbounded");
  }

  #[test]
  fn allows_update_without_where_or_limit() {
    let mut graph = IrGraph::new();
    let location = SourceLocation { file: "test.ts".to_string(), line: 1, column: 1, span: None };
    let orm = OrmNode {
      method: OrmMethod::Update,
      args: OrmArgs { columns: vec![], where_clause: None, limit: None, include: Vec::new() },
      loop_kind: LoopKind::None,
      in_callback: false,
      location,
    };
    graph.add_orm(orm);
    let rule = NoUnboundedFind;
    let ctx =
      RuleContext { graph: &graph, source_text: "", file_path: "test.ts", active_rules: &[] };
    let diags = rule.run(&ctx);
    assert_eq!(diags.len(), 0, "Update should not be flagged as unbounded");
  }

  #[test]
  fn empty_graph_no_diagnostics() {
    let graph = IrGraph::new();
    let rule = NoUnboundedFind;
    let ctx =
      RuleContext { graph: &graph, source_text: "", file_path: "test.ts", active_rules: &[] };
    let diags = rule.run(&ctx);
    assert!(diags.is_empty());
  }
}
