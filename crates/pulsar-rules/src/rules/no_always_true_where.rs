use pulsar_core::{Diagnostic, Severity};
use pulsar_ir::NodeKind;

use crate::rule::{Rule, RuleContext};

/// Flags `.where(true)` clauses which are always-true no-ops.
///
/// A WHERE clause with a literal `true` value has no filtering effect
/// and is likely a mistake (e.g. a forgotten placeholder or debug code).
pub struct NoAlwaysTrueWhere;

impl Rule for NoAlwaysTrueWhere {
  fn id(&self) -> &'static str {
    "no-always-true-where"
  }

  fn docs(&self) -> &'static str {
    "Flags `.where(true)` clauses which are always-true no-ops.\n\
    \n\
    A WHERE clause with a literal `true` value has no filtering effect \
    and is likely a mistake (e.g. a forgotten placeholder or debug code)."
  }

  fn run(&self, ctx: &RuleContext) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    for id in ctx.graph.node_indices() {
      if let NodeKind::Orm(orm) = ctx.graph.node(id).expect("node should exist") {
        if let Some(ref where_clause) = orm.args.where_clause {
          if where_clause == "true" {
            diags.push(Diagnostic {
              severity: Severity::Error,
              message:
                ".where(true) has no filtering effect — remove it or provide a real condition."
                  .to_string(),
              location: orm.location.clone(),
              rule_id: self.id().to_string(),
            });
          }
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

  fn make_graph(where_clause: Option<&str>) -> IrGraph {
    let mut graph = IrGraph::new();
    let location = SourceLocation { file: "test.ts".to_string(), line: 1, column: 1, span: None };

    let orm = OrmNode {
      method: OrmMethod::Select,
      args: OrmArgs {
        columns: vec!["id".to_string()],
        where_clause: where_clause.map(ToString::to_string),
        limit: None,
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
  fn flags_true_where() {
    let graph = make_graph(Some("true"));
    let rule = NoAlwaysTrueWhere;
    let ctx =
      RuleContext { graph: &graph, source_text: "", file_path: "test.ts", active_rules: &[] };
    let diags = rule.run(&ctx);
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].rule_id, "no-always-true-where");
    assert_eq!(diags[0].severity, Severity::Error);
  }

  #[test]
  fn allows_real_condition() {
    let graph = make_graph(Some("eq(users.id, 1)"));
    let rule = NoAlwaysTrueWhere;
    let ctx =
      RuleContext { graph: &graph, source_text: "", file_path: "test.ts", active_rules: &[] };
    let diags = rule.run(&ctx);
    assert_eq!(diags.len(), 0);
  }

  #[test]
  fn allows_no_where() {
    let graph = make_graph(None);
    let rule = NoAlwaysTrueWhere;
    let ctx =
      RuleContext { graph: &graph, source_text: "", file_path: "test.ts", active_rules: &[] };
    let diags = rule.run(&ctx);
    assert_eq!(diags.len(), 0);
  }

  #[test]
  fn empty_graph_no_diagnostics() {
    let graph = IrGraph::new();
    let rule = NoAlwaysTrueWhere;
    let ctx =
      RuleContext { graph: &graph, source_text: "", file_path: "test.ts", active_rules: &[] };
    let diags = rule.run(&ctx);
    assert!(diags.is_empty());
  }
}
