#![allow(clippy::while_let_on_iterator)]

/// Strips string literal contents (between double quotes) to avoid parsing
/// dots inside string values as `table.column` separators.
fn strip_string_literals(s: &str) -> String {
  let mut result = String::with_capacity(s.len());
  let mut chars = s.chars();
  while let Some(c) = chars.next() {
    if c == '"' {
      while let Some(c) = chars.next() {
        if c == '"' {
          break;
        }
      }
    } else {
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
}
