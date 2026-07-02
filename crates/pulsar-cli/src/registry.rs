use std::collections::HashMap;

use pulsar_rules::rules::NoSelectStar;
use pulsar_rules::{Rule, RuleEngine};

type RuleConstructor = fn() -> Box<dyn Rule>;

/// Returns all built-in rules keyed by their `id()`.
#[must_use]
pub fn builtin_rules() -> HashMap<&'static str, RuleConstructor> {
  let mut map: HashMap<&'static str, RuleConstructor> = HashMap::new();
  map.insert("no-select-star", || Box::new(NoSelectStar));
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
