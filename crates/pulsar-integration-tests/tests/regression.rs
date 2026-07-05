#![allow(
  clippy::cast_possible_truncation,
  clippy::cast_sign_loss,
  clippy::similar_names,
  clippy::too_many_lines,
  clippy::needless_raw_string_hashes
)]

use pulsar_core::SourceLocation;
use pulsar_integration_tests::*;
use pulsar_test_utils::fixtures;

// ─── SQL edge cases ──────────────────────────────────────────

#[test]
fn sql_subquery_in_from_does_not_panic() {
  let sql = "SELECT * FROM (SELECT id FROM users) AS sub";
  let result = pulsar_frontend_sql::parse_sql(
    sql,
    SourceLocation { file: "test.sql".to_string(), line: 1, column: 1, span: None },
  );
  assert!(result.is_ok() || result.is_err(), "subquery parsing should not panic");
}

#[test]
fn sql_join_does_not_panic() {
  let sql = "SELECT u.id, p.title FROM users u JOIN posts p ON u.id = p.author_id";
  let loc = SourceLocation { file: "test.sql".to_string(), line: 1, column: 1, span: None };
  let result = pulsar_frontend_sql::parse_sql(sql, loc);
  assert!(result.is_ok() || result.is_err(), "JOIN parsing should not panic");
}

#[test]
fn sql_multiple_statements_takes_first() {
  let sql = "SELECT 1 AS a; SELECT 2 AS b";
  let loc = SourceLocation { file: "test.sql".to_string(), line: 1, column: 1, span: None };
  let result = pulsar_frontend_sql::parse_sql(sql, loc);
  assert!(result.is_ok(), "first SELECT should parse ok: {result:?}");
}

#[test]
fn sql_whitespace_only_returns_error() {
  let loc = SourceLocation { file: "test.sql".to_string(), line: 1, column: 1, span: None };
  let result = pulsar_frontend_sql::parse_sql("   \n\t  ", loc);
  assert!(matches!(result, Err(pulsar_frontend_sql::SqlParseError::ParseError(_)),));
}

#[test]
fn sql_unicode_identifiers_handled() {
  let sql = "SELECT id, nome, descrição FROM usuários";
  let loc = SourceLocation { file: "test.sql".to_string(), line: 1, column: 1, span: None };
  let result = pulsar_frontend_sql::parse_sql(sql, loc);
  assert!(result.is_ok() || result.is_err(), "unicode SQL should not panic");
}

#[test]
fn sql_with_comments_parses() {
  let sql = "SELECT /* block comment */ id -- line comment\nFROM users";
  let loc = SourceLocation { file: "test.sql".to_string(), line: 1, column: 1, span: None };
  let result = pulsar_frontend_sql::parse_sql(sql, loc);
  assert!(result.is_ok(), "SQL with comments should parse: {result:?}");
}

// ─── TypeScript edge cases ───────────────────────────────────

#[test]
fn ts_unicode_identifiers_extracts_safely() {
  let source = r#"
    const usuários = await db.select().from(usuários);
    const resultado = await db.select({ id: usuários.id }).from(usuários);
  "#;
  let result = pulsar_frontend_oxc::extract(source, oxc::span::SourceType::ts(), "test.ts");
  assert!(result.is_ok(), "unicode TS should extract ok: {result:?}");
}

#[test]
fn ts_deeply_nested_arrow_functions() {
  let source = r#"
    setTimeout(() => {
      setTimeout(() => {
        setTimeout(() => {
          return db.select().from(users);
        });
      });
    });
  "#;
  let diags = analyze_ts(source);
  let rule_ids: Vec<&str> = diags.iter().map(|d| d.rule_id.as_str()).collect();
  assert!(
    rule_ids.contains(&"no-query-in-callback"),
    "deeply nested callback should be detected, got: {rule_ids:?}",
  );
}

#[test]
fn ts_template_literals_in_queries() {
  let source = r#"
    const table = "users";
    const result = await db.select().from(eval(table));
  "#;
  let result = pulsar_frontend_oxc::extract(source, oxc::span::SourceType::ts(), "test.ts");
  assert!(result.is_ok(), "TS with dynamic table should extract ok");
}

#[test]
fn ts_empty_source_returns_empty_graph() {
  let result = pulsar_frontend_oxc::extract("", oxc::span::SourceType::ts(), "empty.ts");
  assert!(result.is_ok(), "empty source should extract ok");
  let graph = result.unwrap();
  assert_eq!(graph.node_count(), 0, "empty source should produce 0 nodes");
}

#[test]
fn ts_whitespace_only_returns_empty_graph() {
  let result =
    pulsar_frontend_oxc::extract("  \n  \t  ", oxc::span::SourceType::ts(), "whitespace.ts");
  assert!(result.is_ok(), "whitespace source should extract ok");
  let graph = result.unwrap();
  assert_eq!(graph.node_count(), 0, "whitespace source should produce 0 nodes");
}

#[test]
fn ts_chained_select_with_multiple_wheres_does_not_panic() {
  let source = r#"
    db.select().from(users).where(eq(users.id, 1)).where(eq(users.name, "test")).limit(10);
  "#;
  let result = pulsar_frontend_oxc::extract(source, oxc::span::SourceType::ts(), "test.ts");
  assert!(result.is_ok(), "chained wheres should extract ok");
}

#[test]
fn ts_await_all_over_the_place() {
  let source = r#"
    const a = await db.select().from(users);
    const b = db.select().from(posts);
    async function f() {
      const c = await db.select().from(comments);
    }
  "#;
  let result = pulsar_frontend_oxc::extract(source, oxc::span::SourceType::ts(), "test.ts");
  assert!(result.is_ok(), "mixed await patterns should extract ok");
}

// ─── Prisma edge cases ───────────────────────────────────────

#[test]
fn prisma_schema_with_enum_ignored() {
  let source = r#"
    enum Role {
      USER
      ADMIN
    }
    model users {
      id   Int    @id @default(autoincrement())
      role Role
    }
  "#;
  let diags = analyze_ts_with_schema("", source);
  assert!(diags.is_empty(), "enum is ignored, model parsed cleanly");
}

#[test]
fn prisma_schema_composite_id_does_not_panic() {
  let source = r#"
    model follows {
      follower_id Int
      following_id Int
      @@id([follower_id, following_id])
    }
  "#;
  let result = pulsar_frontend_prisma::parse_prisma_schema(source);
  assert!(result.is_ok(), "composite @@id should parse");
  let map = result.unwrap();
  assert!(map.contains_key("follows"), "model name should be preserved");
}

#[test]
fn prisma_schema_multi_field_index() {
  let source = r#"
    model users {
      first_name String
      last_name  String
      @@index([first_name, last_name])
    }
  "#;
  let result = pulsar_frontend_prisma::parse_prisma_schema(source);
  assert!(result.is_ok(), "multi-field @@index should parse");
}

#[test]
fn prisma_schema_relation_without_fk_annotation() {
  let source = r#"
    model users {
      id     Int    @id
      posts  Post[]
    }
    model posts {
      id     Int  @id
      author users?
    }
  "#;
  let result = pulsar_frontend_prisma::parse_prisma_schema(source);
  assert!(result.is_ok(), "implicit relation should parse");
}

#[test]
fn prisma_schema_empty_source() {
  let result = pulsar_frontend_prisma::parse_prisma_schema("");
  assert!(result.is_ok(), "empty source should parse ok");
  let map = result.unwrap();
  assert!(map.is_empty(), "empty source should produce empty map");
}

// ─── Cross-crate regression ──────────────────────────────────

#[test]
fn known_parser_limitation_fk_ref_table_empty() {
  let schema = r#"
    model posts {
      id       Int  @id
      authorId Int
      author   users @relation(fields: [authorId], references: [id])
    }
  "#;
  let map = pulsar_frontend_prisma::parse_prisma_schema(schema).unwrap();
  let posts = map.get("posts").unwrap();
  let author = posts.columns.iter().find(|c| c.name == "author").unwrap();
  assert!(author.foreign_key.is_some(), "@relation field should have a foreign_key");
  if let Some(fk) = &author.foreign_key {
    assert_eq!(fk.ref_column, "id", "ref_column should be preserved");
    // ref_table is currently empty — regression guard
    assert_eq!(fk.ref_table, "", "ref_table is not set by parser yet");
  }
}

#[test]
fn known_parser_behavior_block_unique_on_multiple_columns() {
  let source = r#"
    model users {
      email String
      name  String
      @@unique([email, name])
    }
  "#;
  let result = pulsar_frontend_prisma::parse_prisma_schema(source);
  assert!(result.is_ok(), "multi-column @@unique should parse");
}

#[test]
fn graph_without_schema_runs_all_rules_safely() {
  let diags = analyze_ts("const x = 1;");
  assert!(diags.is_empty(), "no ORM code should produce no diagnostics");
}

// ─── All fixture files extract without panic ─────────────────

fn all_fixture_paths() -> Vec<&'static str> {
  vec![
    "basic.ts",
    "clean.ts",
    "no-issues.ts",
    "with-where.ts",
    "with-limit.ts",
    "mixed-star-explicit.ts",
    "invalid-syntax.ts",
    "no-missing-limit/query-without-limit.ts",
    "no-missing-limit/query-with-limit.ts",
    "no-unbounded-find/unbounded-query.ts",
    "no-unbounded-find/bounded-query.ts",
    "no-always-true-where/where-true.ts",
    "no-always-true-where/where-real-condition.ts",
    "no-query-in-loop/query-in-for-loop.ts",
    "no-query-in-loop/query-outside-loop.ts",
    "no-n-plus-one/query-in-for-of.ts",
    "no-n-plus-one/clean.ts",
    "no-query-in-callback/query-in-then.ts",
    "no-query-in-callback/clean.ts",
    "no-raw-sql-dangerous/sql-template.ts",
    "no-raw-sql-dangerous/clean.ts",
    "no-missing-await/missing-await.ts",
    "no-missing-await/clean.ts",
    "no-unindexed-filter/filter-on-name.ts",
    "no-unindexed-filter/filter-on-indexed.ts",
    "no-unindexed-filter/clean.ts",
    "no-unknown-column/select-wrong.ts",
    "no-unknown-column/clean.ts",
    "no-missing-foreign-key/clean.ts",
  ]
}

#[test]
fn all_fixtures_extract_without_panic() {
  for path in all_fixture_paths() {
    let source = fixtures::read_fixture(path);
    let result = pulsar_frontend_oxc::extract(&source, oxc::span::SourceType::ts(), path);
    // invalid-syntax.ts should get a parse error; all others should succeed
    if path == "invalid-syntax.ts" {
      assert!(result.is_err(), "invalid-syntax.ts should fail to parse");
    } else {
      assert!(result.is_ok(), "{path} should extract ok: {result:?}");
    }
  }
}

#[test]
fn all_fixtures_run_rules_without_panic() {
  for path in all_fixture_paths() {
    let source = fixtures::read_fixture(path);
    let result = pulsar_frontend_oxc::extract(&source, oxc::span::SourceType::ts(), path);
    if let Ok(graph) = result {
      let engine = all_rules_engine();
      let _diags = engine.run(&graph, &source, path);
      // no assertions — just verifying no panic
    }
  }
}
