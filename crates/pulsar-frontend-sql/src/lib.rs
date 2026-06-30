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
pub fn parse_sql(sql: &str) -> Result<SQLNode, SqlParseError> {
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

      Ok(SQLNode { kind, columns, table, limit, where_clause })
    }
    other => Err(SqlParseError::UnsupportedStatement(other.to_string())),
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parse_select_star() {
    let node = parse_sql("SELECT * FROM users").unwrap();
    assert!(node.is_select_star());
    assert_eq!(node.columns.len(), 1);
    assert_eq!(node.columns[0].name, "*");
    assert!(node.table.is_some());
    assert_eq!(node.table.as_ref().unwrap().name, "users");
  }

  #[test]
  fn parse_select_explicit_columns() {
    let node = parse_sql("SELECT id, name FROM users").unwrap();
    assert!(!node.is_select_star());
    assert_eq!(node.columns.len(), 2);
    assert_eq!(node.columns[0].name, "id");
    assert_eq!(node.columns[1].name, "name");
  }

  #[test]
  fn parse_select_with_limit() {
    let node = parse_sql("SELECT id FROM users LIMIT 10").unwrap();
    assert!(node.limit);
  }

  #[test]
  fn parse_select_without_limit() {
    let node = parse_sql("SELECT id FROM users").unwrap();
    assert!(!node.limit);
  }

  #[test]
  fn parse_select_with_where() {
    let node = parse_sql("SELECT id FROM users WHERE status = 'active'").unwrap();
    assert!(node.where_clause);
  }

  #[test]
  fn parse_select_without_where() {
    let node = parse_sql("SELECT id FROM users").unwrap();
    assert!(!node.where_clause);
  }

  #[test]
  fn parse_invalid_sql() {
    let result = parse_sql("SELECT");
    assert!(result.is_err());
  }

  #[test]
  fn parse_unsupported_statement() {
    let result = parse_sql("INSERT INTO users (id) VALUES (1)");
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), SqlParseError::UnsupportedStatement(_)));
  }
}
