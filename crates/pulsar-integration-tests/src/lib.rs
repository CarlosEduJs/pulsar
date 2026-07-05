#![allow(
  clippy::module_name_repetitions,
  clippy::must_use_candidate,
  clippy::missing_panics_doc,
  clippy::missing_errors_doc,
  clippy::cast_possible_truncation,
  clippy::cast_sign_loss,
  clippy::similar_names
)]

use pulsar_core::{Diagnostic, SourceLocation};
use pulsar_ir::IrGraph;
use pulsar_rules::rules::*;
use pulsar_rules::RuleEngine;

/// Creates a [`RuleEngine`] with all 12 lint rules registered.
pub fn all_rules_engine() -> RuleEngine {
  let mut engine = RuleEngine::new();
  engine.register(Box::new(NoSelectStar));
  engine.register(Box::new(NoMissingLimit));
  engine.register(Box::new(NoUnboundedFind));
  engine.register(Box::new(NoAlwaysTrueWhere));
  engine.register(Box::new(NoQueryInLoop));
  engine.register(Box::new(NoQueryInCallback));
  engine.register(Box::new(NoNPlusOne));
  engine.register(Box::new(NoRawSqlDangerous));
  engine.register(Box::new(NoMissingAwait));
  engine.register(Box::new(NoUnindexedFilter));
  engine.register(Box::new(NoUnknownColumn));
  engine.register(Box::new(NoMissingForeignKey));
  engine
}

/// Convenience wrapper: extract a TypeScript source into an [`IrGraph`].
///
/// # Panics
///
/// Panics if extraction fails (wraps the error in a panic message).
pub fn extract_ts(source: &str) -> IrGraph {
  let source_type = oxc::span::SourceType::ts();
  pulsar_frontend_oxc::extract(source, source_type, "test.ts")
    .expect("TS extraction should succeed")
}

/// Run the full TS → extract → rules pipeline and return diagnostics.
pub fn analyze_ts(source: &str) -> Vec<Diagnostic> {
  let graph = extract_ts(source);
  let engine = all_rules_engine();
  engine.run(&graph, source, "test.ts")
}

/// Load a Prisma schema into an [`IrGraph`].
///
/// # Panics
///
/// Panics if the schema cannot be parsed.
pub fn load_schema(graph: &mut IrGraph, prisma_source: &str) {
  let schema =
    pulsar_frontend_prisma::parse_prisma_schema(prisma_source).expect("Prisma schema should parse");
  graph.load_schema(schema);
}

/// Extract TS, load schema, run rules, return diagnostics.
pub fn analyze_ts_with_schema(ts_source: &str, prisma_source: &str) -> Vec<Diagnostic> {
  let mut graph = extract_ts(ts_source);
  load_schema(&mut graph, prisma_source);
  let engine = all_rules_engine();
  engine.run(&graph, ts_source, "test.ts")
}

/// Extract diagnostics for a SQL snippet (no schema linking).
///
/// # Panics
///
/// Panics if the SQL cannot be parsed.
pub fn analyze_sql(sql: &str) -> Vec<Diagnostic> {
  let loc = SourceLocation { file: "test.sql".to_string(), line: 1, column: 1, span: None };
  let sql_node = pulsar_frontend_sql::parse_sql(sql, loc).expect("SQL should parse");
  let mut graph = IrGraph::new();
  graph.add_sql(sql_node);
  let engine = all_rules_engine();
  engine.run(&graph, sql, "test.sql")
}
