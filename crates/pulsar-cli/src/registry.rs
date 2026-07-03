use std::collections::BTreeMap;

use pulsar_rules::rules::{
  NoAlwaysTrueWhere, NoMissingAwait, NoMissingLimit, NoNPlusOne, NoQueryInCallback, NoQueryInLoop,
  NoRawSqlDangerous, NoSelectStar, NoUnboundedFind,
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
