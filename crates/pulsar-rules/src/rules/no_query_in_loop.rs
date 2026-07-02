use pulsar_core::{Diagnostic, Severity};
use pulsar_ir::NodeKind;

use crate::rule::{Rule, RuleContext};

/// Flags database queries executed inside loops.
///
/// Running queries inside a loop (for, while, do-while, for-in, for-of)
/// causes N+1 query problems and significant performance degradation.
/// Prefer batch queries outside the loop.
pub struct NoQueryInLoop;

impl Rule for NoQueryInLoop {
  fn id(&self) -> &'static str {
    "no-query-in-loop"
  }

  fn docs(&self) -> &'static str {
    "Flags database queries executed inside loops.\n\
    \n\
    Running queries inside a loop (for, while, do-while, for-in, for-of) \
    causes N+1 query problems and significant performance degradation. \
    Prefer batch queries or collect the data first, then query once."
  }

  fn run(&self, ctx: &RuleContext) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    for id in ctx.graph.node_indices() {
      if let NodeKind::Orm(orm) = ctx.graph.node(id).expect("node should exist") {
        if orm.in_loop {
          diags.push(Diagnostic {
            severity: Severity::Error,
            message: "Database query inside a loop — extract it outside to avoid N+1 queries."
              .to_string(),
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
  use pulsar_ir::{IrGraph, OrmArgs, OrmMethod, OrmNode};

  fn make_graph(in_loop: bool) -> IrGraph {
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
      in_loop,
      location,
    };

    graph.add_orm(orm);
    graph
  }

  #[test]
  fn flags_query_in_loop() {
    let graph = make_graph(true);
    let rule = NoQueryInLoop;
    let ctx = RuleContext { graph: &graph, source_text: "", file_path: "test.ts" };
    let diags = rule.run(&ctx);
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].rule_id, "no-query-in-loop");
    assert_eq!(diags[0].severity, Severity::Error);
  }

  #[test]
  fn allows_query_outside_loop() {
    let graph = make_graph(false);
    let rule = NoQueryInLoop;
    let ctx = RuleContext { graph: &graph, source_text: "", file_path: "test.ts" };
    let diags = rule.run(&ctx);
    assert_eq!(diags.len(), 0);
  }

  #[test]
  fn empty_graph_no_diagnostics() {
    let graph = IrGraph::new();
    let rule = NoQueryInLoop;
    let ctx = RuleContext { graph: &graph, source_text: "", file_path: "test.ts" };
    let diags = rule.run(&ctx);
    assert!(diags.is_empty());
  }
}
