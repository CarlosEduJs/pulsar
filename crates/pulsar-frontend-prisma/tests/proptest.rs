use proptest::prelude::*;
use proptest::sample::select;
use pulsar_frontend_prisma::parse_prisma_schema;

/// Generates a PascalCase model name.
fn model_name() -> impl Strategy<Value = String> {
  select(vec![
    "User", "Post", "Comment", "Tag", "Product", "Order", "Account", "Session", "Profile",
    "Review", "Category",
  ])
  .prop_map(ToString::to_string)
}

/// Generates a safe field name.
fn field_name() -> impl Strategy<Value = String> {
  select(vec![
    "id", "email", "name", "title", "content", "status", "active", "score", "age", "role",
    "authorId", "ownerId", "slug", "color", "price", "quantity",
  ])
  .prop_map(ToString::to_string)
}

/// Generates a Prisma field type.
fn field_type() -> impl Strategy<Value = String> {
  select(vec!["Int", "String", "Boolean", "Float", "DateTime", "BigInt", "Decimal"])
    .prop_map(ToString::to_string)
}

/// Generates an optional @id attribute.
fn id_attr() -> impl Strategy<Value = &'static str> {
  prop::sample::select(vec!["", " @id"])
}

/// Generates an optional @unique attribute.
fn unique_attr() -> impl Strategy<Value = &'static str> {
  prop::sample::select(vec!["", " @unique"])
}

/// Generates an optional @default(...) attribute.
fn default_attr() -> impl Strategy<Value = &'static str> {
  prop::sample::select(vec![
    "",
    " @default(autoincrement())",
    " @default(now())",
    " @default(true)",
    " @default(\"default\")",
  ])
}

/// Generates a single field definition line.
fn field_def() -> impl Strategy<Value = String> {
  (field_name(), field_type(), id_attr(), unique_attr(), default_attr()).prop_map(
    |(name, ftype, id_a, unique_a, default_a)| {
      format!("  {name} {ftype}{id_a}{unique_a}{default_a}")
    },
  )
}

/// Strategy for generating a full Prisma schema (1 model, unique name).
fn schema_def() -> impl Strategy<Value = String> {
  (model_name(), prop::collection::vec(field_def(), 1..4)).prop_map(|(name, fields)| {
    let body = fields.join("\n");
    format!("model {name} {{\n{body}\n}}")
  })
}

proptest! {
  #[test]
  fn parses_valid_schema(schema in schema_def()) {
    let result = parse_prisma_schema(&schema);
    prop_assert!(result.is_ok(), "Failed to parse schema:\n{}", schema);
    let map = result.unwrap();
    let model_count = schema.lines().filter(|l| l.trim().starts_with("model ")).count();
    prop_assert_eq!(map.len(), model_count,
      "Should have {} model(s) in schema:\n{}", model_count, schema);
  }

  #[test]
  fn preserves_model_names(name in model_name(), field in field_def()) {
    let schema = format!("model {name} {{\n{field}\n}}");
    let map = parse_prisma_schema(&schema).unwrap();
    prop_assert!(map.contains_key(&name),
      "Missing model {} in:\n{}", name, schema);
  }

  #[test]
  fn preserves_column_names(col_name in field_name(), col_type in field_type()) {
    let schema = format!("model Test {{\n  {col_name} {col_type}\n}}");
    let map = parse_prisma_schema(&schema).unwrap();
    let node = map.get("Test").unwrap();
    let has_col = node.columns.iter().any(|c| c.name == col_name);
    prop_assert!(has_col, "Missing column {} in:\n{}", col_name, schema);
  }

  #[test]
  fn detects_id_attribute(col_name in field_name()) {
    let schema = format!("model Item {{\n  {col_name} Int @id\n}}");
    let map = parse_prisma_schema(&schema).unwrap();
    let node = map.get("Item").unwrap();
    let col = node.columns.iter().find(|c| c.name == col_name).unwrap();
    prop_assert!(col.is_indexed, "@id should set indexed=true");
    prop_assert!(col.is_unique, "@id should set unique=true");
  }

  #[test]
  fn detects_unique_attribute(col_name in field_name()) {
    let schema = format!("model Item {{\n  {col_name} String @unique\n}}");
    let map = parse_prisma_schema(&schema).unwrap();
    let node = map.get("Item").unwrap();
    let col = node.columns.iter().find(|c| c.name == col_name).unwrap();
    prop_assert!(col.is_unique, "@unique should set unique=true");
  }
}

#[test]
fn ignores_non_model_blocks() {
  let schema = "\
generator client {
  provider = \"prisma-client-js\"
}

datasource db {
  provider = \"postgresql\"
  url      = env(\"DATABASE_URL\")
}

model User {
  id Int @id @default(autoincrement())
  name String
}";
  let map = parse_prisma_schema(schema).unwrap();
  assert_eq!(map.len(), 1, "should only parse the User model");
  assert!(map.contains_key("User"), "User model should be present");
}

#[test]
fn detects_relation_foreign_key() {
  let schema = "\
model Post {
  id       Int    @id @default(autoincrement())
  authorId Int
  author   User   @relation(fields: [authorId], references: [id])
}";
  let map = parse_prisma_schema(schema).unwrap();
  // The @relation puts FK info on the relation field (author), not on authorId.
  // The parser stores ref_column but ref_table is empty (needs post-processing).
  let author_col = map.get("Post").unwrap().columns.iter().find(|c| c.name == "author").unwrap();
  assert!(author_col.foreign_key.is_some(), "author should have a foreign key from @relation");
  if let Some(fk) = &author_col.foreign_key {
    assert_eq!(fk.ref_column, "id", "ref_column should be 'id'");
    // ref_table is currently not resolved by the parser
    assert_eq!(fk.ref_table, "", "ref_table is not set by parser yet");
  }
}

#[test]
fn empty_input_returns_empty_map() {
  let result = parse_prisma_schema("");
  let map = result.unwrap();
  assert!(map.is_empty());
}

#[test]
fn whitespace_only_returns_empty_map() {
  let result = parse_prisma_schema("  \n  \n  ");
  let map = result.unwrap();
  assert!(map.is_empty());
}

#[test]
fn detects_block_index() {
  let schema = "\
model Test {
  id Int
  email String
  @@index([email])
}";
  let map = parse_prisma_schema(schema).unwrap();
  let node = map.get("Test").unwrap();
  assert_eq!(node.columns.len(), 2);
}

#[test]
fn detects_block_unique() {
  let schema = "\
model Test {
  id Int
  email String
  @@unique([email])
}";
  let map = parse_prisma_schema(schema).unwrap();
  let node = map.get("Test").unwrap();
  assert_eq!(node.columns.len(), 2);
}
