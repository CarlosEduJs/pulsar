#![allow(
  clippy::missing_errors_doc,
  clippy::missing_const_for_fn,
  clippy::needless_raw_string_hashes
)]

use std::collections::HashMap;

use pulsar_ir::{
  ForeignKeyRef, SchemaColumn, SchemaError, SchemaIndex, SchemaNode, SchemaProvider,
};

/// Parses a Prisma schema string and returns schema nodes keyed by table name.
pub fn parse_prisma_schema(source: &str) -> Result<HashMap<String, SchemaNode>, SchemaError> {
  let mut tables: HashMap<String, SchemaNode> = HashMap::new();
  let mut current_model: Option<String> = None;
  let mut current_columns: Vec<SchemaColumn> = Vec::new();
  let mut current_indexes: Vec<SchemaIndex> = Vec::new();
  let mut inside_model = false;

  for line in source.lines() {
    let trimmed = line.trim();

    // Skip empty lines and comments
    if trimmed.is_empty() || trimmed.starts_with("//") {
      continue;
    }

    if trimmed.starts_with("model ") {
      flush_model(&mut current_model, &mut current_columns, &mut current_indexes, &mut tables);
      let name = trimmed
        .strip_prefix("model ")
        .and_then(|s| s.split(['{', ' ', '\t']).next())
        .map(|s| s.trim().to_string());
      current_model = name;
      inside_model = true;
      continue;
    }

    if trimmed == "}" && inside_model {
      flush_model(&mut current_model, &mut current_columns, &mut current_indexes, &mut tables);
      inside_model = false;
      continue;
    }

    if !inside_model {
      continue;
    }

    // Block-level attributes: @@index, @@unique
    if let Some(index) = try_parse_block_index(trimmed) {
      current_indexes.push(index);
      continue;
    }

    if let Some(index) = try_parse_block_unique(trimmed) {
      let cols = index.columns.clone();
      current_indexes.push(index);
      // Also mark individual columns as unique
      for col_name in &cols {
        if let Some(col) = current_columns.iter_mut().find(|c| c.name == *col_name) {
          col.is_unique = true;
        }
      }
      continue;
    }

    // Field line: name type [modifiers] [@attributes]
    if let Some(col) = try_parse_field(trimmed) {
      current_columns.push(col);
    }
  }

  flush_model(&mut current_model, &mut current_columns, &mut current_indexes, &mut tables);

  Ok(tables)
}

fn flush_model(
  name: &mut Option<String>,
  columns: &mut Vec<SchemaColumn>,
  indexes: &mut Vec<SchemaIndex>,
  tables: &mut HashMap<String, SchemaNode>,
) {
  if let Some(table_name) = name.take() {
    let node = SchemaNode {
      table_name: table_name.clone(),
      columns: std::mem::take(columns),
      indexes: std::mem::take(indexes),
    };
    tables.insert(table_name, node);
  }
}

/// Parses `@@index([col1, col2])` or `@@index([col1], name: "my_index")`.
fn try_parse_block_index(line: &str) -> Option<SchemaIndex> {
  let inner = line.strip_prefix("@@index(")?;
  let cols = extract_bracket_list(inner)?;
  Some(SchemaIndex { columns: cols, is_unique: false, is_partial: false })
}

/// Parses `@@unique([col1, col2])`.
fn try_parse_block_unique(line: &str) -> Option<SchemaIndex> {
  let inner = line.strip_prefix("@@unique(")?;
  let cols = extract_bracket_list(inner)?;
  Some(SchemaIndex { columns: cols, is_unique: true, is_partial: false })
}

/// Extracts column names from `[col1, col2, ...]` at the start of a string.
fn extract_bracket_list(s: &str) -> Option<Vec<String>> {
  let s = s.trim();
  let rest = s.strip_prefix('[')?;
  let end = rest.find(']')?;
  let list = &rest[..end];
  let items: Vec<String> = list
    .split(',')
    .map(|s| s.trim().trim_matches('"').to_string())
    .filter(|s| !s.is_empty())
    .collect();
  if items.is_empty() {
    None
  } else {
    Some(items)
  }
}

/// Parses a field declaration line like:
/// `email String @unique @default("")`
/// `id Int @id @default(autoincrement())`
/// `author User @relation(fields: [authorId], references: [id])`
/// `name String?`
fn try_parse_field(line: &str) -> Option<SchemaColumn> {
  // Remove block-level attributes
  if line.starts_with("@@") {
    return None;
  }

  let line = line.trim_end_matches(',');
  let mut parts = line.split_whitespace();

  let name = parts.next().filter(|s| !s.starts_with('@') && !s.starts_with("@@"))?;
  let type_and_mod = parts.next().filter(|s| !s.starts_with('@'))?;

  // Skip relations (Type[])
  if type_and_mod.ends_with("[]") {
    return None;
  }

  let is_nullable = type_and_mod.ends_with('?');
  let col_type = type_and_mod.trim_end_matches('?').to_string();

  let mut is_unique = false;
  let mut is_indexed = false;
  let mut col_default: Option<String> = None;
  let mut foreign_key: Option<ForeignKeyRef> = None;

  let attr_str: String = parts.collect::<Vec<_>>().join(" ");

  // Detect @id (implies indexed + unique)
  if attr_str.contains("@id") {
    is_indexed = true;
    is_unique = true;
  }

  // Detect @unique
  if attr_str.contains("@unique") {
    is_unique = true;
    is_indexed = true;
  }

  // Detect @default(...)
  if let Some(start) = attr_str.find("@default(") {
    let after = &attr_str[start + "@default(".len()..];
    let end = find_matching_paren(after).unwrap_or(0);
    col_default = Some(after[..end].to_string());
  }

  // Detect @relation(fields: [...], references: [...])
  if let Some(start) = attr_str.find("@relation(") {
    let after = &attr_str[start + "@relation(".len()..];
    let end = find_matching_paren(after).unwrap_or(0);
    let inner = &after[..end];
    let fields = extract_named_list(inner, "fields");
    let references = extract_named_list(inner, "references");
    if let (Some(f), Some(r)) =
      (fields.and_then(|v| v.into_iter().next()), references.and_then(|v| v.into_iter().next()))
    {
      foreign_key = Some(ForeignKeyRef { ref_table: String::new(), ref_column: r });
      // Store referenced field in col_default as a marker; ref_table is resolved later
      col_default = Some(format!("fk:{f}"));
    }
  }

  Some(SchemaColumn {
    name: name.to_string(),
    col_type,
    is_nullable,
    is_indexed,
    col_default,
    is_unique,
    foreign_key,
  })
}

/// Finds the matching closing paren, accounting for nested parens.
fn find_matching_paren(s: &str) -> Option<usize> {
  let mut depth = 0;
  for (i, c) in s.char_indices() {
    match c {
      '(' => depth += 1,
      ')' if depth == 0 => return Some(i),
      ')' => depth -= 1,
      _ => {}
    }
  }
  None
}

/// Extracts a named list from a relation argument string.
/// `fields: [authorId, ...]` or `references: [id, ...]`
fn extract_named_list(s: &str, name: &str) -> Option<Vec<String>> {
  let pattern = format!("{name}:");
  let start = s.find(&pattern)?;
  let after = &s[start + pattern.len()..];
  extract_bracket_list(after)
}

/// Schema provider implementation for Prisma schema files.
pub struct PrismaSchemaProvider {
  source: String,
}

impl PrismaSchemaProvider {
  /// Creates a new provider from a `.prisma` file path.
  ///
  /// # Errors
  ///
  /// Returns [`SchemaError::Io`] if the file cannot be read.
  pub fn from_file(path: &str) -> Result<Self, SchemaError> {
    let source = std::fs::read_to_string(path)?;
    Ok(Self { source })
  }

  /// Creates a new provider from a source string (for testing).
  #[must_use]
  pub fn from_source(source: String) -> Self {
    Self { source }
  }
}

impl SchemaProvider for PrismaSchemaProvider {
  fn load(&self) -> Result<HashMap<String, SchemaNode>, SchemaError> {
    parse_prisma_schema(&self.source)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parse_simple_model() {
    let source = r#"
model User {
  id    Int    @id @default(autoincrement())
  email String @unique
  name  String?
}
"#;
    let tables = parse_prisma_schema(source).unwrap();
    assert_eq!(tables.len(), 1);

    let user = tables.get("User").unwrap();
    assert_eq!(user.table_name, "User");
    assert_eq!(user.columns.len(), 3);

    let id = &user.columns[0];
    assert_eq!(id.name, "id");
    assert_eq!(id.col_type, "Int");
    assert!(!id.is_nullable);
    assert!(id.is_indexed);
    assert!(id.is_unique);
    assert_eq!(id.col_default.as_deref(), Some("autoincrement()"));

    let email = &user.columns[1];
    assert_eq!(email.name, "email");
    assert!(email.is_unique);
    assert!(email.is_indexed);
    assert!(!email.is_nullable);

    let name = &user.columns[2];
    assert_eq!(name.name, "name");
    assert!(name.is_nullable);
    assert!(!name.is_unique);
  }

  #[test]
  fn parse_foreign_key_relation() {
    let source = r#"
model Post {
  id        Int  @id @default(autoincrement())
  authorId  Int
  author    User @relation(fields: [authorId], references: [id])
}
"#;
    let tables = parse_prisma_schema(source).unwrap();
    let post = tables.get("Post").unwrap();
    assert_eq!(post.columns.len(), 3);

    let author = &post.columns[2];
    assert_eq!(author.name, "author");
    assert!(author.foreign_key.is_some());
    let fk = author.foreign_key.as_ref().unwrap();
    assert_eq!(fk.ref_column, "id");
    // ref_table is empty until resolution; stored in col_default as marker
    assert!(author.col_default.as_deref().unwrap().starts_with("fk:"));
  }

  #[test]
  fn parse_block_index() {
    let source = r#"
model User {
  id    Int    @id
  email String

  @@index([email])
}
"#;
    let tables = parse_prisma_schema(source).unwrap();
    let user = tables.get("User").unwrap();
    assert_eq!(user.indexes.len(), 1);
    assert_eq!(user.indexes[0].columns, vec!["email"]);
    assert!(!user.indexes[0].is_unique);
  }

  #[test]
  fn parse_block_unique() {
    let source = r#"
model User {
  id    Int    @id
  email String

  @@unique([email, name])
}
"#;
    let tables = parse_prisma_schema(source).unwrap();
    let user = tables.get("User").unwrap();
    assert_eq!(user.indexes.len(), 1);
    assert_eq!(user.indexes[0].columns, vec!["email", "name"]);
    assert!(user.indexes[0].is_unique);
  }

  #[test]
  fn parse_multiple_models() {
    let source = r#"
model User {
  id Int @id
}

model Post {
  id Int @id
}
"#;
    let tables = parse_prisma_schema(source).unwrap();
    assert_eq!(tables.len(), 2);
    assert!(tables.contains_key("User"));
    assert!(tables.contains_key("Post"));
  }

  #[test]
  fn skip_non_model_blocks() {
    let source = r#"
generator client {
  provider = "prisma-client-js"
}

datasource db {
  provider = "postgresql"
  url      = env("DATABASE_URL")
}

model User {
  id Int @id
}
"#;
    let tables = parse_prisma_schema(source).unwrap();
    assert_eq!(tables.len(), 1);
    assert!(tables.contains_key("User"));
  }

  #[test]
  fn ignore_enum() {
    let source = r#"
enum Role {
  USER
  ADMIN
}

model User {
  id Int @id
}
"#;
    let tables = parse_prisma_schema(source).unwrap();
    assert_eq!(tables.len(), 1);
  }

  #[test]
  fn default_value_string() {
    let source = r#"
model User {
  id   Int    @id
  role String @default("user")
}
"#;
    let tables = parse_prisma_schema(source).unwrap();
    let user = tables.get("User").unwrap();
    let role = &user.columns[1];
    assert_eq!(role.col_default.as_deref(), Some("\"user\""));
  }

  #[test]
  fn provider_from_file_not_found() {
    let err = PrismaSchemaProvider::from_file("/nonexistent/schema.prisma");
    assert!(err.is_err());
  }

  #[test]
  fn provider_loads_schema() {
    let source = r#"
model Foo {
  id   Int    @id
  name String
}
"#
    .to_string();
    let provider = PrismaSchemaProvider::from_source(source);
    let tables = provider.load().unwrap();
    assert_eq!(tables.len(), 1);
    let foo = tables.get("Foo").unwrap();
    assert_eq!(foo.columns.len(), 2);
    assert_eq!(foo.columns[0].name, "id");
    assert!(foo.columns[0].is_indexed);
    assert_eq!(foo.columns[1].name, "name");
    assert!(!foo.columns[1].is_indexed);
  }
}
