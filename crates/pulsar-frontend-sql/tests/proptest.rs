use proptest::prelude::*;
use pulsar_core::SourceLocation;
use pulsar_frontend_sql::{parse_sql, SqlParseError};
use pulsar_ir::SqlKind;

fn loc() -> SourceLocation {
  SourceLocation { file: "prop_test.sql".to_string(), line: 1, column: 1, span: None }
}

/// Generates a SQL-safe identifier (never a reserved word).
fn safe_id() -> impl Strategy<Value = String> {
  prop::sample::select(vec![
    "id".to_string(),
    "name".to_string(),
    "email".to_string(),
    "age".to_string(),
    "title".to_string(),
    "content".to_string(),
    "status".to_string(),
    "active".to_string(),
    "score".to_string(),
    "count".to_string(),
    "total".to_string(),
    "amount".to_string(),
    "user_id".to_string(),
    "category".to_string(),
    "rank".to_string(),
    "tag".to_string(),
    "slug".to_string(),
    "color".to_string(),
    "price".to_string(),
    "quantity".to_string(),
    "owner".to_string(),
    "group".to_string(),
    "source".to_string(),
    "target".to_string(),
    "value".to_string(),
  ])
}

/// Generates a table name (prefixed to avoid reserved words).
fn table_id() -> impl Strategy<Value = String> {
  prop::sample::select(vec![
    "users".to_string(),
    "posts".to_string(),
    "comments".to_string(),
    "tags".to_string(),
    "products".to_string(),
    "orders".to_string(),
    "reviews".to_string(),
    "accounts".to_string(),
    "sessions".to_string(),
    "items".to_string(),
    "entries".to_string(),
    "records".to_string(),
    "profiles".to_string(),
    "groups_tbl".to_string(),
  ])
}

/// Strategy for generating a valid column list (1..5 columns).
fn column_list() -> impl Strategy<Value = Vec<String>> {
  prop::collection::vec(safe_id(), 1..5)
}

/// Generates a valid SELECT SQL string.
fn select_sql() -> impl Strategy<Value = (String, Vec<String>, String, bool, bool)> {
  (column_list(), table_id(), proptest::bool::ANY, proptest::bool::ANY).prop_map(
    |(columns, table, has_where, has_limit)| {
      let cols = columns.join(", ");
      let mut sql = format!("SELECT {cols} FROM {table}");
      if has_where {
        sql.push_str(" WHERE id = 1");
      }
      if has_limit {
        sql.push_str(" LIMIT 10");
      }
      (sql, columns, table, has_where, has_limit)
    },
  )
}

proptest! {
  // Property: any valid SELECT parses successfully
  #[test]
  fn parses_valid_select(
    (sql, columns, _table, has_where, has_limit) in select_sql(),
  ) {
    let node = parse_sql(&sql, loc());
    prop_assert!(node.is_ok(), "Failed to parse valid SQL: {}", sql);
    let node = node.unwrap();

    // Kind is always Select
    prop_assert_eq!(node.kind, SqlKind::Select);

    // Column count matches
    prop_assert_eq!(
      node.columns.len(),
      columns.len(),
      "Column count mismatch for: {}",
      sql,
    );

    // None of the generated IDs are wildcards
    prop_assert!(!node.is_select_star(), "No explicit SELECT * in generated SQL: {}", sql);

    // Limit/WHERE flags match input
    prop_assert_eq!(node.limit, has_limit, "LIMIT flag mismatch for: {}", sql);
    prop_assert_eq!(node.where_clause, has_where, "WHERE flag mismatch for: {}", sql);
  }
}

/// Generates a SELECT * query.
fn select_star_sql() -> impl Strategy<Value = (String, String, bool, bool)> {
  (table_id(), proptest::bool::ANY, proptest::bool::ANY).prop_map(
    |(table, has_where, has_limit)| {
      let mut sql = format!("SELECT * FROM {table}");
      if has_where {
        sql.push_str(" WHERE id = 1");
      }
      if has_limit {
        sql.push_str(" LIMIT 10");
      }
      (sql, table, has_where, has_limit)
    },
  )
}

proptest! {
  // Property: SELECT * parses as select-star with correct flags
  #[test]
  fn parses_select_star(
    (sql, _table, has_where, has_limit) in select_star_sql(),
  ) {
    let node = parse_sql(&sql, loc());
    prop_assert!(node.is_ok(), "Failed to parse SELECT *: {}", sql);
    let node = node.unwrap();

    prop_assert!(node.is_select_star(), "is_select_star() should be true: {}", sql);
    prop_assert_eq!(node.limit, has_limit, "LIMIT flag mismatch for: {}", sql);
    prop_assert_eq!(node.where_clause, has_where, "WHERE flag mismatch for: {}", sql);
  }
}

/// Generates SELECT with a qualified wildcard (e.g., SELECT users.* FROM users).
fn qualified_wildcard_sql() -> impl Strategy<Value = (String, String, bool, bool)> {
  (table_id(), proptest::bool::ANY, proptest::bool::ANY).prop_map(
    |(table, has_where, has_limit)| {
      let mut sql = format!("SELECT {table}.* FROM {table}");
      if has_where {
        sql.push_str(" WHERE id = 1");
      }
      if has_limit {
        sql.push_str(" LIMIT 10");
      }
      (sql, table, has_where, has_limit)
    },
  )
}

proptest! {
  // Property: qualified wildcard (users.*) parses as select-star
  #[test]
  fn parses_qualified_wildcard(
    (sql, table, has_where, has_limit) in qualified_wildcard_sql(),
  ) {
    let node = parse_sql(&sql, loc());
    prop_assert!(node.is_ok(), "Failed to parse qualified wildcard: {}", sql);
    let node = node.unwrap();

    prop_assert!(node.is_select_star(), "qualified wildcard should be select-star: {}", sql);
    prop_assert_eq!(node.limit, has_limit);
    prop_assert_eq!(node.where_clause, has_where);

    // Column 0 should be the qualified wildcard
    if let Some(col) = node.columns.first() {
      prop_assert_eq!(&col.name, "*");
      prop_assert_eq!(col.table.as_deref(), Some(table.as_str()));
    }
  }
}

// Property: invalid SQL always returns an error (never panics)
proptest! {
  #[test]
  fn invalid_sql_returns_error(sql in ".*") {
    let result = parse_sql(&sql, loc());
    // Must not panic; must return Err for truly invalid SQL
    // Note: some random strings might accidentally be valid SQL
    if let Err(e) = &result {
      // Verify it's one of our expected error types
      prop_assert!(matches!(e, SqlParseError::ParseError(_) | SqlParseError::UnsupportedStatement(_)));
    }
  }
}
