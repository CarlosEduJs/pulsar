use pulsar_core::{Diagnostic, Severity, SourceLocation};
use pulsar_ir::{
  ColumnRef, ForeignKeyRef, LoopKind, OrmArgs, OrmMethod, OrmNode, RawSqlKind, RawSqlNode, SQLNode,
  SchemaColumn, SchemaIndex, SchemaNode, SqlKind, TableRef,
};

// SourceLocation
// ==============

pub fn loc() -> SourceLocation {
  SourceLocation { file: "test.ts".to_string(), line: 1, column: 1, span: None }
}

pub fn loc_at(file: &str, line: usize, column: usize) -> SourceLocation {
  SourceLocation { file: file.to_string(), line, column, span: None }
}

pub fn loc_with_span(
  file: &str,
  line: usize,
  column: usize,
  start: usize,
  end: usize,
) -> SourceLocation {
  SourceLocation { file: file.to_string(), line, column, span: Some((start, end)) }
}

// Column / Table references
// =========================

pub fn col(name: &str) -> ColumnRef {
  ColumnRef { name: name.to_string(), table: None }
}

pub fn col_qualified(name: &str, table: &str) -> ColumnRef {
  ColumnRef { name: name.to_string(), table: Some(table.to_string()) }
}

pub fn col_star() -> ColumnRef {
  ColumnRef { name: "*".to_string(), table: None }
}

pub fn col_star_qualified(table: &str) -> ColumnRef {
  ColumnRef { name: "*".to_string(), table: Some(table.to_string()) }
}

pub fn table(name: &str) -> TableRef {
  TableRef { name: name.to_string(), alias: None }
}

pub fn table_aliased(name: &str, alias: &str) -> TableRef {
  TableRef { name: name.to_string(), alias: Some(alias.to_string()) }
}

// SQLNode
// =======

pub fn sql_select(columns: Vec<ColumnRef>) -> SQLNode {
  sql_select_full(columns, Some(table("users")), false, false, false, loc())
}

pub fn sql_select_star() -> SQLNode {
  sql_select_full(vec![], Some(table("users")), false, false, false, loc())
}

#[allow(clippy::too_many_arguments)]
pub fn sql_select_full(
  columns: Vec<ColumnRef>,
  table: Option<TableRef>,
  has_limit: bool,
  has_where: bool,
  in_callback: bool,
  location: SourceLocation,
) -> SQLNode {
  SQLNode {
    kind: SqlKind::Select,
    columns,
    table,
    limit: has_limit,
    where_clause: has_where,
    in_callback,
    location,
  }
}

pub fn sql_select_custom(columns: Vec<ColumnRef>, table: Option<TableRef>) -> SQLNode {
  sql_select_full(columns, table, false, false, false, loc())
}

// OrmNode
// =======

pub fn orm_select(columns: Vec<&str>) -> OrmNode {
  orm_select_full(columns, None, None, vec![], LoopKind::None, false, false, loc())
}

pub fn orm_select_bare() -> OrmNode {
  orm_select_full(vec![], None, None, vec![], LoopKind::None, false, false, loc())
}

#[allow(clippy::too_many_arguments)]
pub fn orm_select_full(
  columns: Vec<&str>,
  where_clause: Option<&str>,
  limit: Option<u64>,
  include: Vec<&str>,
  loop_kind: LoopKind,
  in_callback: bool,
  missing_await: bool,
  location: SourceLocation,
) -> OrmNode {
  OrmNode {
    method: OrmMethod::Select,
    args: OrmArgs {
      columns: columns.into_iter().map(ToString::to_string).collect(),
      where_clause: where_clause.map(ToString::to_string),
      limit,
      include: include.into_iter().map(ToString::to_string).collect(),
    },
    loop_kind,
    in_callback,
    missing_await,
    location,
  }
}

pub fn orm_method(
  method: OrmMethod,
  columns: Vec<&str>,
  where_clause: Option<&str>,
  limit: Option<u64>,
) -> OrmNode {
  OrmNode {
    method,
    args: OrmArgs {
      columns: columns.into_iter().map(ToString::to_string).collect(),
      where_clause: where_clause.map(ToString::to_string),
      limit,
      include: vec![],
    },
    loop_kind: LoopKind::None,
    in_callback: false,
    missing_await: false,
    location: loc(),
  }
}

// RawSqlNode
// ==========

pub fn raw_sql(kind: RawSqlKind, has_interpolation: bool) -> RawSqlNode {
  RawSqlNode { kind, has_interpolation, location: loc() }
}

pub fn raw_sql_tagged(has_interpolation: bool) -> RawSqlNode {
  raw_sql(RawSqlKind::TaggedTemplate, has_interpolation)
}

pub fn raw_sql_db_method(has_interpolation: bool) -> RawSqlNode {
  raw_sql(RawSqlKind::DbRawMethod, has_interpolation)
}

// Schema
// ======

pub fn schema_column(name: &str, col_type: &str) -> SchemaColumn {
  SchemaColumn {
    name: name.to_string(),
    col_type: col_type.to_string(),
    is_nullable: false,
    is_indexed: false,
    col_default: None,
    is_unique: false,
    foreign_key: None,
  }
}

pub fn schema_column_indexed(name: &str, col_type: &str) -> SchemaColumn {
  SchemaColumn {
    name: name.to_string(),
    col_type: col_type.to_string(),
    is_nullable: false,
    is_indexed: true,
    col_default: None,
    is_unique: false,
    foreign_key: None,
  }
}

pub fn schema_column_pk(name: &str, col_type: &str) -> SchemaColumn {
  SchemaColumn {
    name: name.to_string(),
    col_type: col_type.to_string(),
    is_nullable: false,
    is_indexed: true,
    col_default: Some("autoincrement()".to_string()),
    is_unique: true,
    foreign_key: None,
  }
}

pub fn schema_column_fk(
  name: &str,
  col_type: &str,
  ref_table: &str,
  ref_column: &str,
) -> SchemaColumn {
  SchemaColumn {
    name: name.to_string(),
    col_type: col_type.to_string(),
    is_nullable: true,
    is_indexed: false,
    col_default: None,
    is_unique: false,
    foreign_key: Some(ForeignKeyRef {
      ref_table: ref_table.to_string(),
      ref_column: ref_column.to_string(),
    }),
  }
}

pub fn schema_table(name: &str, columns: Vec<SchemaColumn>) -> SchemaNode {
  SchemaNode { table_name: name.to_string(), columns, indexes: vec![] }
}

pub fn schema_table_with_indexes(
  name: &str,
  columns: Vec<SchemaColumn>,
  indexes: Vec<SchemaIndex>,
) -> SchemaNode {
  SchemaNode { table_name: name.to_string(), columns, indexes }
}

pub fn schema_index(columns: Vec<&str>) -> SchemaIndex {
  SchemaIndex {
    columns: columns.into_iter().map(ToString::to_string).collect(),
    is_unique: false,
    is_partial: false,
  }
}

pub fn schema_unique_index(columns: Vec<&str>) -> SchemaIndex {
  SchemaIndex {
    columns: columns.into_iter().map(ToString::to_string).collect(),
    is_unique: true,
    is_partial: false,
  }
}

// ForeignKey
// ==========

pub fn fk_ref(ref_table: &str, ref_column: &str) -> ForeignKeyRef {
  ForeignKeyRef { ref_table: ref_table.to_string(), ref_column: ref_column.to_string() }
}

// Diagnostic
// ==========

pub fn diag(
  rule_id: &str,
  severity: Severity,
  message: &str,
  location: SourceLocation,
) -> Diagnostic {
  Diagnostic { severity, message: message.to_string(), location, rule_id: rule_id.to_string() }
}

pub fn diag_error(rule_id: &str, message: &str) -> Diagnostic {
  diag(rule_id, Severity::Error, message, loc())
}

pub fn diag_warning(rule_id: &str, message: &str) -> Diagnostic {
  diag(rule_id, Severity::Warning, message, loc())
}

pub fn diag_info(rule_id: &str, message: &str) -> Diagnostic {
  diag(rule_id, Severity::Info, message, loc())
}
