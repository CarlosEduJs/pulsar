use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;

// Node identifiers
// ================

/// Unique identifier for a node in the IR graph.
pub type NodeId = NodeIndex;

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

/// A node in the IR graph, representing one of the three IR kinds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeKind {
    Sql(SQLNode),
    Orm(OrmNode),
    Schema(SchemaNode),
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
        Self {
            graph: DiGraph::new(),
        }
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
        self.graph
            .edge_references()
            .map(|e| (e.source(), e.target(), e.weight()))
    }
}

impl Default for IrGraph {
    fn default() -> Self {
        Self::new()
    }
}
