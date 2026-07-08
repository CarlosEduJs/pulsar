/// Strips string literal contents (between double or single quotes) to avoid parsing
/// dots inside string values as `table.column` separators.
fn strip_string_literals(s: &str) -> String {
  let mut result = String::with_capacity(s.len());
  let mut in_string = false;
  let mut quote_char = '"';
  let mut chars = s.chars();
  while let Some(c) = chars.next() {
    if c == '"' || c == '\'' {
      if !in_string {
        in_string = true;
        quote_char = c;
      } else if c == quote_char {
        in_string = false;
      }
      // else: different quote inside a string — keep it (it's part of the content)
    } else if c == '\\' && in_string {
      // Skip escaped character (e.g. \")
      chars.next();
    } else if !in_string {
      result.push(c);
    }
  }
  result
}

/// Extracts column references from a where clause string like `eq(users.id, 1)`.
/// Strips string literal contents first so dots inside strings (e.g. `"test@test.com"`)
/// are not misparsed as column references.
/// Returns `(table_name, column_name)` pairs.
pub fn extract_where_columns(where_clause: &str) -> Vec<(Option<String>, String)> {
  let cleaned = strip_string_literals(where_clause);
  let mut cols = Vec::new();
  let mut remaining = cleaned.as_str();

  while let Some(dot_pos) = remaining.find('.') {
    let before = &remaining[..dot_pos];
    let table_start = before.rfind(|c: char| !c.is_alphanumeric() && c != '_').map_or(0, |p| p + 1);
    let table = before[table_start..].to_string();

    let after = &remaining[dot_pos + 1..];
    let col_end = after.find(|c: char| !c.is_alphanumeric() && c != '_').unwrap_or(after.len());
    let column = after[..col_end].to_string();

    if !table.is_empty() && !column.is_empty() {
      cols.push((Some(table), column));
    }

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

  #[test]
  fn ignores_dots_inside_string_literals() {
    let cols = extract_where_columns("eq(users.email, \"test@test.com\")");
    assert_eq!(cols, vec![(Some("users".to_string()), "email".to_string())]);
  }

  #[test]
  fn mixed_column_refs_and_strings() {
    let cols =
      extract_where_columns("and(eq(posts.authorId, 1), eq(posts.title, \"hello.world\"))");
    assert_eq!(
      cols,
      vec![
        (Some("posts".to_string()), "authorId".to_string()),
        (Some("posts".to_string()), "title".to_string()),
      ]
    );
  }

  #[test]
  fn handles_escaped_quotes_in_string_literals() {
    let cols = extract_where_columns("eq(users.name, \"a\\\"b\")");
    assert_eq!(cols, vec![(Some("users".to_string()), "name".to_string())]);
  }

  // Regression: Bug #4 — single-quoted strings with dots should not be parsed as column refs
  #[test]
  fn ignores_dots_inside_single_quoted_strings() {
    let cols = extract_where_columns("eq(users.name, 'test.test')");
    assert_eq!(
      cols,
      vec![(Some("users".to_string()), "name".to_string())],
      "single-quoted 'test.test' should not be parsed as a column reference"
    );
  }

  // Regression: Bug #7 — columns from other tables should be distinguishable
  #[test]
  fn extracts_cross_table_columns_separately() {
    let cols = extract_where_columns("eq(posts.authorId, users.id)");
    assert_eq!(
      cols,
      vec![
        (Some("posts".to_string()), "authorId".to_string()),
        (Some("users".to_string()), "id".to_string()),
      ],
      "columns from posts and users should both be extracted with their table qualifiers"
    );
  }

  // Regression: Bug #7 — extrai colunas de outras tabelas (whitespace variation)
  #[test]
  fn extracts_columns_from_joined_tables() {
    let cols = extract_where_columns("and(eq(users.id, posts.author_id), eq(posts.id, 1))");
    assert_eq!(
      cols,
      vec![
        (Some("users".to_string()), "id".to_string()),
        (Some("posts".to_string()), "author_id".to_string()),
        (Some("posts".to_string()), "id".to_string()),
      ],
      "columns from different tables should all be extracted"
    );
  }
}
