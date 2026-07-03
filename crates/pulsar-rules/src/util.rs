/// Extracts column references from a where clause string like `eq(users.id, 1)`.
/// Returns `(table_name, column_name)` pairs.
pub fn extract_where_columns(where_clause: &str) -> Vec<(Option<String>, String)> {
  let mut cols = Vec::new();
  let mut remaining = where_clause;

  while let Some(dot_pos) = remaining.find('.') {
    // Look backwards to find the start of the table name
    let before = &remaining[..dot_pos];
    let table_start = before.rfind(|c: char| !c.is_alphanumeric() && c != '_').map_or(0, |p| p + 1);
    let table = before[table_start..].to_string();

    // Look forward to find the end of the column name
    let after = &remaining[dot_pos + 1..];
    let col_end = after.find(|c: char| !c.is_alphanumeric() && c != '_').unwrap_or(after.len());
    let column = after[..col_end].to_string();

    if !table.is_empty() && !column.is_empty() {
      cols.push((Some(table), column));
    }

    // Advance past this match
    let next = dot_pos + 1 + col_end;
    remaining = &remaining[next..];
  }

  cols
}

/// Returns the columns referenced in a where clause, without table qualifiers.
pub fn extract_where_column_names(where_clause: &str) -> Vec<String> {
  extract_where_columns(where_clause).into_iter().map(|(_, col)| col).collect()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn extracts_simple_eq() {
    let cols = extract_where_columns("eq(users.id, 1)");
    assert_eq!(cols, vec![(Some("users".to_string()), "id".to_string())]);
  }

  #[test]
  fn extracts_multiple_columns() {
    let cols = extract_where_columns("and(eq(users.id, 1), eq(users.name, \"foo\"))");
    assert_eq!(
      cols,
      vec![
        (Some("users".to_string()), "id".to_string()),
        (Some("users".to_string()), "name".to_string()),
      ]
    );
  }

  #[test]
  fn empty_string() {
    let cols = extract_where_columns("");
    assert!(cols.is_empty());
  }

  #[test]
  fn no_column_refs() {
    let cols = extract_where_columns("eq(true)");
    assert!(cols.is_empty());
  }
}
