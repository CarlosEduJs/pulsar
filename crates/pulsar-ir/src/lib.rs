use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use pulsar_core::SourceLocation;

// Node identifiers
// ================

/// Unique identifier for a node in the IR graph.
pub type NodeId = NodeIndex;

/// Contextual kind of loop an ORM node appears in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoopKind {
  /// Not inside any loop.
  None,
  /// Inside a counter-based loop (for, while, do-while).
  Counter,
  /// Inside an iteration loop (for-of, for-in).
  Iteration,
}

// Edge kinds
// ==========

/// Kind of relationship between two nodes in the IR graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeKind {
  /// An ORM call generates a SQL query.
  Generates,
  /// A SQL query accesses a schema entity.
  Accesses,
  /// An ORM call directly maps to a schema entity.
  MapsTo,
}

// SQL IR
// ======

/// Kind of SQL statement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SqlKind {
  Select,
  Insert,
  Update,
  Delete,
}

/// A column reference in a SQL query.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColumnRef {
  pub name: String,
  pub table: Option<String>,
}

/// A table reference in a SQL query.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableRef {
  pub name: String,
  pub alias: Option<String>,
}

/// A parsed SQL query, scoped to SELECT for v0.1.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SQLNode {
  pub kind: SqlKind,
  pub columns: Vec<ColumnRef>,
  pub table: Option<TableRef>,
  pub limit: bool,
  pub where_clause: bool,
  pub in_callback: bool,
  pub location: SourceLocation,
}

impl SQLNode {
  /// Returns `true` if this query uses `SELECT *` (implicit or explicit wildcard).
  #[must_use]
  pub fn is_select_star(&self) -> bool {
    self.columns.is_empty() || self.columns.iter().any(|c| c.name == "*")
  }
}

// ORM IR
// ======

/// Kind of ORM method call.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrmMethod {
  Select,
  FindMany,
  FindFirst,
  Insert,
  Update,
  Delete,
}

/// Arguments extracted from an ORM call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrmArgs {
  pub columns: Vec<String>,
  pub where_clause: Option<String>,
  pub limit: Option<u64>,
  pub include: Vec<String>,
}

/// A Drizzle ORM operation extracted from TypeScript.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrmNode {
  pub method: OrmMethod,
  pub args: OrmArgs,
  pub loop_kind: LoopKind,
  pub in_callback: bool,
  pub location: SourceLocation,
}

// Raw SQL IR
// ==========

/// Kind of raw SQL usage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RawSqlKind {
  /// sql tagged template literal.
  TaggedTemplate,
  /// Method on db object that executes raw SQL (e.g. db.execute, db.all).
  DbRawMethod,
}

/// A raw SQL expression detected in source code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawSqlNode {
  pub kind: RawSqlKind,
  pub has_interpolation: bool,
  pub location: SourceLocation,
}

// Schema IR
// =========

/// A column definition in the database schema.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaColumn {
  pub name: String,
  pub col_type: String,
  pub is_nullable: bool,
  pub is_indexed: bool,
}

/// A database table reconstructed from schema.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaNode {
  pub table_name: String,
  pub columns: Vec<SchemaColumn>,
}

// Unified node type
// =================

/// A node in the IR graph, representing one of the IR kinds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeKind {
  Sql(SQLNode),
  Orm(OrmNode),
  Schema(SchemaNode),
  RawSql(RawSqlNode),
}

// IR Graph
// ========

/// The unified dependency graph connecting SQL, ORM, and Schema IR nodes.
#[derive(Debug, Clone)]
pub struct IrGraph {
  graph: DiGraph<NodeKind, EdgeKind>,
}

impl IrGraph {
  /// Creates an empty IR graph.
  #[must_use]
  pub fn new() -> Self {
    Self { graph: DiGraph::new() }
  }

  /// Adds a SQL node and returns its identifier.
  pub fn add_sql(&mut self, node: SQLNode) -> NodeId {
    self.graph.add_node(NodeKind::Sql(node))
  }

  /// Adds an ORM node and returns its identifier.
  pub fn add_orm(&mut self, node: OrmNode) -> NodeId {
    self.graph.add_node(NodeKind::Orm(node))
  }

  /// Adds a schema node and returns its identifier.
  pub fn add_schema(&mut self, node: SchemaNode) -> NodeId {
    self.graph.add_node(NodeKind::Schema(node))
  }

  /// Adds a raw SQL node and returns its identifier.
  pub fn add_raw_sql(&mut self, node: RawSqlNode) -> NodeId {
    self.graph.add_node(NodeKind::RawSql(node))
  }

  /// Adds a directed edge between two nodes.
  pub fn add_edge(&mut self, from: NodeId, to: NodeId, kind: EdgeKind) {
    self.graph.add_edge(from, to, kind);
  }

  /// Returns a reference to the node data for the given identifier.
  #[must_use]
  pub fn node(&self, id: NodeId) -> Option<&NodeKind> {
    self.graph.node_weight(id)
  }

  /// Returns the number of nodes in the graph.
  #[must_use]
  pub fn node_count(&self) -> usize {
    self.graph.node_count()
  }

  /// Returns the number of edges in the graph.
  #[must_use]
  pub fn edge_count(&self) -> usize {
    self.graph.edge_count()
  }

  /// Returns an iterator over all node indices.
  pub fn node_indices(&self) -> impl Iterator<Item = NodeId> + '_ {
    self.graph.node_indices()
  }

  /// Returns an iterator over edges and their endpoints.
  pub fn edge_references(&self) -> impl Iterator<Item = (NodeId, NodeId, &EdgeKind)> + '_ {
    self.graph.edge_references().map(|e| (e.source(), e.target(), e.weight()))
  }
}

impl Default for IrGraph {
  fn default() -> Self {
    Self::new()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn make_sql(columns: Vec<ColumnRef>) -> SQLNode {
    SQLNode {
      kind: SqlKind::Select,
      columns,
      table: Some(TableRef { name: "users".to_string(), alias: None }),
      limit: false,
      where_clause: false,
      in_callback: false,
      location: SourceLocation { file: "test.ts".to_string(), line: 1, column: 1, span: None },
    }
  }

  fn location(f: &str, l: usize, c: usize) -> SourceLocation {
    SourceLocation { file: f.to_string(), line: l, column: c, span: None }
  }

  // SQLNode::is_select_star
  // =======================

  #[test]
  fn is_select_star_implicit() {
    let sql = make_sql(vec![]);
    assert!(sql.is_select_star());
  }

  #[test]
  fn is_select_star_explicit_wildcard() {
    let sql = make_sql(vec![ColumnRef { name: "*".to_string(), table: None }]);
    assert!(sql.is_select_star());
  }

  #[test]
  fn is_select_star_mixed_with_wildcard() {
    let sql = make_sql(vec![
      ColumnRef { name: "id".to_string(), table: None },
      ColumnRef { name: "*".to_string(), table: None },
    ]);
    assert!(sql.is_select_star());
  }

  #[test]
  fn is_select_star_explicit_columns() {
    let sql = make_sql(vec![
      ColumnRef { name: "id".to_string(), table: None },
      ColumnRef { name: "name".to_string(), table: None },
    ]);
    assert!(!sql.is_select_star());
  }

  #[test]
  fn is_select_star_qualified_wildcard() {
    let sql = make_sql(vec![ColumnRef { name: "*".to_string(), table: Some("users".to_string()) }]);
    assert!(sql.is_select_star());
  }

  // IrGraph
  // =======

  #[test]
  fn graph_new_is_empty() {
    let g = IrGraph::new();
    assert_eq!(g.node_count(), 0);
    assert_eq!(g.edge_count(), 0);
    assert_eq!(g.node_indices().count(), 0);
    assert_eq!(g.edge_references().count(), 0);
  }

  #[test]
  fn graph_add_sql_and_retrieve() {
    let mut g = IrGraph::new();
    let sql = make_sql(vec![]);
    let id = g.add_sql(sql.clone());
    assert_eq!(g.node_count(), 1);
    match g.node(id) {
      Some(NodeKind::Sql(n)) => assert_eq!(*n, sql),
      _ => panic!("expected Sql node"),
    }
  }

  #[test]
  fn graph_add_orm_and_retrieve() {
    let mut g = IrGraph::new();
    let orm = OrmNode {
      method: OrmMethod::Select,
      args: OrmArgs {
        columns: vec!["id".to_string()],
        where_clause: None,
        limit: None,
        include: Vec::new(),
      },
      loop_kind: LoopKind::None,
      in_callback: false,
      location: location("test.ts", 1, 1),
    };
    let id = g.add_orm(orm.clone());
    assert_eq!(g.node_count(), 1);
    match g.node(id) {
      Some(NodeKind::Orm(n)) => assert_eq!(*n, orm),
      _ => panic!("expected Orm node"),
    }
  }

  #[test]
  fn graph_add_schema_and_retrieve() {
    let mut g = IrGraph::new();
    let schema = SchemaNode {
      table_name: "users".to_string(),
      columns: vec![SchemaColumn {
        name: "id".to_string(),
        col_type: "integer".to_string(),
        is_nullable: false,
        is_indexed: true,
      }],
    };
    let id = g.add_schema(schema.clone());
    assert_eq!(g.node_count(), 1);
    match g.node(id) {
      Some(NodeKind::Schema(n)) => assert_eq!(*n, schema),
      _ => panic!("expected Schema node"),
    }
  }

  #[test]
  fn graph_add_edge_and_query() {
    let mut g = IrGraph::new();
    let sql = g.add_sql(make_sql(vec![]));
    let orm = g.add_orm(OrmNode {
      method: OrmMethod::Select,
      args: OrmArgs { columns: vec![], where_clause: None, limit: None, include: Vec::new() },
      loop_kind: LoopKind::None,
      in_callback: false,
      location: location("test.ts", 1, 1),
    });
    g.add_edge(orm, sql, EdgeKind::Generates);
    assert_eq!(g.node_count(), 2);
    assert_eq!(g.edge_count(), 1);
    let refs: Vec<_> = g.edge_references().collect();
    assert_eq!(refs.len(), 1);
    assert_eq!(refs[0], (orm, sql, &EdgeKind::Generates));
  }

  #[test]
  fn graph_node_indices_iteration() {
    let mut g = IrGraph::new();
    let s1 = g.add_sql(make_sql(vec![]));
    let s2 = g.add_sql(make_sql(vec![ColumnRef { name: "id".to_string(), table: None }]));
    let ids: Vec<_> = g.node_indices().collect();
    assert_eq!(ids.len(), 2);
    assert!(ids.contains(&s1));
    assert!(ids.contains(&s2));
  }

  #[test]
  fn graph_node_returns_none_for_invalid_id() {
    let g = IrGraph::new();
    assert!(g.node(NodeIndex::new(999)).is_none());
  }

  #[test]
  fn graph_default_is_empty() {
    let g = IrGraph::default();
    assert_eq!(g.node_count(), 0);
  }

  // EdgeKind and SqlKind debug/equality
  #[test]
  fn graph_add_raw_sql_and_retrieve() {
    let mut g = IrGraph::new();
    let raw = RawSqlNode {
      kind: RawSqlKind::TaggedTemplate,
      has_interpolation: true,
      location: location("test.ts", 1, 1),
    };
    let id = g.add_raw_sql(raw.clone());
    assert_eq!(g.node_count(), 1);
    match g.node(id) {
      Some(NodeKind::RawSql(n)) => assert_eq!(*n, raw),
      _ => panic!("expected RawSql node"),
    }
  }

  #[test]
  fn raw_sql_kind_variants() {
    assert_eq!(RawSqlKind::TaggedTemplate, RawSqlKind::TaggedTemplate);
    assert_eq!(RawSqlKind::DbRawMethod, RawSqlKind::DbRawMethod);
    assert_ne!(RawSqlKind::TaggedTemplate, RawSqlKind::DbRawMethod);
  }

  #[test]
  fn edge_kind_variants() {
    assert_eq!(EdgeKind::Generates, EdgeKind::Generates);
    assert_eq!(EdgeKind::Accesses, EdgeKind::Accesses);
    assert_eq!(EdgeKind::MapsTo, EdgeKind::MapsTo);
    assert_ne!(EdgeKind::Generates, EdgeKind::Accesses);
  }

  #[test]
  fn sql_kind_variants() {
    assert_eq!(SqlKind::Select, SqlKind::Select);
    assert_eq!(SqlKind::Insert, SqlKind::Insert);
    assert_eq!(SqlKind::Update, SqlKind::Update);
    assert_eq!(SqlKind::Delete, SqlKind::Delete);
  }
}
