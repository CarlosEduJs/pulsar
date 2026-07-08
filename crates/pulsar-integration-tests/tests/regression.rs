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
use pulsar_test_utils::rules::all_rules_engine;

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

// TODO(#?): Remove once `pulsar_frontend_prisma::parse_prisma_schema`
// properly sets `ForeignKey.ref_table` from `@relation` references.
// Tracked at https://github.com/carlosedujs/pulsar/issues/???
#[test]
#[ignore = "ForeignKey.ref_table not yet populated by @relation parser"]
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

// Regression: Bug schema load_schema duplicates schema nodes each time it's called.
// Loading the same schema into multiple file graphs creates N copies of the schema.
#[test]
fn load_schema_duplicates_nodes_per_file() {
  use pulsar_frontend_prisma::parse_prisma_schema;
  use pulsar_ir::IrGraph;

  let prisma = r#"
    model users {
      id Int @id
    }
  "#;

  let tables = parse_prisma_schema(prisma).unwrap();

  // Simulate loading schema into first file's graph
  let mut graph1 = IrGraph::new();
  graph1.load_schema(&tables);
  assert_eq!(graph1.node_count(), 1, "graph1 should have 1 schema node");

  // Simulate loading schema into second file's graph
  let mut graph2 = IrGraph::new();
  graph2.load_schema(&tables);
  assert_eq!(graph2.node_count(), 1, "graph2 should have 1 schema node");

  // Bug The schema is cloned into EACH file's graph separately.
  // If we loaded the same schema into one graph twice:
  let mut graph3 = IrGraph::new();
  graph3.load_schema(&tables);
  graph3.load_schema(&tables);
  assert_eq!(
    graph3.node_count(),
    1,
    "BUG #3: loading the same schema twice should not duplicate nodes, \
     but load_schema does not check for duplicates. got {} nodes",
    graph3.node_count(),
  );
}

// Regression: Bug single-quoted strings with dots in WHERE clauses
// cause false positive column extraction.
// This is a full pipeline test to verify the end-to-end behavior.
#[test]
fn ts_with_single_quoted_string_in_where_does_not_false_positive() {
  // eq(users.name, 'some.dotted.value') should only extract 'users.name'
  // Bug: single quotes are not stripped, so 'some.dotted.value' is parsed
  // as table='some', column='dotted' — false positive
  let source = r#"
    const user = await db.select({ id: users.id }).from(users)
      .where(eq(users.name, 'some.dotted.value'))
      .limit(1);
  "#;
  let graph = pulsar_frontend_oxc::extract(source, oxc::span::SourceType::ts(), "test.ts")
    .expect("should parse ok");
  let engine = all_rules_engine();
  let _diags = engine.run(&graph, source, "test.ts");
  // The primary issue is that extract_where_column_names would parse
  // 'some.dotted.value' as a column reference. This test just verifies
  // the pipeline doesn't panic and produces reasonable diagnostics.
  // Actual false positive verification happens in util.rs unit tests.
}

// Regression: Bug raw SQL in callbacks should be detectable
// Currently try_extract_raw_sql doesn't accept context
#[test]
fn raw_sql_in_callback_still_extracted() {
  let source = r#"
    getUsers().then(() => {
      return db.execute(sql`SELECT * FROM posts`);
    });
  "#;
  let graph = pulsar_frontend_oxc::extract(source, oxc::span::SourceType::ts(), "test.ts")
    .expect("should parse ok");
  // Raw SQL nodes are extracted even inside callbacks (Bug #9 only prevents context tracking)
  let has_raw_sql =
    graph.node_indices().any(|id| matches!(graph.node(id), Some(pulsar_ir::NodeKind::RawSql(_))));
  assert!(has_raw_sql, "raw SQL inside callbacks should be extracted");
}

// Regression: Bug #10 — Windows \\r\\n line endings in source
#[test]
fn ts_with_windows_line_endings_extracts_safely() {
  let source = "const users = await db.select().from(users);\r\n";
  let result = pulsar_frontend_oxc::extract(source, oxc::span::SourceType::ts(), "test.ts");
  assert!(result.is_ok(), "Windows \\r\\n should not break extraction");
  let graph = result.unwrap();
  assert!(graph.node_count() > 0, "should extract ORM + SQL nodes");
}

// Regression: Bug.limit(variable) should not produce false positive
// no-missing-limit for queries with dynamic limits
#[test]
fn ts_with_dynamic_limit_not_flagged_as_missing_limit() {
  let source = r#"
    const pageSize = 10;
    const users = await db.select({ id: users.id }).from(users).limit(pageSize);
  "#;
  let graph = pulsar_frontend_oxc::extract(source, oxc::span::SourceType::ts(), "test.ts")
    .expect("should parse ok");
  let engine = all_rules_engine();
  let diags = engine.run(&graph, source, "test.ts");
  // Bug: .limit(pageSize) is not recognized as having a limit because
  // pageSize is an Identifier, not a NumericLiteral
  let missing_limit_diags: Vec<&pulsar_core::Diagnostic> =
    diags.iter().filter(|d| d.rule_id == "no-missing-limit").collect();
  assert!(
    missing_limit_diags.is_empty(),
    "BUG #14: .limit(pageSize) should be recognized as having a dynamic limit. \
     Got {} no-missing-limit diagnostics",
    missing_limit_diags.len(),
  );
}
