use pulsar_core::{Diagnostic, Severity};
use pulsar_ir::NodeKind;

use crate::rule::{Rule, RuleContext};

pub struct NoMissingAwait;

impl Rule for NoMissingAwait {
  fn id(&self) -> &'static str {
    "no-missing-await"
  }

  fn docs(&self) -> &'static str {
    "Flags database queries that lack the `await` keyword.\n\
    \n\
    Drizzle queries return promises and must be awaited. Forgetting `await` \
    leads to race conditions, unhandled promise rejections, and data races."
  }

  fn run(&self, ctx: &RuleContext) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    for id in ctx.graph.node_indices() {
      if let NodeKind::Orm(orm) = ctx.graph.node(id).expect("node should exist") {
        if orm.missing_await {
          diags.push(Diagnostic {
            severity: Severity::Error,
            message: "Missing `await` — database queries must be awaited.".to_string(),
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

  fn make_graph(missing_await: bool) -> IrGraph {
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
      in_callback: false,
      missing_await,
      location,
    };

    graph.add_orm(orm);
    graph
  }

  #[test]
  fn flags_missing_await() {
    let graph = make_graph(true);
    let rule = NoMissingAwait;
    let ctx =
      RuleContext { graph: &graph, source_text: "", file_path: "test.ts", active_rules: &[] };
    let diags = rule.run(&ctx);
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].rule_id, "no-missing-await");
    assert_eq!(diags[0].severity, Severity::Error);
  }

  #[test]
  fn allows_query_with_await() {
    let graph = make_graph(false);
    let rule = NoMissingAwait;
    let ctx =
      RuleContext { graph: &graph, source_text: "", file_path: "test.ts", active_rules: &[] };
    let diags = rule.run(&ctx);
    assert!(diags.is_empty());
  }

  #[test]
  fn empty_graph_no_diagnostics() {
    let graph = IrGraph::new();
    let rule = NoMissingAwait;
    let ctx =
      RuleContext { graph: &graph, source_text: "", file_path: "test.ts", active_rules: &[] };
    let diags = rule.run(&ctx);
    assert!(diags.is_empty());
  }
}
