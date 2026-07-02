use pulsar_core::{Diagnostic, Severity};
use pulsar_ir::{LoopKind, NodeKind};

use crate::rule::{Rule, RuleContext};

pub struct NoNPlusOne;

impl Rule for NoNPlusOne {
  fn id(&self) -> &'static str {
    "no-n-plus-one"
  }

  fn docs(&self) -> &'static str {
    "Flags database queries inside iteration loops (for-of, for-in).\n\
    \n\
    Queries inside for-of/for-in loops cause N+1 query problems: the query \
    runs once per iteration instead of once. Use batch queries or collect \
    identifiers first, then query with a WHERE IN clause."
  }

  fn run(&self, ctx: &RuleContext) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    for id in ctx.graph.node_indices() {
      if let NodeKind::Orm(orm) = ctx.graph.node(id).expect("node should exist") {
        if orm.loop_kind == LoopKind::Iteration {
          diags.push(Diagnostic {
            severity: Severity::Warning,
            message: "Database query inside an iteration loop — causes N+1 queries. Use batch queries instead."
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
  use pulsar_ir::{IrGraph, LoopKind, OrmArgs, OrmMethod, OrmNode};

  fn make_graph(loop_kind: LoopKind) -> IrGraph {
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
      loop_kind,
      in_callback: false,
      location,
    };

    graph.add_orm(orm);
    graph
  }

  #[test]
  fn flags_query_in_iteration_loop() {
    let graph = make_graph(LoopKind::Iteration);
    let rule = NoNPlusOne;
    let ctx =
      RuleContext { graph: &graph, source_text: "", file_path: "test.ts", active_rules: &[] };
    let diags = rule.run(&ctx);
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].rule_id, "no-n-plus-one");
    assert_eq!(diags[0].severity, Severity::Warning);
  }

  #[test]
  fn allows_query_in_counter_loop() {
    let graph = make_graph(LoopKind::Counter);
    let rule = NoNPlusOne;
    let ctx =
      RuleContext { graph: &graph, source_text: "", file_path: "test.ts", active_rules: &[] };
    let diags = rule.run(&ctx);
    assert_eq!(diags.len(), 0);
  }

  #[test]
  fn allows_query_outside_loop() {
    let graph = make_graph(LoopKind::None);
    let rule = NoNPlusOne;
    let ctx =
      RuleContext { graph: &graph, source_text: "", file_path: "test.ts", active_rules: &[] };
    let diags = rule.run(&ctx);
    assert_eq!(diags.len(), 0);
  }

  #[test]
  fn empty_graph_no_diagnostics() {
    let graph = IrGraph::new();
    let rule = NoNPlusOne;
    let ctx =
      RuleContext { graph: &graph, source_text: "", file_path: "test.ts", active_rules: &[] };
    let diags = rule.run(&ctx);
    assert!(diags.is_empty());
  }
}
