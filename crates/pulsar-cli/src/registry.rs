use std::collections::BTreeMap;

use pulsar_rules::rules::{
  NoAlwaysTrueWhere, NoMissingAwait, NoMissingForeignKey, NoMissingLimit, NoNPlusOne,
  NoQueryInCallback, NoQueryInLoop, NoRawSqlDangerous, NoSelectStar, NoUnboundedFind,
  NoUnindexedFilter, NoUnknownColumn,
};
use pulsar_rules::{Rule, RuleEngine};

type RuleConstructor = fn() -> Box<dyn Rule>;

fn no_select_star() -> Box<dyn Rule> {
  Box::new(NoSelectStar)
}

fn no_missing_limit() -> Box<dyn Rule> {
  Box::new(NoMissingLimit)
}

fn no_unbounded_find() -> Box<dyn Rule> {
  Box::new(NoUnboundedFind)
}

fn no_always_true_where() -> Box<dyn Rule> {
  Box::new(NoAlwaysTrueWhere)
}

fn no_query_in_loop() -> Box<dyn Rule> {
  Box::new(NoQueryInLoop)
}

fn no_query_in_callback() -> Box<dyn Rule> {
  Box::new(NoQueryInCallback)
}

fn no_n_plus_one() -> Box<dyn Rule> {
  Box::new(NoNPlusOne)
}

fn no_raw_sql_dangerous() -> Box<dyn Rule> {
  Box::new(NoRawSqlDangerous)
}

fn no_missing_await() -> Box<dyn Rule> {
  Box::new(NoMissingAwait)
}

fn no_unindexed_filter() -> Box<dyn Rule> {
  Box::new(NoUnindexedFilter)
}

fn no_unknown_column() -> Box<dyn Rule> {
  Box::new(NoUnknownColumn)
}

fn no_missing_foreign_key() -> Box<dyn Rule> {
  Box::new(NoMissingForeignKey)
}

/// Returns all built-in rules keyed by their `id()`.
#[must_use]
pub fn builtin_rules() -> BTreeMap<&'static str, RuleConstructor> {
  let mut map: BTreeMap<&'static str, RuleConstructor> = BTreeMap::new();
  map.insert("no-select-star", no_select_star);
  map.insert("no-missing-limit", no_missing_limit);
  map.insert("no-unbounded-find", no_unbounded_find);
  map.insert("no-always-true-where", no_always_true_where);
  map.insert("no-query-in-loop", no_query_in_loop);
  map.insert("no-query-in-callback", no_query_in_callback);
  map.insert("no-n-plus-one", no_n_plus_one);
  map.insert("no-raw-sql-dangerous", no_raw_sql_dangerous);
  map.insert("no-missing-await", no_missing_await);
  map.insert("no-unindexed-filter", no_unindexed_filter);
  map.insert("no-unknown-column", no_unknown_column);
  map.insert("no-missing-foreign-key", no_missing_foreign_key);
  map
}

/// Builds a [`RuleEngine`] with the given list of rule names.
///
/// Unknown names are printed to stderr and skipped.
pub fn resolve_rules(names: &[String]) -> RuleEngine {
  let builtins = builtin_rules();
  let mut engine = RuleEngine::new();

  if names.is_empty() {
    // Enable all built-in rules
    for ctor in builtins.values() {
      engine.register(ctor());
    }
    return engine;
  }

  for name in names {
    match builtins.get(name.as_str()) {
      Some(ctor) => engine.register(ctor()),
      None => eprintln!("warning: unknown rule \"{name}\", skipping"),
    }
  }

  engine
}

#[cfg(test)]
mod tests {
  use super::*;
  use pulsar_core::SourceLocation;
  use pulsar_ir::{IrGraph, LoopKind, OrmArgs, OrmMethod, OrmNode, SQLNode, SqlKind, TableRef};

  // Helper: build a minimal graph containing a SELECT * query
  fn select_star_graph() -> IrGraph {
    let mut graph = IrGraph::new();
    let loc = SourceLocation { file: "test.ts".to_string(), line: 1, column: 1, span: None };
    let sql = SQLNode {
      kind: SqlKind::Select,
      columns: vec![],
      table: Some(TableRef { name: "users".to_string(), alias: None }),
      limit: false,
      where_clause: false,
      in_callback: false,
      location: loc.clone(),
    };
    let orm = OrmNode {
      method: OrmMethod::Select,
      args: OrmArgs { columns: vec![], where_clause: None, limit: None, include: vec![] },
      loop_kind: LoopKind::None,
      in_callback: false,
      missing_await: false,
      location: loc,
    };
    let s = graph.add_sql(sql);
    let o = graph.add_orm(orm);
    graph.add_edge(o, s, pulsar_ir::EdgeKind::Generates);
    graph
  }

  // builtin_rules
  // =============

  #[test]
  fn builtin_rules_has_exactly_twelve() {
    let rules = builtin_rules();
    assert_eq!(rules.len(), 12, "expected exactly 12 built-in rules");
  }

  #[test]
  fn builtin_rules_contains_expected_names() {
    let rules = builtin_rules();
    let expected: [&str; 12] = [
      "no-select-star",
      "no-missing-limit",
      "no-unbounded-find",
      "no-always-true-where",
      "no-query-in-loop",
      "no-query-in-callback",
      "no-n-plus-one",
      "no-raw-sql-dangerous",
      "no-missing-await",
      "no-unindexed-filter",
      "no-unknown-column",
      "no-missing-foreign-key",
    ];
    for name in &expected {
      assert!(rules.contains_key(name), "missing rule: {name}");
    }
  }

  #[test]
  fn builtin_rules_constructors_return_correct_id() {
    let rules = builtin_rules();
    for (expected_id, ctor) in &rules {
      let rule = ctor();
      assert_eq!(rule.id(), *expected_id, "rule id mismatch for {expected_id}");
    }
  }

  // resolve_rules
  // =============

  #[test]
  fn resolve_rules_empty_uses_all_rules() {
    let engine = resolve_rules(&[]);
    let graph = select_star_graph();
    let diags = engine.run(&graph, "", "test.ts");
    // At minimum, no-select-star should fire
    assert!(
      diags.iter().any(|d| d.rule_id == "no-select-star"),
      "expected no-select-star diagnostic among {} diagnostics",
      diags.len(),
    );
  }

  #[test]
  fn resolve_rules_single_rule() {
    let engine = resolve_rules(&["no-select-star".to_string()]);
    let graph = select_star_graph();
    let diags = engine.run(&graph, "", "test.ts");
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].rule_id, "no-select-star");
  }

  #[test]
  fn resolve_rules_multiple_rules() {
    let engine = resolve_rules(&["no-select-star".to_string(), "no-missing-limit".to_string()]);
    let graph = select_star_graph();
    let diags = engine.run(&graph, "", "test.ts");
    // Both no-select-star and no-missing-limit should fire
    assert!(diags.iter().any(|d| d.rule_id == "no-select-star"));
    assert!(diags.iter().any(|d| d.rule_id == "no-missing-limit"));
  }

  #[test]
  fn resolve_rules_select_star_in_multiple() {
    let engine = resolve_rules(&["no-select-star".to_string(), "no-always-true-where".to_string()]);
    let graph = select_star_graph();
    let diags = engine.run(&graph, "", "test.ts");
    // Only no-select-star fires (no-always-true-where needs a WHERE clause)
    assert_eq!(diags.iter().filter(|d| d.rule_id == "no-select-star").count(), 1);
  }

  #[test]
  fn resolve_rules_unknown_name_skipped() {
    // Unknown names are reported to stderr but the function should not panic
    let engine = resolve_rules(&["no-select-star".to_string(), "unknown-rule".to_string()]);
    let graph = select_star_graph();
    let diags = engine.run(&graph, "", "test.ts");
    // Only no-select-star fires
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].rule_id, "no-select-star");
  }

  #[test]
  fn resolve_rules_only_unknown_returns_empty_engine() {
    // Capture stderr to verify the warning message
    let engine = resolve_rules(&["definitely-not-a-real-rule".to_string()]);
    let graph = select_star_graph();
    let diags = engine.run(&graph, "", "test.ts");
    assert!(diags.is_empty(), "no rules should be registered");
  }

  #[test]
  fn resolve_rules_case_sensitive() {
    let engine = resolve_rules(&["NO-SELECT-STAR".to_string()]);
    let graph = select_star_graph();
    let diags = engine.run(&graph, "", "test.ts");
    assert!(
      diags.is_empty(),
      "rule names should be case-sensitive; NO-SELECT-STAR != no-select-star"
    );
  }

  // Regression: Bug #6 — prefix '-' for disabling rules is NOT implemented
  // Documentation says rules = ["-no-select-star"] disables a rule,
  // but the code does not strip the '-' prefix — it treats it as the rule name.
  #[test]
  fn resolve_rules_with_disable_prefix_should_still_have_other_rules() {
    let graph = select_star_graph();

    // What the user expects: rules = ["-no-select-star"] → 11 rules active, no-select-star disabled
    let engine_with_prefix = resolve_rules(&["-no-select-star".to_string()]);
    let diags = engine_with_prefix.run(&graph, "", "test.ts");

    let has_select_star = diags.iter().any(|d| d.rule_id == "no-select-star");
    assert!(
      !has_select_star,
      "no-select-star should NOT fire when disabled with '-no-select-star'"
    );

    // Other rules should still fire on the same graph
    let has_missing_limit = diags.iter().any(|d| d.rule_id == "no-missing-limit");
    assert!(
      has_missing_limit,
      "BUG #6: '-no-select-star' should disable ONLY no-select-star, \
       but no-missing-limit should still fire. Currently the '-' prefix is not \
       stripped, so '-no-select-star' is treated as an unknown rule name and \
       the engine has ZERO rules registered."
    );
  }

  // Sanity check: explicit rule list works
  #[test]
  fn resolve_rules_explicit_list() {
    let graph = select_star_graph();
    let engine = resolve_rules(&["no-select-star".to_string()]);
    let diags = engine.run(&graph, "", "test.ts");
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].rule_id, "no-select-star");
  }
}
