use pulsar_core::SourceLocation;
use pulsar_ir::{
  ColumnRef, EdgeKind, IrGraph, OrmArgs, OrmMethod, OrmNode, SQLNode, SqlKind, TableRef,
};

/// Constructs an [`OrmNode`] from extracted method-chain data.
#[must_use]
pub const fn build_orm_node(
  columns: Vec<String>,
  where_clause: Option<String>,
  limit: Option<u64>,
  in_loop: bool,
  location: SourceLocation,
) -> OrmNode {
  OrmNode {
    method: OrmMethod::Select,
    args: OrmArgs { columns, where_clause, limit, include: Vec::new() },
    in_loop,
    location,
  }
}

/// Constructs a [`SQLNode`] from extracted method-chain data.
#[must_use]
pub fn build_sql_node(
  columns: Vec<String>,
  table_name: Option<String>,
  has_limit: bool,
  has_where: bool,
  location: SourceLocation,
) -> SQLNode {
  let cols = columns.into_iter().map(|c| ColumnRef { name: c, table: None }).collect();
  let table = table_name.map(|t| TableRef { name: t, alias: None });
  SQLNode {
    kind: SqlKind::Select,
    columns: cols,
    table,
    limit: has_limit,
    where_clause: has_where,
    location,
  }
}

/// Converts a Drizzle `select` chain into ORM + SQL nodes and links them in the graph.
#[allow(clippy::too_many_arguments)]
pub fn process_drizzle_chain(
  columns: Vec<String>,
  table_name: Option<String>,
  limit: Option<u64>,
  where_clause: Option<String>,
  in_loop: bool,
  location: SourceLocation,
  graph: &mut IrGraph,
) {
  let has_limit = limit.is_some();
  let has_where = where_clause.is_some();

  let orm_node = build_orm_node(columns.clone(), where_clause, limit, in_loop, location.clone());
  let sql_node = build_sql_node(columns, table_name, has_limit, has_where, location);

  let orm_id = graph.add_orm(orm_node);
  let sql_id = graph.add_sql(sql_node);
  graph.add_edge(orm_id, sql_id, EdgeKind::Generates);
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn process_select_chain_adds_two_nodes_and_edge() {
    let mut graph = IrGraph::new();
    let loc = SourceLocation { file: "test.ts".to_string(), line: 1, column: 1, span: None };

    process_drizzle_chain(
      vec!["id".to_string(), "name".to_string()],
      Some("users".to_string()),
      Some(10),
      Some("eq(users.id, 1)".to_string()),
      false,
      loc,
      &mut graph,
    );

    assert_eq!(graph.node_count(), 2, "should have ORM + SQL nodes");
    assert_eq!(graph.edge_count(), 1, "should have Generates edge");
  }

  #[test]
  fn process_select_star_chain() {
    let mut graph = IrGraph::new();
    let loc = SourceLocation { file: "test.ts".to_string(), line: 2, column: 5, span: None };

    process_drizzle_chain(vec![], Some("users".to_string()), None, None, false, loc, &mut graph);

    assert_eq!(graph.node_count(), 2);
    for id in graph.node_indices() {
      if let pulsar_ir::NodeKind::Sql(sql) = graph.node(id).unwrap() {
        assert!(sql.is_select_star());
      }
    }
  }

  #[test]
  fn process_chain_marks_in_loop() {
    let mut graph = IrGraph::new();
    let loc = SourceLocation { file: "test.ts".to_string(), line: 3, column: 7, span: None };

    process_drizzle_chain(
      vec!["id".to_string()],
      Some("users".to_string()),
      Some(5),
      None,
      true,
      loc,
      &mut graph,
    );

    assert_eq!(graph.node_count(), 2);
    for id in graph.node_indices() {
      if let pulsar_ir::NodeKind::Orm(orm) = graph.node(id).unwrap() {
        assert!(orm.in_loop);
        assert_eq!(orm.args.limit, Some(5));
      }
    }
  }
}
