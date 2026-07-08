#![allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, clippy::similar_names)]

use pulsar_integration_tests::*;
use pulsar_test_utils::fixtures;

/// Shared prisma schema for schema-aware tests.
fn schema_source() -> String {
  fixtures::schema_prisma()
}

#[test]
fn no_unindexed_filter_detects_unindexed_column() {
  let source = fixtures::read_rule_fixture("no-unindexed-filter", "filter-on-name.ts");
  let diags = analyze_ts_with_schema(&source, &schema_source());
  let rule_ids: Vec<&str> = diags.iter().map(|d| d.rule_id.as_str()).collect();
  assert!(
    rule_ids.contains(&"no-unindexed-filter"),
    "expected no-unindexed-filter for filter-on-name, got: {rule_ids:?}",
  );
}

#[test]
fn no_unindexed_filter_ignores_indexed_column() {
  let source = fixtures::read_rule_fixture("no-unindexed-filter", "filter-on-indexed.ts");
  let diags = analyze_ts_with_schema(&source, &schema_source());
  let rule_ids: Vec<&str> = diags.iter().map(|d| d.rule_id.as_str()).collect();
  assert!(
    !rule_ids.contains(&"no-unindexed-filter"),
    "should not fire no-unindexed-filter for filter-on-indexed, got: {rule_ids:?}",
  );
}

#[test]
fn no_unindexed_filter_ignores_indexed_via_block() {
  let source =
    fixtures::read_rule_fixture("no-unindexed-filter", "filter-on-indexed-via-block.ts");
  let diags = analyze_ts_with_schema(&source, &schema_source());
  let rule_ids: Vec<&str> = diags.iter().map(|d| d.rule_id.as_str()).collect();
  assert!(
    !rule_ids.contains(&"no-unindexed-filter"),
    "should not fire no-unindexed-filter for filter-on-indexed-via-block, got: {rule_ids:?}",
  );
}

#[test]
fn no_unknown_column_detects_wrong_column() {
  let source = fixtures::read_rule_fixture("no-unknown-column", "select-wrong.ts");
  let diags = analyze_ts_with_schema(&source, &schema_source());
  let rule_ids: Vec<&str> = diags.iter().map(|d| d.rule_id.as_str()).collect();
  assert!(
    rule_ids.contains(&"no-unknown-column"),
    "expected no-unknown-column for select-wrong, got: {rule_ids:?}",
  );
}

#[test]
fn clean_ts_with_schema_no_diagnostics() {
  let diags = analyze_ts_with_schema(&fixtures::clean_ts(), &schema_source());
  assert!(diags.is_empty(), "clean.ts with schema should produce 0 diagnostics");
}
