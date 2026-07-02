#![allow(clippy::multiple_crate_versions, clippy::cast_possible_truncation, clippy::cast_sign_loss)]

use oxc::allocator::Allocator;
use oxc::ast::ast::{
  Argument, CallExpression, Expression, ObjectExpression, ObjectPropertyKind, PropertyKey,
  Statement,
};
use oxc::parser::Parser;
use oxc::span::SourceType;
use pulsar_core::SourceLocation;
use pulsar_ir::{
  ColumnRef, EdgeKind, IrGraph, OrmArgs, OrmMethod, OrmNode, SQLNode, SqlKind, TableRef,
};

/// Errors that can occur during Oxc extraction.
#[derive(Debug, thiserror::Error)]
pub enum ExtractError {
  /// Parsing the source file failed.
  #[error("parse error: {0}")]
  ParseError(String),
}

/// A single method call in a Drizzle chain, e.g. `.select()`, `.from(users)`.
struct MethodCall<'a> {
  name: &'a str,
  args: &'a [Argument<'a>],
}

/// Extracts Drizzle ORM queries from TypeScript source code and populates an [`IrGraph`].
///
/// # Errors
///
/// Returns [`ExtractError`] if parsing fails.
pub fn extract(
  source_text: &str,
  source_type: SourceType,
  file_path: &str,
) -> Result<IrGraph, ExtractError> {
  let allocator = Allocator::default();
  let ret = Parser::new(&allocator, source_text, source_type).parse();

  if let Some(err) = ret.errors.into_iter().next() {
    return Err(ExtractError::ParseError(err.to_string()));
  }

  let mut graph = IrGraph::new();

  for stmt in &ret.program.body {
    extract_from_statement(stmt, source_text, file_path, &mut graph);
  }

  Ok(graph)
}

fn extract_from_statement<'a>(
  stmt: &'a Statement<'a>,
  source: &'a str,
  file_path: &str,
  graph: &mut IrGraph,
) {
  extract_from_statement_with_loop(stmt, source, file_path, graph, false);
}

fn extract_from_statement_with_loop<'a>(
  stmt: &'a Statement<'a>,
  source: &'a str,
  file_path: &str,
  graph: &mut IrGraph,
  in_loop: bool,
) {
  match stmt {
    Statement::ExpressionStatement(expr_stmt) => {
      try_extract_chain(&expr_stmt.expression, source, file_path, graph, in_loop);
    }
    Statement::VariableDeclaration(decl) => {
      for declarator in &decl.declarations {
        if let Some(init) = &declarator.init {
          try_extract_chain(init, source, file_path, graph, in_loop);
        }
      }
    }
    Statement::ForStatement(for_stmt) => {
      handle_loop_body(&for_stmt.body, source, file_path, graph);
    }
    Statement::ForInStatement(for_in_stmt) => {
      handle_loop_body(&for_in_stmt.body, source, file_path, graph);
    }
    Statement::ForOfStatement(for_of_stmt) => {
      handle_loop_body(&for_of_stmt.body, source, file_path, graph);
    }
    Statement::WhileStatement(while_stmt) => {
      handle_loop_body(&while_stmt.body, source, file_path, graph);
    }
    Statement::DoWhileStatement(do_while_stmt) => {
      handle_loop_body(&do_while_stmt.body, source, file_path, graph);
    }
    Statement::IfStatement(if_stmt) => {
      extract_from_statement_with_loop(&if_stmt.consequent, source, file_path, graph, in_loop);
      if let Some(alt) = &if_stmt.alternate {
        extract_from_statement_with_loop(alt, source, file_path, graph, in_loop);
      }
    }
    Statement::BlockStatement(block) => {
      for s in &block.body {
        extract_from_statement_with_loop(s, source, file_path, graph, in_loop);
      }
    }
    _ => {}
  }
}

fn handle_loop_body<'a>(
  stmt: &'a Statement<'a>,
  source: &'a str,
  file_path: &str,
  graph: &mut IrGraph,
) {
  match stmt {
    Statement::BlockStatement(block) => {
      for s in &block.body {
        extract_from_statement_with_loop(s, source, file_path, graph, true);
      }
    }
    other => {
      extract_from_statement_with_loop(other, source, file_path, graph, true);
    }
  }
}

fn try_extract_chain<'a>(
  expr: &'a Expression<'a>,
  source: &'a str,
  file_path: &str,
  graph: &mut IrGraph,
  in_loop: bool,
) {
  let inner = match expr {
    Expression::AwaitExpression(await_expr) => &await_expr.argument,
    other => other,
  };

  if let Expression::CallExpression(call) = inner {
    if let Some(chain) = resolve_chain(call) {
      if is_drizzle_select_chain(&chain) {
        let location = span_to_location(call.span, source, file_path);
        process_drizzle_chain(&chain, location, graph, in_loop);
      }
    }
  }
}

/// Resolves a method chain into a list of `(method_name, arguments)` pairs.
///
/// For `db.select().from(users).where(eq(...))` this returns:
/// `[(select, []), (from, [users]), (where, [eq(...)])]`.
fn resolve_chain<'a>(call: &'a CallExpression<'a>) -> Option<Vec<MethodCall<'a>>> {
  let mut methods = Vec::new();
  let mut current: &'a Expression<'a> = &call.callee;

  // Extract the first method (innermost — furthest from `db`)
  match current {
    Expression::StaticMemberExpression(member) => {
      methods
        .push(MethodCall { name: member.property.name.as_str(), args: call.arguments.as_slice() });
      current = &member.object;
    }
    _ => return None,
  }

  // Walk up the chain — each link is either a CallExpression (chained method)
  // or an Identifier (base, e.g. `db`).
  loop {
    match current {
      Expression::CallExpression(prev_call) => match &prev_call.callee {
        Expression::StaticMemberExpression(member) => {
          methods.push(MethodCall {
            name: member.property.name.as_str(),
            args: prev_call.arguments.as_slice(),
          });
          current = &member.object;
        }
        _ => return None,
      },
      Expression::Identifier(ident) => {
        // Only recognize chains starting with `db`
        if ident.name.as_str() == "db" {
          methods.reverse();
          return Some(methods);
        }
        return None;
      }
      _ => return None,
    }
  }
}

/// Checks whether a resolved chain starts with `db.select(...)`.
fn is_drizzle_select_chain(chain: &[MethodCall]) -> bool {
  chain.first().is_some_and(|m| m.name == "select")
}

/// Converts a Drizzle method chain into ORM and SQL nodes, adding them to the graph.
fn process_drizzle_chain(
  chain: &[MethodCall],
  location: SourceLocation,
  graph: &mut IrGraph,
  in_loop: bool,
) {
  let orm_node = build_orm_node(chain, location.clone(), in_loop);
  let sql_node = build_sql_node(chain, location);

  let orm_id = graph.add_orm(orm_node);
  let sql_id = graph.add_sql(sql_node);
  graph.add_edge(orm_id, sql_id, EdgeKind::Generates);
}

fn build_orm_node(chain: &[MethodCall], location: SourceLocation, in_loop: bool) -> OrmNode {
  let columns = extract_select_columns(chain);
  let limit = extract_limit(chain);
  let where_clause = extract_where(chain);

  OrmNode {
    method: OrmMethod::Select,
    args: OrmArgs { columns, where_clause, limit, include: Vec::new() },
    in_loop,
    location,
  }
}

fn build_sql_node(chain: &[MethodCall], location: SourceLocation) -> SQLNode {
  let columns =
    extract_select_columns(chain).into_iter().map(|c| ColumnRef { name: c, table: None }).collect();
  let table = extract_table(chain).map(|t| TableRef { name: t, alias: None });
  let limit = extract_limit(chain).is_some();
  let where_clause = extract_where(chain).is_some();

  SQLNode { kind: SqlKind::Select, columns, table, limit, where_clause, location }
}

// Argument extraction helpers
// ===========================

/// Extracts column names from `select({ id: ..., name: ... })`.
fn extract_select_columns(chain: &[MethodCall]) -> Vec<String> {
  let select_call = chain.first().expect("chain must start with select");
  let first_arg = select_call.args.first().and_then(arg_as_expr);

  match first_arg {
    Some(Expression::ObjectExpression(obj)) => extract_object_keys(obj),
    Some(_) | None => Vec::new(),
  }
}

/// Extracts the table name from `.from(table)`.
fn extract_table(chain: &[MethodCall]) -> Option<String> {
  chain.iter().find(|m| m.name == "from").and_then(|m| {
    m.args.first().and_then(arg_as_expr).and_then(|e| match e {
      Expression::Identifier(ident) => Some(ident.name.to_string()),
      Expression::StringLiteral(s) => Some(s.value.to_string()),
      _ => None,
    })
  })
}

/// Extracts the limit value from `.limit(n)`.
fn extract_limit(chain: &[MethodCall]) -> Option<u64> {
  chain.iter().find(|m| m.name == "limit").and_then(|m| {
    m.args.first().and_then(arg_as_expr).and_then(|e| match e {
      Expression::NumericLiteral(lit) if lit.value >= 0.0 => Some(lit.value as u64),
      _ => None,
    })
  })
}

/// Extracts the WHERE clause as a string representation.
fn extract_where(chain: &[MethodCall]) -> Option<String> {
  chain.iter().find(|m| m.name == "where").map(|m| {
    m.args.first().map_or_else(
      || "?".to_string(),
      |arg| arg_as_expr(arg).map_or_else(|| "?".to_string(), |e| expr_to_source(e)),
    )
  })
}

/// Converts an expression back to source-like representation.
fn expr_to_source(expr: &Expression) -> String {
  match expr {
    Expression::CallExpression(call) => {
      let callee = expr_to_source(&call.callee);
      let args: Vec<String> = call
        .arguments
        .iter()
        .map(|a| arg_as_expr(a).map_or_else(|| "?".to_string(), |e| expr_to_source(e)))
        .collect();
      format!("{callee}({})", args.join(", "))
    }
    Expression::Identifier(ident) => ident.name.to_string(),
    Expression::StaticMemberExpression(member) => {
      format!("{}.{}", expr_to_source(&member.object), member.property.name.as_str())
    }
    Expression::NumericLiteral(lit) => {
      if lit.value.fract() == 0.0 {
        format!("{}", lit.value as i64)
      } else {
        format!("{}", lit.value)
      }
    }
    Expression::StringLiteral(s) => format!("\"{}\"", s.value.as_str()),
    Expression::BooleanLiteral(b) => b.value.to_string(),
    Expression::NullLiteral(_) => "null".to_string(),
    _ => "?".to_string(),
  }
}

// Low-level helpers
// =================

fn arg_as_expr<'a>(arg: &'a Argument<'a>) -> Option<&'a Expression<'a>> {
  match arg {
    // Argument inherits all Expression variants + SpreadElement
    Argument::SpreadElement(_) => None,
    // Other variants are Expression variants inherited via inherit_variants!
    // Check by trying to access the expression via the generated method
    other => other.as_expression(),
  }
}

fn extract_object_keys(obj: &ObjectExpression) -> Vec<String> {
  obj
    .properties
    .iter()
    .filter_map(|prop| match prop {
      ObjectPropertyKind::ObjectProperty(obj_prop) => match &obj_prop.key {
        PropertyKey::StaticIdentifier(ident) => Some(ident.name.to_string()),
        PropertyKey::StringLiteral(s) => Some(s.value.to_string()),
        _ => None,
      },
      ObjectPropertyKind::SpreadProperty(_) => None,
    })
    .collect()
}

fn span_to_location(span: oxc::span::Span, source: &str, file_path: &str) -> SourceLocation {
  let (line, column) = byte_to_line_col(source, span.start);
  SourceLocation {
    file: file_path.to_string(),
    line,
    column,
    span: Some((span.start as usize, span.end as usize)),
  }
}

fn byte_to_line_col(source: &str, offset: u32) -> (usize, usize) {
  let offset = offset as usize;
  let mut line = 1;
  let mut col = 1;
  for (i, c) in source.char_indices() {
    if i >= offset {
      break;
    }
    if c == '\n' {
      line += 1;
      col = 1;
    } else {
      col += 1;
    }
  }
  (line, col)
}

// Tests
// =====

#[cfg(test)]
mod tests {
  use super::*;

  fn ts_source(_code: &str) -> SourceType {
    SourceType::from_path("test.ts").unwrap()
  }

  const TEST_FILE: &str = "test.ts";

  #[test]
  fn extract_select_star() {
    let source = "const users = await db.select().from(users);";
    let graph = extract(source, ts_source(source), TEST_FILE).unwrap();
    assert_eq!(graph.node_count(), 2, "should have ORM + SQL nodes");
    assert_eq!(graph.edge_count(), 1, "should have Generates edge");
  }

  #[test]
  fn extract_select_with_columns() {
    let source = "const users = await db.select({ id: users.id, name: users.name }).from(users);";
    let graph = extract(source, ts_source(source), TEST_FILE).unwrap();
    assert_eq!(graph.node_count(), 2);
  }

  #[test]
  fn extract_select_with_where() {
    let source = "const user = await db.select().from(users).where(eq(users.id, 1));";
    let graph = extract(source, ts_source(source), TEST_FILE).unwrap();
    assert_eq!(graph.node_count(), 2);
  }

  #[test]
  fn extract_select_with_limit() {
    let source = "const users = await db.select().from(users).limit(10);";
    let graph = extract(source, ts_source(source), TEST_FILE).unwrap();
    assert_eq!(graph.node_count(), 2);
  }

  #[test]
  fn extract_full_chain() {
    let source = "const result = await db.select({ id: users.id }).from(users).where(eq(users.id, 1)).limit(10);";
    let graph = extract(source, ts_source(source), TEST_FILE).unwrap();
    assert_eq!(graph.node_count(), 2);
  }

  #[test]
  fn extract_expression_statement() {
    let source = "db.select().from(users);";
    let graph = extract(source, ts_source(source), TEST_FILE).unwrap();
    assert_eq!(graph.node_count(), 2);
  }

  #[test]
  fn extract_multiple_chains() {
    let source = "\
            const a = await db.select().from(users);\
            const b = await db.select({ id: posts.id }).from(posts);\
        ";
    let graph = extract(source, ts_source(source), TEST_FILE).unwrap();
    assert_eq!(graph.node_count(), 4, "2 ORM + 2 SQL nodes");
    assert_eq!(graph.edge_count(), 2);
  }

  #[test]
  fn skip_non_drizzle_calls() {
    let source = "const x = foo.bar().baz();";
    let graph = extract(source, ts_source(source), TEST_FILE).unwrap();
    assert_eq!(graph.node_count(), 0);
  }

  #[test]
  fn skip_regular_function_calls() {
    let source = "const x = someFunction();";
    let graph = extract(source, ts_source(source), TEST_FILE).unwrap();
    assert_eq!(graph.node_count(), 0);
  }

  #[test]
  fn verify_select_star_detection() {
    let source = "await db.select().from(users);";
    let graph = extract(source, ts_source(source), TEST_FILE).unwrap();
    for id in graph.node_indices() {
      if let pulsar_ir::NodeKind::Sql(sql) = graph.node(id).unwrap() {
        assert!(sql.is_select_star());
        return;
      }
    }
    panic!("expected SQL node");
  }

  #[test]
  fn verify_explicit_columns_not_star() {
    let source = "await db.select({ id: users.id }).from(users);";
    let graph = extract(source, ts_source(source), TEST_FILE).unwrap();
    for id in graph.node_indices() {
      if let pulsar_ir::NodeKind::Sql(sql) = graph.node(id).unwrap() {
        assert!(!sql.is_select_star());
        return;
      }
    }
    panic!("expected SQL node");
  }

  #[test]
  fn extract_empty_source() {
    let graph = extract("", ts_source(""), TEST_FILE).unwrap();
    assert_eq!(graph.node_count(), 0);
  }

  #[test]
  fn extract_invalid_typescript() {
    let result = extract("const x = ;", ts_source("const x = ;"), TEST_FILE);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ExtractError::ParseError(_)));
  }

  #[test]
  fn extract_from_member_expression() {
    let source = "await db.select().from(schema.users);";
    let graph = extract(source, ts_source(source), TEST_FILE).unwrap();
    // Currently member expressions are not resolved as table names
    // so the chain is still extracted but table is None
    assert_eq!(graph.node_count(), 2);
    for id in graph.node_indices() {
      if let pulsar_ir::NodeKind::Sql(sql) = graph.node(id).unwrap() {
        assert!(sql.table.is_none(), "member expr table is not yet resolved");
      }
    }
  }

  #[test]
  fn extract_from_string_literal() {
    let source = "await db.select().from(\"users\");";
    let graph = extract(source, ts_source(source), TEST_FILE).unwrap();
    assert_eq!(graph.node_count(), 2);
    for id in graph.node_indices() {
      if let pulsar_ir::NodeKind::Sql(sql) = graph.node(id).unwrap() {
        assert_eq!(sql.table.as_ref().unwrap().name, "users");
      }
    }
  }

  #[test]
  fn extract_where_without_arguments() {
    let source = "await db.select().from(users).where();";
    let graph = extract(source, ts_source(source), TEST_FILE).unwrap();
    assert_eq!(graph.node_count(), 2);
    for id in graph.node_indices() {
      if let pulsar_ir::NodeKind::Sql(sql) = graph.node(id).unwrap() {
        assert!(sql.where_clause, "where() with no args still counts as where");
      }
    }
  }

  #[test]
  fn extract_limit_with_float() {
    let source = "await db.select().from(users).limit(5.5);";
    let graph = extract(source, ts_source(source), TEST_FILE).unwrap();
    assert_eq!(graph.node_count(), 2);
    for id in graph.node_indices() {
      if let pulsar_ir::NodeKind::Orm(orm) = graph.node(id).unwrap() {
        assert_eq!(orm.args.limit, Some(5));
      }
    }
  }

  #[test]
  fn extract_non_await_chain() {
    let source = "const x = db.select().from(users);";
    let graph = extract(source, ts_source(source), TEST_FILE).unwrap();
    assert_eq!(graph.node_count(), 2);
  }

  #[test]
  fn extract_only_regular_code() {
    let source = "const x = 1 + 2;\nconsole.log(x);\n";
    let graph = extract(source, ts_source(source), TEST_FILE).unwrap();
    assert_eq!(graph.node_count(), 0);
  }

  #[test]
  fn extract_select_with_boolean_and_null() {
    let source = "await db.select({ active: true, deleted: null }).from(users);";
    let graph = extract(source, ts_source(source), TEST_FILE).unwrap();
    assert_eq!(graph.node_count(), 2);
    for id in graph.node_indices() {
      if let pulsar_ir::NodeKind::Orm(orm) = graph.node(id).unwrap() {
        // Both keys are extracted regardless of their values
        assert_eq!(orm.args.columns, vec!["active", "deleted"]);
      }
    }
  }

  #[test]
  fn extract_multiple_different_chains() {
    let source = "\
            await db.select().from(users);\
            await db.select({ id: posts.id }).from(posts).where(eq(posts.id, 1));\
            await db.select().from(comments).limit(5);\
        ";
    let graph = extract(source, ts_source(source), TEST_FILE).unwrap();
    assert_eq!(graph.node_count(), 6, "3 ORM + 3 SQL nodes");
    assert_eq!(graph.edge_count(), 3);
  }

  #[test]
  fn extract_in_for_loop_sets_in_loop_flag() {
    let source = "\
for (let i = 0; i < 10; i++) {\
  await db.select().from(users);\
}\
";
    let graph = extract(source, ts_source(source), TEST_FILE).unwrap();
    assert_eq!(graph.node_count(), 2);
    for id in graph.node_indices() {
      if let pulsar_ir::NodeKind::Orm(orm) = graph.node(id).unwrap() {
        assert!(orm.in_loop, "ORM node inside for loop should have in_loop=true");
      }
    }
  }

  #[test]
  fn extract_in_while_loop_sets_in_loop_flag() {
    let source = "\
while (true) {\
  await db.select().from(users);\
}\
";
    let graph = extract(source, ts_source(source), TEST_FILE).unwrap();
    assert_eq!(graph.node_count(), 2);
    for id in graph.node_indices() {
      if let pulsar_ir::NodeKind::Orm(orm) = graph.node(id).unwrap() {
        assert!(orm.in_loop, "ORM node inside while loop should have in_loop=true");
      }
    }
  }

  #[test]
  fn extract_in_for_of_loop_sets_in_loop_flag() {
    let source = "\
for (const user of users) {\
  await db.select().from(posts);\
}\
";
    let graph = extract(source, ts_source(source), TEST_FILE).unwrap();
    assert_eq!(graph.node_count(), 2);
    for id in graph.node_indices() {
      if let pulsar_ir::NodeKind::Orm(orm) = graph.node(id).unwrap() {
        assert!(orm.in_loop, "ORM node inside for-of loop should have in_loop=true");
      }
    }
  }

  #[test]
  fn extract_standalone_query_has_in_loop_false() {
    let source = "await db.select().from(users);";
    let graph = extract(source, ts_source(source), TEST_FILE).unwrap();
    assert_eq!(graph.node_count(), 2);
    for id in graph.node_indices() {
      if let pulsar_ir::NodeKind::Orm(orm) = graph.node(id).unwrap() {
        assert!(!orm.in_loop, "standalone query should have in_loop=false");
      }
    }
  }
}
