use std::collections::HashMap;

use pulsar_ir::{EdgeKind, IrGraph, NodeId, OrmNode, RawSqlNode, SQLNode, SchemaNode};

/// Fluent builder for constructing test [`IrGraph`] instances.
///
/// # Example
///
/// ```ignore
/// use pulsar_test_utils::builders::G;
/// use pulsar_test_utils::factories::*;
///
/// let graph = G::new()
///     .orm(orm_select_bare())
///     .sql(sql_select_star())
///     .generates()
///     .schema(schema_table("users", vec![schema_column_pk("id", "Int")]))
///     .accesses()
///     .finish();
/// ```
///
/// The builder tracks the latest added node of each type so convenience
/// methods like `.generates()` and `.accesses()` connect the most recent
/// ORM→SQL and SQL→Schema respectively.
pub struct G {
  graph: IrGraph,
  last_orm: Option<NodeId>,
  last_sql: Option<NodeId>,
  last_schema: Option<NodeId>,
  last_raw_sql: Option<NodeId>,
}

impl G {
  /// Creates a new empty graph builder.
  #[must_use]
  pub fn new() -> Self {
    Self {
      graph: IrGraph::new(),
      last_orm: None,
      last_sql: None,
      last_schema: None,
      last_raw_sql: None,
    }
  }

  /// Adds an ORM node and tracks it as the most recent ORM node.
  pub fn orm(&mut self, node: OrmNode) -> &mut Self {
    let id = self.graph.add_orm(node);
    self.last_orm = Some(id);
    self
  }

  /// Adds a SQL node and tracks it as the most recent SQL node.
  pub fn sql(&mut self, node: SQLNode) -> &mut Self {
    let id = self.graph.add_sql(node);
    self.last_sql = Some(id);
    self
  }

  /// Adds a schema node and tracks it as the most recent schema node.
  pub fn schema(&mut self, node: SchemaNode) -> &mut Self {
    let id = self.graph.add_schema(node);
    self.last_schema = Some(id);
    self
  }

  /// Adds a raw SQL node and tracks it as the most recent raw SQL node.
  pub fn raw_sql(&mut self, node: RawSqlNode) -> &mut Self {
    let id = self.graph.add_raw_sql(node);
    self.last_raw_sql = Some(id);
    self
  }

  /// Adds a [`Generates`](EdgeKind::Generates) edge from the last ORM node to
  /// the last SQL node.
  ///
  /// # Panics
  ///
  /// Panics if no ORM or SQL node has been added yet.
  pub fn generates(&mut self) -> &mut Self {
    let from = self.last_orm.expect("no ORM node added before .generates()");
    let to = self.last_sql.expect("no SQL node added before .generates()");
    self.graph.add_edge(from, to, EdgeKind::Generates);
    self
  }

  /// Adds an [`Accesses`](EdgeKind::Accesses) edge from the last SQL node to
  /// the last schema node.
  ///
  /// # Panics
  ///
  /// Panics if no SQL or schema node has been added yet.
  pub fn accesses(&mut self) -> &mut Self {
    let from = self.last_sql.expect("no SQL node added before .accesses()");
    let to = self.last_schema.expect("no schema node added before .accesses()");
    self.graph.add_edge(from, to, EdgeKind::Accesses);
    self
  }

  /// Adds a [`MapsTo`](EdgeKind::MapsTo) edge from the last ORM node to the
  /// last schema node.
  ///
  /// # Panics
  ///
  /// Panics if no ORM or schema node has been added yet.
  pub fn maps_to(&mut self) -> &mut Self {
    let from = self.last_orm.expect("no ORM node added before .maps_to()");
    let to = self.last_schema.expect("no schema node added before .maps_to()");
    self.graph.add_edge(from, to, EdgeKind::MapsTo);
    self
  }

  /// Adds a custom edge between two specific nodes.
  pub fn edge(&mut self, from: NodeId, to: NodeId, kind: EdgeKind) -> &mut Self {
    self.graph.add_edge(from, to, kind);
    self
  }

  /// Returns the [`NodeId`] of the most recently added ORM node.
  #[must_use]
  pub fn last_orm(&self) -> Option<NodeId> {
    self.last_orm
  }

  /// Returns the [`NodeId`] of the most recently added SQL node.
  #[must_use]
  pub fn last_sql(&self) -> Option<NodeId> {
    self.last_sql
  }

  /// Returns the [`NodeId`] of the most recently added schema node.
  #[must_use]
  pub fn last_schema(&self) -> Option<NodeId> {
    self.last_schema
  }

  /// Returns the [`NodeId`] of the most recently added raw SQL node.
  #[must_use]
  pub fn last_raw_sql(&self) -> Option<NodeId> {
    self.last_raw_sql
  }

  /// Loads a schema map into the graph (links existing SQL/ORM nodes).
  pub fn load_schema(&mut self, schema: &HashMap<String, SchemaNode>) -> &mut Self {
    self.graph.load_schema(schema);
    self
  }

  /// Consumes the builder and returns the constructed [`IrGraph`].
  #[must_use]
  pub fn finish(&mut self) -> IrGraph {
    std::mem::take(&mut self.graph)
  }
}

impl Default for G {
  fn default() -> Self {
    Self::new()
  }
}

// Convenience constructors for common graph shapes
// ================================================

impl G {
  /// Builds a graph with a single ORM node (no edges).
  ///
  /// Useful for rules that inspect ORM nodes in isolation (e.g.,
  /// `no-unbounded-find`, `no-query-in-loop`).
  pub fn single_orm(orm: OrmNode) -> IrGraph {
    let mut g = Self::new();
    g.orm(orm);
    g.finish()
  }

  /// Builds a graph with a single SQL node (no edges).
  pub fn single_sql(sql: SQLNode) -> IrGraph {
    let mut g = Self::new();
    g.sql(sql);
    g.finish()
  }

  /// Builds a graph with a single raw SQL node (no edges).
  pub fn single_raw_sql(raw: RawSqlNode) -> IrGraph {
    let mut g = Self::new();
    g.raw_sql(raw);
    g.finish()
  }

  /// Builds a graph with an ORM node linked to a SQL node via a
  /// [`Generates`](EdgeKind::Generates) edge.
  ///
  /// This is the most common shape for non-schema rules.
  pub fn orm_sql(orm: OrmNode, sql: SQLNode) -> IrGraph {
    let mut g = Self::new();
    g.orm(orm).sql(sql).generates();
    g.finish()
  }

  /// Builds a graph with ORM → SQL → Schema linked.
  ///
  /// This is the standard shape for schema-aware rules.
  pub fn orm_sql_schema(orm: OrmNode, sql: SQLNode, schema: SchemaNode) -> IrGraph {
    let mut g = Self::new();
    g.orm(orm).sql(sql).schema(schema).generates().accesses();
    g.finish()
  }

  /// Builds a graph with ORM → Schema directly linked (MapsTo).
  pub fn orm_schema(orm: OrmNode, schema: SchemaNode) -> IrGraph {
    let mut g = Self::new();
    g.orm(orm).schema(schema).maps_to();
    g.finish()
  }
}
