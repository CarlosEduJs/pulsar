use pulsar_core::SourceLocation;
use pulsar_ir::{ColumnRef, SQLNode, SqlKind, TableRef};
use sqlparser::ast::{SelectItem, SetExpr, Statement, TableFactor};
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::parser::Parser;

/// Errors that can occur during SQL parsing.
#[derive(Debug, thiserror::Error)]
pub enum SqlParseError {
  /// The SQL could not be parsed.
  #[error("parse error: {0}")]
  ParseError(String),
  /// The statement kind is not yet supported.
  #[error("unsupported statement: {0}")]
  UnsupportedStatement(String),
}

/// Parses a SQL query string into a [`SQLNode`].
///
/// For v0.1 only `SELECT` statements are supported.
///
/// # Errors
///
/// Returns [`SqlParseError`] if the SQL is invalid or the statement kind
/// is not yet supported.
pub fn parse_sql(sql: &str, location: SourceLocation) -> Result<SQLNode, SqlParseError> {
  let dialect = PostgreSqlDialect {};
  let statements =
    Parser::parse_sql(&dialect, sql).map_err(|e| SqlParseError::ParseError(e.to_string()))?;

  let statement = statements
    .into_iter()
    .next()
    .ok_or_else(|| SqlParseError::ParseError("empty input".to_string()))?;

  match statement {
    Statement::Query(query) => {
      let body = *query.body;

      let (select, limit) = match body {
        SetExpr::Select(select) => (*select, query.limit),
        _ => {
          return Err(SqlParseError::UnsupportedStatement("only SELECT is supported".to_string()));
        }
      };

      let kind = SqlKind::Select;

      let columns: Vec<ColumnRef> = select
        .projection
        .iter()
        .map(|item| match item {
          SelectItem::UnnamedExpr(expr) | SelectItem::ExprWithAlias { expr, .. } => {
            ColumnRef { name: expr.to_string(), table: None }
          }
          SelectItem::QualifiedWildcard(prefix, _) => {
            ColumnRef { name: "*".to_string(), table: Some(prefix.to_string()) }
          }
          SelectItem::Wildcard(..) => ColumnRef { name: "*".to_string(), table: None },
        })
        .collect();

      let table = select.from.first().map(|t| match &t.relation {
        TableFactor::Table { name, alias, .. } => TableRef {
          name: name.to_string(),
          alias: alias.as_ref().map(std::string::ToString::to_string),
        },
        _ => TableRef { name: t.relation.to_string(), alias: None },
      });

      let limit = limit.is_some();
      let where_clause = select.selection.is_some();

      Ok(SQLNode { kind, columns, table, limit, where_clause, location })
    }
    other => Err(SqlParseError::UnsupportedStatement(other.to_string())),
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn test_location() -> SourceLocation {
    SourceLocation { file: "test.sql".to_string(), line: 1, column: 1, span: None }
  }

  #[test]
  fn parse_select_star() {
    let node = parse_sql("SELECT * FROM users", test_location()).unwrap();
    assert!(node.is_select_star());
    assert_eq!(node.columns.len(), 1);
    assert_eq!(node.columns[0].name, "*");
    assert!(node.table.is_some());
    assert_eq!(node.table.as_ref().unwrap().name, "users");
  }

  #[test]
  fn parse_select_explicit_columns() {
    let node = parse_sql("SELECT id, name FROM users", test_location()).unwrap();
    assert!(!node.is_select_star());
    assert_eq!(node.columns.len(), 2);
    assert_eq!(node.columns[0].name, "id");
    assert_eq!(node.columns[1].name, "name");
  }

  #[test]
  fn parse_select_with_limit() {
    let node = parse_sql("SELECT id FROM users LIMIT 10", test_location()).unwrap();
    assert!(node.limit);
  }

  #[test]
  fn parse_select_without_limit() {
    let node = parse_sql("SELECT id FROM users", test_location()).unwrap();
    assert!(!node.limit);
  }

  #[test]
  fn parse_select_with_where() {
    let node = parse_sql("SELECT id FROM users WHERE status = 'active'", test_location()).unwrap();
    assert!(node.where_clause);
  }

  #[test]
  fn parse_select_without_where() {
    let node = parse_sql("SELECT id FROM users", test_location()).unwrap();
    assert!(!node.where_clause);
  }

  #[test]
  fn parse_invalid_sql() {
    let result = parse_sql("SELECT", test_location());
    assert!(result.is_err());
  }

  #[test]
  fn parse_unsupported_statement() {
    let result = parse_sql("INSERT INTO users (id) VALUES (1)", test_location());
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), SqlParseError::UnsupportedStatement(_)));
  }

  #[test]
  fn parse_qualified_wildcard() {
    let node = parse_sql("SELECT users.* FROM users", test_location()).unwrap();
    assert!(node.is_select_star());
    assert_eq!(node.columns.len(), 1);
    assert_eq!(node.columns[0].name, "*");
    assert_eq!(node.columns[0].table, Some("users".to_string()));
  }

  #[test]
  fn parse_select_with_alias() {
    let node = parse_sql("SELECT id AS user_id FROM users", test_location()).unwrap();
    assert!(!node.is_select_star());
    assert_eq!(node.columns.len(), 1);
    // sqlparser normalizes ExprWithAlias to use the expression name
    assert_eq!(node.columns[0].name, "id");
  }

  #[test]
  fn parse_select_with_distinct() {
    let node = parse_sql("SELECT DISTINCT id FROM users", test_location()).unwrap();
    assert!(!node.is_select_star());
    assert_eq!(node.columns.len(), 1);
    assert_eq!(node.columns[0].name, "id");
  }

  #[test]
  fn parse_select_with_table_alias() {
    let node = parse_sql("SELECT id FROM users AS u", test_location()).unwrap();
    assert_eq!(node.table.as_ref().unwrap().name, "users");
    assert_eq!(node.table.as_ref().unwrap().alias, Some("u".to_string()));
  }

  #[test]
  fn parse_empty_sql() {
    let result = parse_sql("", test_location());
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), SqlParseError::ParseError(_)));
  }

  #[test]
  fn parse_select_with_where_and_limit() {
    let node =
      parse_sql("SELECT id FROM users WHERE active = true LIMIT 10", test_location()).unwrap();
    assert!(node.where_clause);
    assert!(node.limit);
  }

  #[test]
  fn parse_select_without_table() {
    // SELECT without FROM is valid in PostgreSQL (e.g., SELECT 1)
    let node = parse_sql("SELECT 1", test_location()).unwrap();
    assert!(!node.is_select_star());
    assert_eq!(node.columns.len(), 1);
    assert_eq!(node.columns[0].name, "1");
    assert!(node.table.is_none());
  }
}
