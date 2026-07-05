#![no_main]

use libfuzzer_sys::fuzz_target;
use pulsar_core::SourceLocation;
use pulsar_ir::IrGraph;

/// Build a rule engine with all available rules.
fn all_rules_engine() -> pulsar_rules::RuleEngine {
  let mut engine = pulsar_rules::RuleEngine::new();
  engine.register(Box::new(pulsar_rules::rules::NoSelectStar));
  engine.register(Box::new(pulsar_rules::rules::NoMissingLimit));
  engine.register(Box::new(pulsar_rules::rules::NoUnboundedFind));
  engine.register(Box::new(pulsar_rules::rules::NoAlwaysTrueWhere));
  engine.register(Box::new(pulsar_rules::rules::NoQueryInLoop));
  engine.register(Box::new(pulsar_rules::rules::NoQueryInCallback));
  engine.register(Box::new(pulsar_rules::rules::NoNPlusOne));
  engine.register(Box::new(pulsar_rules::rules::NoRawSqlDangerous));
  engine.register(Box::new(pulsar_rules::rules::NoMissingAwait));
  engine.register(Box::new(pulsar_rules::rules::NoUnindexedFilter));
  engine.register(Box::new(pulsar_rules::rules::NoUnknownColumn));
  engine.register(Box::new(pulsar_rules::rules::NoMissingForeignKey));
  engine
}

fuzz_target!(|data: &[u8]| {
  if let Ok(source) = std::str::from_utf8(data) {
    let loc = SourceLocation { file: String::new(), line: 1, column: 1, span: None };

    // Try as SQL → build graph + run rules
    if let Ok(sql_node) = pulsar_frontend_sql::parse_sql(source, loc) {
      let mut graph = IrGraph::new();
      let id = graph.add_sql(sql_node);
      // try to link to a schema if a table is referenced
      let table_name = graph
        .node(id)
        .and_then(|n| match n {
          pulsar_ir::NodeKind::Sql(sqln) => sqln.table.as_ref(),
          _ => None,
        })
        .map(|t| t.name.clone());
      if let Some(name) = table_name {
        let schema =
          pulsar_frontend_prisma::parse_prisma_schema(&format!("model {name} {{ id Int @id }}"));
        if let Ok(s) = schema {
          graph.load_schema(s);
          graph.link_sql_to_schema(id, &name);
        }
      }
      let engine = all_rules_engine();
      let _ = engine.run(&graph, source, "fuzz.ts");
    }

    // Try as TypeScript → extract + run rules
    let source_type = oxc::span::SourceType::ts();
    if let Ok(graph) = pulsar_frontend_oxc::extract(source, source_type, "fuzz.ts") {
      let engine = all_rules_engine();
      let _ = engine.run(&graph, source, "fuzz.ts");
    }
  }
});
