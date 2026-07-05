#![allow(clippy::multiple_crate_versions)]

use proptest::prelude::*;
use pulsar_core::{Severity, SourceLocation};
use pulsar_ir::{
  ColumnRef, EdgeKind, IrGraph, LoopKind, OrmArgs, OrmMethod, OrmNode, RawSqlKind, RawSqlNode,
  SQLNode, SchemaColumn, SchemaNode, SqlKind, TableRef,
};
use pulsar_rules::rules::*;
use pulsar_rules::{Rule, RuleContext, RuleEngine};

fn loc() -> SourceLocation {
  SourceLocation { file: "prop_test.ts".to_string(), line: 1, column: 1, span: None }
}

/// Build a rule engine with all available rules.
///
/// NOTE: Keep in sync with `pulsar_test_utils::rules::all_rules_engine`.
/// The canonical copy lives in `pulsar-test-utils` and is used by
/// `pulsar-integration-tests` and the fuzz targets. This local copy
/// exists because `pulsar-rules` cannot take a dev-dependency on
/// `pulsar-test-utils` (would create a cycle).
fn all_rules_engine() -> RuleEngine {
  let mut engine = RuleEngine::new();
  engine.register(Box::new(NoSelectStar));
  engine.register(Box::new(NoMissingLimit));
  engine.register(Box::new(NoUnboundedFind));
  engine.register(Box::new(NoAlwaysTrueWhere));
  engine.register(Box::new(NoQueryInLoop));
  engine.register(Box::new(NoQueryInCallback));
  engine.register(Box::new(NoNPlusOne));
  engine.register(Box::new(NoRawSqlDangerous));
  engine.register(Box::new(NoMissingAwait));
  engine.register(Box::new(NoUnindexedFilter));
  engine.register(Box::new(NoUnknownColumn));
  engine.register(Box::new(NoMissingForeignKey));
  engine
}

#[test]
fn empty_graph_no_diagnostics() {
  let engine = all_rules_engine();
  let diags = engine.run(&IrGraph::new(), "", "test.ts");
  assert!(diags.is_empty());
}

#[test]
fn all_rules_on_random_graph() {
  proptest!(|(num_orms in 0usize..3, num_sqls in 0usize..3, num_schemas in 0usize..2, num_raws in 0usize..2)| {
    let mut graph = IrGraph::new();
    for i in 0..num_orms {
      let orm = OrmNode {
        method: OrmMethod::Select,
        args: OrmArgs {
          columns: vec!["id".to_string()],
          where_clause: if i % 2 == 0 { Some("eq(users.id, 1)".to_string()) } else { None },
          limit: if i % 3 == 0 { Some(10) } else { None },
          include: vec![],
        },
        loop_kind: match i % 3 { 0 => LoopKind::None, 1 => LoopKind::Counter, _ => LoopKind::Iteration },
        in_callback: i % 4 == 0,
        missing_await: i % 5 == 0,
        location: SourceLocation { file: "test.ts".to_string(), line: i + 1, column: 1, span: None },
      };
      let oid = graph.add_orm(orm);
      if num_sqls > 0 {
        let sid = graph.add_sql(sql_node(Some(i)));
        graph.add_edge(oid, sid, EdgeKind::Generates);
      }
    }
    for _ in 0..num_sqls.saturating_sub(num_orms) {
      graph.add_sql(sql_node(None));
    }
    for _ in 0..num_schemas {
      graph.add_schema(SchemaNode {
        table_name: format!("tbl_{}", graph.node_count()),
        columns: vec![SchemaColumn {
          name: "id".to_string(), col_type: "Int".to_string(),
          is_nullable: false, is_indexed: true, col_default: None, is_unique: true,
          foreign_key: None,
        }],
        indexes: vec![],
      });
    }
    for _ in 0..num_raws {
      graph.add_raw_sql(RawSqlNode {
        kind: RawSqlKind::TaggedTemplate, has_interpolation: false, location: loc(),
      });
    }

    let engine = all_rules_engine();
    let diags = engine.run(&graph, "source text", "test.ts");

    for d in &diags {
      prop_assert!(!d.rule_id.is_empty());
      prop_assert!(!d.message.is_empty());
      prop_assert!(!d.location.file.is_empty());
      prop_assert!(matches!(d.severity, Severity::Error | Severity::Warning | Severity::Info));
    }
  });
}

fn sql_node(seed: Option<usize>) -> SQLNode {
  let has_columns = seed.is_some_and(|s| s % 2 == 0);
  SQLNode {
    kind: SqlKind::Select,
    columns: if has_columns {
      vec![ColumnRef { name: "id".to_string(), table: None }]
    } else {
      vec![]
    },
    table: Some(TableRef { name: "users".to_string(), alias: None }),
    limit: seed.is_some_and(|s| s % 3 == 0),
    where_clause: seed.is_some_and(|s| s % 4 == 0),
    in_callback: seed.is_some_and(|s| s % 5 == 0),
    location: loc(),
  }
}

#[test]
fn select_star_fires() {
  proptest!(|(has_limit in proptest::bool::ANY)| {
    let mut graph = IrGraph::new();
    graph.add_sql(SQLNode {
      kind: SqlKind::Select, columns: vec![],
      table: Some(TableRef { name: "users".to_string(), alias: None }),
      limit: has_limit, where_clause: false, in_callback: false, location: loc(),
    });
    graph.add_orm(OrmNode {
      method: OrmMethod::Select,
      args: OrmArgs { columns: vec![], where_clause: None, limit: None, include: vec![] },
      loop_kind: LoopKind::None, in_callback: false, missing_await: false, location: loc(),
    });
    let rule = NoSelectStar;
    let ctx = RuleContext { graph: &graph, source_text: "", file_path: "test.ts", active_rules: &[] };
    let diags = rule.run(&ctx);
    prop_assert_eq!(diags.len(), 1);
    prop_assert_eq!(diags[0].severity, Severity::Error);
  });
}

#[test]
fn missing_limit_fires() {
  let mut graph = IrGraph::new();
  graph.add_sql(SQLNode {
    kind: SqlKind::Select,
    columns: vec![ColumnRef { name: "id".to_string(), table: None }],
    table: Some(TableRef { name: "users".to_string(), alias: None }),
    limit: false,
    where_clause: false,
    in_callback: false,
    location: loc(),
  });
  graph.add_orm(OrmNode {
    method: OrmMethod::Select,
    args: OrmArgs {
      columns: vec!["id".to_string()],
      where_clause: None,
      limit: None,
      include: vec![],
    },
    loop_kind: LoopKind::None,
    in_callback: false,
    missing_await: false,
    location: loc(),
  });
  let rule = NoMissingLimit;
  let ctx = RuleContext { graph: &graph, source_text: "", file_path: "test.ts", active_rules: &[] };
  let diags = rule.run(&ctx);
  assert_eq!(diags.len(), 1);
  assert_eq!(diags[0].severity, Severity::Warning);
}

#[test]
fn query_in_loop_flags_counter() {
  let graph = single_orm_graph(LoopKind::Counter);
  let rule = NoQueryInLoop;
  let ctx = RuleContext { graph: &graph, source_text: "", file_path: "test.ts", active_rules: &[] };
  let diags = rule.run(&ctx);
  assert_eq!(diags.len(), 1);
}

#[test]
fn n_plus_one_flags_iteration() {
  let graph = single_orm_graph(LoopKind::Iteration);
  let rule = NoNPlusOne;
  let ctx = RuleContext { graph: &graph, source_text: "", file_path: "test.ts", active_rules: &[] };
  let diags = rule.run(&ctx);
  assert_eq!(diags.len(), 1);
}

#[test]
fn raw_sql_dangerous_fires_for_interpolated() {
  let mut graph = IrGraph::new();
  graph.add_raw_sql(RawSqlNode {
    kind: RawSqlKind::TaggedTemplate,
    has_interpolation: true,
    location: loc(),
  });
  let rule = NoRawSqlDangerous;
  let ctx = RuleContext { graph: &graph, source_text: "", file_path: "test.ts", active_rules: &[] };
  let diags = rule.run(&ctx);
  assert_eq!(diags.len(), 1);
  assert_eq!(diags[0].severity, Severity::Error);
}

#[test]
fn missing_await_fires() {
  let mut graph = IrGraph::new();
  graph.add_orm(OrmNode {
    method: OrmMethod::Select,
    args: OrmArgs { columns: vec![], where_clause: None, limit: None, include: vec![] },
    loop_kind: LoopKind::None,
    in_callback: false,
    missing_await: true,
    location: loc(),
  });
  let rule = NoMissingAwait;
  let ctx = RuleContext { graph: &graph, source_text: "", file_path: "test.ts", active_rules: &[] };
  let diags = rule.run(&ctx);
  assert_eq!(diags.len(), 1);
  assert_eq!(diags[0].severity, Severity::Error);
}

#[test]
fn always_true_where_fires() {
  let mut graph = IrGraph::new();
  graph.add_orm(OrmNode {
    method: OrmMethod::Select,
    args: OrmArgs {
      columns: vec![],
      where_clause: Some("true".to_string()),
      limit: None,
      include: vec![],
    },
    loop_kind: LoopKind::None,
    in_callback: false,
    missing_await: false,
    location: loc(),
  });
  let rule = NoAlwaysTrueWhere;
  let ctx = RuleContext { graph: &graph, source_text: "", file_path: "test.ts", active_rules: &[] };
  let diags = rule.run(&ctx);
  assert_eq!(diags.len(), 1);
  assert_eq!(diags[0].severity, Severity::Error);
}

#[test]
fn query_in_callback_fires() {
  proptest!(|(in_callback in proptest::bool::ANY)| {
    let mut graph = IrGraph::new();
    graph.add_sql(SQLNode {
      kind: SqlKind::Select, columns: vec![],
      table: Some(TableRef { name: "users".to_string(), alias: None }),
      limit: false, where_clause: false, in_callback, location: loc(),
    });
    let rule = NoQueryInCallback;
    let ctx = RuleContext { graph: &graph, source_text: "", file_path: "test.ts", active_rules: &[] };
    let diags = rule.run(&ctx);
    if in_callback {
      prop_assert_eq!(diags.len(), 1);
    } else {
      prop_assert!(diags.is_empty());
    }
  });
}

#[test]
fn unbounded_find_fires() {
  proptest!(|(has_where in proptest::bool::ANY, has_limit in proptest::bool::ANY)| {
    let mut graph = IrGraph::new();
    graph.add_orm(OrmNode {
      method: OrmMethod::Select,
      args: OrmArgs {
        columns: vec![],
        where_clause: if has_where { Some("eq(x, 1)".to_string()) } else { None },
        limit: if has_limit { Some(10) } else { None },
        include: vec![],
      },
      loop_kind: LoopKind::None, in_callback: false, missing_await: false, location: loc(),
    });
    let rule = NoUnboundedFind;
    let ctx = RuleContext { graph: &graph, source_text: "", file_path: "test.ts", active_rules: &[] };
    let diags = rule.run(&ctx);
    if has_where || has_limit {
      prop_assert!(diags.is_empty(), "bounded query should not be flagged");
    } else {
      prop_assert_eq!(diags.len(), 1, "unbounded query should be flagged");
    }
  });
}

fn single_orm_graph(loop_kind: LoopKind) -> IrGraph {
  let mut graph = IrGraph::new();
  graph.add_orm(OrmNode {
    method: OrmMethod::Select,
    args: OrmArgs {
      columns: vec!["id".to_string()],
      where_clause: None,
      limit: None,
      include: vec![],
    },
    loop_kind,
    in_callback: false,
    missing_await: false,
    location: loc(),
  });
  graph
}
