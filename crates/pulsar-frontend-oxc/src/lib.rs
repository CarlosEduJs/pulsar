#![allow(clippy::multiple_crate_versions, clippy::cast_possible_truncation, clippy::cast_sign_loss)]

use oxc::allocator::Allocator;
use oxc::ast::ast::{
  Argument, CallExpression, Expression, ObjectExpression, ObjectPropertyKind, PropertyKey,
  Statement,
};
use oxc::parser::Parser;
use oxc::span::SourceType;
use pulsar_core::SourceLocation;
use pulsar_ir::{IrGraph, LoopKind, RawSqlKind, RawSqlNode};

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

/// Traversal context propagated through AST extraction.
#[derive(Clone, Copy)]
struct ExtractContext {
  loop_kind: LoopKind,
  in_callback: bool,
}

impl ExtractContext {
  const fn new() -> Self {
    Self { loop_kind: LoopKind::None, in_callback: false }
  }

  const fn with_loop(self, kind: LoopKind) -> Self {
    Self { loop_kind: kind, ..self }
  }

  const fn with_callback(self, val: bool) -> Self {
    Self { in_callback: val, ..self }
  }
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
  extract_from_statement_with_ctx(stmt, source, file_path, graph, ExtractContext::new());
}

fn extract_from_statement_with_ctx<'a>(
  stmt: &'a Statement<'a>,
  source: &'a str,
  file_path: &str,
  graph: &mut IrGraph,
  ctx: ExtractContext,
) {
  match stmt {
    Statement::ExpressionStatement(expr_stmt) => {
      try_extract_from_expr(&expr_stmt.expression, source, file_path, graph, ctx);
    }
    Statement::VariableDeclaration(decl) => {
      for declarator in &decl.declarations {
        if let Some(init) = &declarator.init {
          try_extract_from_expr(init, source, file_path, graph, ctx);
        }
      }
    }
    Statement::ForStatement(_) | Statement::WhileStatement(_) | Statement::DoWhileStatement(_) => {
      handle_loop_body(
        find_loop_body(stmt),
        source,
        file_path,
        graph,
        ctx.with_loop(LoopKind::Counter),
      );
    }
    Statement::ForInStatement(_) | Statement::ForOfStatement(_) => {
      handle_loop_body(
        find_loop_body(stmt),
        source,
        file_path,
        graph,
        ctx.with_loop(LoopKind::Iteration),
      );
    }
    Statement::IfStatement(if_stmt) => {
      extract_from_statement_with_ctx(&if_stmt.consequent, source, file_path, graph, ctx);
      if let Some(alt) = &if_stmt.alternate {
        extract_from_statement_with_ctx(alt, source, file_path, graph, ctx);
      }
    }
    Statement::BlockStatement(block) => {
      for s in &block.body {
        extract_from_statement_with_ctx(s, source, file_path, graph, ctx);
      }
    }
    _ => {}
  }
}

/// Returns the body of a loop statement.
fn find_loop_body<'a>(stmt: &'a Statement<'a>) -> &'a Statement<'a> {
  match stmt {
    Statement::ForStatement(s) => &s.body,
    Statement::ForInStatement(s) => &s.body,
    Statement::ForOfStatement(s) => &s.body,
    Statement::WhileStatement(s) => &s.body,
    Statement::DoWhileStatement(s) => &s.body,
    _ => unreachable!("not a loop statement"),
  }
}

fn handle_loop_body<'a>(
  stmt: &'a Statement<'a>,
  source: &'a str,
  file_path: &str,
  graph: &mut IrGraph,
  ctx: ExtractContext,
) {
  match stmt {
    Statement::BlockStatement(block) => {
      for s in &block.body {
        extract_from_statement_with_ctx(s, source, file_path, graph, ctx);
      }
    }
    other => {
      extract_from_statement_with_ctx(other, source, file_path, graph, ctx);
    }
  }
}

/// Callback-taking methods that trigger `in_callback` context.
const CALLBACK_METHODS: &[&str] =
  &["then", "catch", "finally", "map", "filter", "forEach", "reduce", "flatMap"];

/// Standalone functions whose first argument is a callback.
const CALLBACK_FUNCTIONS: &[&str] = &["setTimeout", "setInterval"];

/// If `expr` is a call to a known callback-taking function/method, traverse its callback body.
fn try_extract_from_callback<'a>(
  expr: &'a Expression<'a>,
  source: &'a str,
  file_path: &str,
  graph: &mut IrGraph,
  ctx: ExtractContext,
) {
  let Expression::CallExpression(call_expr) = strip_await(expr) else { return };

  let callback_arg = match &call_expr.callee {
    Expression::Identifier(ident) if CALLBACK_FUNCTIONS.contains(&ident.name.as_str()) => {
      call_expr.arguments.first().and_then(arg_as_expr)
    }
    Expression::StaticMemberExpression(member)
      if CALLBACK_METHODS.contains(&member.property.name.as_str()) =>
    {
      call_expr.arguments.first().and_then(arg_as_expr)
    }
    _ => None,
  };

  let Some(callback_expr) = callback_arg else { return };

  let stmts = match callback_expr {
    Expression::ArrowFunctionExpression(arrow) => &arrow.body.statements,
    Expression::FunctionExpression(func) => match &func.body {
      Some(body) => &body.statements,
      None => return,
    },
    _ => return,
  };

  let cb_ctx = ctx.with_callback(true);
  for stmt in stmts {
    extract_from_statement_with_ctx(stmt, source, file_path, graph, cb_ctx);
  }
}

/// Methods on `db` that execute raw SQL.
const RAW_DB_METHODS: &[&str] = &["execute", "all", "get", "run"];

/// Detects raw SQL usage and adds [`RawSqlNode`]s to the graph.
fn try_extract_raw_sql<'a>(
  expr: &'a Expression<'a>,
  source: &'a str,
  file_path: &str,
  graph: &mut IrGraph,
) {
  let inner = strip_await(expr);

  match inner {
    Expression::TaggedTemplateExpression(tagged) => {
      if let Expression::Identifier(ident) = &tagged.tag {
        if ident.name.as_str() == "sql" {
          let has_interpolation = !tagged.quasi.expressions.is_empty();
          let location = span_to_location(tagged.span, source, file_path);
          graph.add_raw_sql(RawSqlNode {
            kind: RawSqlKind::TaggedTemplate,
            has_interpolation,
            location,
          });
        }
      }
    }
    Expression::CallExpression(call) => {
      if let Expression::StaticMemberExpression(member) = &call.callee {
        if let Expression::Identifier(obj) = &member.object {
          if obj.name.as_str() == "db" && RAW_DB_METHODS.contains(&member.property.name.as_str()) {
            let has_interpolation = call.arguments.iter().any(|arg| {
              arg_as_expr(arg).is_some_and(|e| match e {
                Expression::StringLiteral(_) => false,
                Expression::TemplateLiteral(t) => !t.expressions.is_empty(),
                _ => true,
              })
            });
            let location = span_to_location(call.span, source, file_path);
            graph.add_raw_sql(RawSqlNode {
              kind: RawSqlKind::DbRawMethod,
              has_interpolation,
              location,
            });
          }
        }
      }
    }
    _ => {}
  }
}

/// Entry point for extracting raw SQL, Drizzle chains, and callbacks from an expression.
fn try_extract_from_expr<'a>(
  expr: &'a Expression<'a>,
  source: &'a str,
  file_path: &str,
  graph: &mut IrGraph,
  ctx: ExtractContext,
) {
  try_extract_from_callback(expr, source, file_path, graph, ctx);
  try_extract_raw_sql(expr, source, file_path, graph);
  try_extract_chain(expr, source, file_path, graph, ctx);
}

fn strip_await<'a>(expr: &'a Expression<'a>) -> &'a Expression<'a> {
  match expr {
    Expression::AwaitExpression(await_expr) => &await_expr.argument,
    other => other,
  }
}

fn try_extract_chain<'a>(
  expr: &'a Expression<'a>,
  source: &'a str,
  file_path: &str,
  graph: &mut IrGraph,
  ctx: ExtractContext,
) {
  let missing_await = !matches!(expr, Expression::AwaitExpression(_));
  let inner = strip_await(expr);

  if let Expression::CallExpression(call) = inner {
    if let Some(chain) = resolve_chain(call) {
      if is_drizzle_select_chain(&chain) {
        let location = span_to_location(call.span, source, file_path);
        process_drizzle_chain(&chain, location, graph, ctx, missing_await);
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

/// Extracts data from a Drizzle chain and delegates graph construction to [`pulsar_graph`].
fn process_drizzle_chain(
  chain: &[MethodCall],
  location: SourceLocation,
  graph: &mut IrGraph,
  ctx: ExtractContext,
  missing_await: bool,
) {
  let columns = extract_select_columns(chain);
  let table_name = extract_table(chain);
  let limit = extract_limit(chain);
  let where_clause = extract_where(chain);

  pulsar_graph::process_drizzle_chain(
    columns,
    table_name,
    limit,
    where_clause,
    ctx.loop_kind,
    ctx.in_callback,
    missing_await,
    location,
    graph,
  );
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
        assert_eq!(orm.loop_kind, LoopKind::Counter, "for loop should set Counter");
      }
    }
  }

  #[test]
  fn extract_in_while_loop_sets_loop_kind_counter() {
    let source = "\
while (true) {\
  await db.select().from(users);\
}\
";
    let graph = extract(source, ts_source(source), TEST_FILE).unwrap();
    assert_eq!(graph.node_count(), 2);
    for id in graph.node_indices() {
      if let pulsar_ir::NodeKind::Orm(orm) = graph.node(id).unwrap() {
        assert_eq!(orm.loop_kind, LoopKind::Counter, "while loop should set Counter");
      }
    }
  }

  #[test]
  fn extract_in_for_of_loop_sets_loop_kind_iteration() {
    let source = "\
for (const user of users) {\
  await db.select().from(posts);\
}\
";
    let graph = extract(source, ts_source(source), TEST_FILE).unwrap();
    assert_eq!(graph.node_count(), 2);
    for id in graph.node_indices() {
      if let pulsar_ir::NodeKind::Orm(orm) = graph.node(id).unwrap() {
        assert_eq!(orm.loop_kind, LoopKind::Iteration, "for-of loop should set Iteration");
      }
    }
  }

  #[test]
  fn extract_standalone_query_has_loop_kind_none() {
    let source = "await db.select().from(users);";
    let graph = extract(source, ts_source(source), TEST_FILE).unwrap();
    assert_eq!(graph.node_count(), 2);
    for id in graph.node_indices() {
      if let pulsar_ir::NodeKind::Orm(orm) = graph.node(id).unwrap() {
        assert_eq!(orm.loop_kind, LoopKind::None, "standalone query should have loop_kind=None");
      }
    }
  }
}
