/// Build a rule engine with all available rules registered.
///
/// This is the single source of truth for the full rule set.
/// Keep it aligned with the rules registered in `pulsar_rules::RuleEngine::new()`.
pub fn all_rules_engine() -> pulsar_rules::RuleEngine {
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
