#![allow(
  clippy::cast_possible_truncation,
  clippy::cast_sign_loss,
  clippy::similar_names,
  clippy::too_many_lines,
)]

use pulsar_integration_tests::*;
use pulsar_test_utils::fixtures;

#[test]
fn basic_ts_detects_select_star() {
  let diags = analyze_ts(&fixtures::basic_ts());
  assert!(!diags.is_empty(), "basic.ts should produce diagnostics");
  let rule_ids: Vec<&str> = diags.iter().map(|d| d.rule_id.as_str()).collect();
  assert!(
    rule_ids.contains(&"no-select-star"),
    "expected no-select-star, got: {rule_ids:?}",
  );
}

#[test]
fn clean_ts_no_diagnostics() {
  let diags = analyze_ts(&fixtures::clean_ts());
  assert!(diags.is_empty(), "clean.ts should produce 0 diagnostics, got {diags:?}");
}

#[test]
fn no_issues_ts_no_diagnostics() {
  let diags = analyze_ts(&fixtures::no_issues_ts());
  assert!(diags.is_empty(), "no-issues.ts should produce 0 diagnostics, got {diags:?}");
}

#[test]
fn with_where_detects_select_star() {
  let diags = analyze_ts(&fixtures::with_where_ts());
  assert!(!diags.is_empty(), "with-where.ts should produce diagnostics");
  let rule_ids: Vec<&str> = diags.iter().map(|d| d.rule_id.as_str()).collect();
  assert!(rule_ids.contains(&"no-select-star"));
}

#[test]
fn with_limit_detects_select_star() {
  let diags = analyze_ts(&fixtures::with_limit_ts());
  assert!(!diags.is_empty(), "with-limit.ts should produce diagnostics");
  let rule_ids: Vec<&str> = diags.iter().map(|d| d.rule_id.as_str()).collect();
  assert!(rule_ids.contains(&"no-select-star"));
}

#[test]
fn invalid_syntax_returns_parse_error() {
  let result = pulsar_frontend_oxc::extract(
    &fixtures::invalid_syntax_ts(),
    oxc::span::SourceType::ts(),
    "invalid-syntax.ts",
  );
  assert!(result.is_err(), "invalid-syntax.ts should fail to parse");
}

// ─── Rule-specific fixtures ──────────────────────────────────

#[test]
fn no_missing_limit_detects_missing_limit() {
  let source = fixtures::read_rule_fixture("no-missing-limit", "query-without-limit.ts");
  let diags = analyze_ts(&source);
  let rule_ids: Vec<&str> = diags.iter().map(|d| d.rule_id.as_str()).collect();
  assert!(
    rule_ids.contains(&"no-missing-limit"),
    "expected no-missing-limit, got: {rule_ids:?}",
  );
}

#[test]
fn no_missing_limit_ignores_with_limit() {
  let source = fixtures::read_rule_fixture("no-missing-limit", "query-with-limit.ts");
  let diags = analyze_ts(&source);
  let rule_ids: Vec<&str> = diags.iter().map(|d| d.rule_id.as_str()).collect();
  assert!(
    !rule_ids.contains(&"no-missing-limit"),
    "should not fire no-missing-limit, got: {rule_ids:?}",
  );
}

#[test]
fn no_unbounded_find_detects_unbounded() {
  let source = fixtures::read_rule_fixture("no-unbounded-find", "unbounded-query.ts");
  let diags = analyze_ts(&source);
  let rule_ids: Vec<&str> = diags.iter().map(|d| d.rule_id.as_str()).collect();
  assert!(
    rule_ids.contains(&"no-unbounded-find"),
    "expected no-unbounded-find, got: {rule_ids:?}",
  );
}

#[test]
fn no_unbounded_find_ignores_bounded() {
  let source = fixtures::read_rule_fixture("no-unbounded-find", "bounded-query.ts");
  let diags = analyze_ts(&source);
  let rule_ids: Vec<&str> = diags.iter().map(|d| d.rule_id.as_str()).collect();
  assert!(
    !rule_ids.contains(&"no-unbounded-find"),
    "should not fire no-unbounded-find, got: {rule_ids:?}",
  );
}

#[test]
fn no_always_true_where_detects() {
  let source = fixtures::read_rule_fixture("no-always-true-where", "where-true.ts");
  let diags = analyze_ts(&source);
  let rule_ids: Vec<&str> = diags.iter().map(|d| d.rule_id.as_str()).collect();
  assert!(
    rule_ids.contains(&"no-always-true-where"),
    "expected no-always-true-where, got: {rule_ids:?}",
  );
}

#[test]
fn no_query_in_loop_detects() {
  let source = fixtures::read_rule_fixture("no-query-in-loop", "query-in-for-loop.ts");
  let diags = analyze_ts(&source);
  let rule_ids: Vec<&str> = diags.iter().map(|d| d.rule_id.as_str()).collect();
  assert!(
    rule_ids.contains(&"no-query-in-loop"),
    "expected no-query-in-loop, got: {rule_ids:?}",
  );
}

#[test]
fn no_query_in_callback_detects() {
  let source = fixtures::read_rule_fixture("no-query-in-callback", "query-in-then.ts");
  let diags = analyze_ts(&source);
  let rule_ids: Vec<&str> = diags.iter().map(|d| d.rule_id.as_str()).collect();
  assert!(
    rule_ids.contains(&"no-query-in-callback"),
    "expected no-query-in-callback, got: {rule_ids:?}",
  );
}

#[test]
fn no_raw_sql_dangerous_detects() {
  let source = fixtures::read_rule_fixture("no-raw-sql-dangerous", "sql-template.ts");
  let diags = analyze_ts(&source);
  let rule_ids: Vec<&str> = diags.iter().map(|d| d.rule_id.as_str()).collect();
  assert!(
    rule_ids.contains(&"no-raw-sql-dangerous"),
    "expected no-raw-sql-dangerous, got: {rule_ids:?}",
  );
}

#[test]
fn no_missing_await_detects() {
  let source = fixtures::read_rule_fixture("no-missing-await", "missing-await.ts");
  let diags = analyze_ts(&source);
  let rule_ids: Vec<&str> = diags.iter().map(|d| d.rule_id.as_str()).collect();
  assert!(
    rule_ids.contains(&"no-missing-await"),
    "expected no-missing-await, got: {rule_ids:?}",
  );
}
