use pulsar_core::{Diagnostic, Severity};
use pulsar_ir::NodeKind;

use crate::rule::{Rule, RuleContext};
use crate::util::extract_where_column_names;

pub struct NoUnindexedFilter;

impl Rule for NoUnindexedFilter {
  fn id(&self) -> &'static str {
    "no-unindexed-filter"
  }

  fn docs(&self) -> &'static str {
    "Flags queries that filter on columns without an index.\n\
    \n\
    Filtering on unindexed columns leads to full table scans and poor \
    performance. Ensure indexed columns are used in WHERE clauses, or \
    add a database index for the filtered column."
  }

  fn run(&self, ctx: &RuleContext) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    for id in ctx.graph.node_indices() {
      let NodeKind::Orm(orm) = ctx.graph.node(id).expect("node should exist") else { continue };

      let Some(ref where_clause) = orm.args.where_clause else { continue };
      let filtered_cols = extract_where_column_names(where_clause);
      if filtered_cols.is_empty() {
        continue;
      }

      let Some(schema) = ctx.graph.schema_for_orm(id) else { continue };

      for col_name in &filtered_cols {
        let col = schema.columns.iter().find(|c| c.name == *col_name);
        match col {
          Some(c) if c.is_indexed => continue,
          Some(c) => {
            diags.push(Diagnostic {
              severity: Severity::Warning,
              message: format!(
                "Filtering on `{}` which is not indexed — add an index for better performance.",
                c.name,
              ),
              location: orm.location.clone(),
              rule_id: self.id().to_string(),
            });
          }
          None => {
            diags.push(Diagnostic {
              severity: Severity::Warning,
              message: format!(
                "Filtering on `{col_name}` which does not exist in schema table `{}`.",
                schema.table_name,
              ),
              location: orm.location.clone(),
              rule_id: self.id().to_string(),
            });
          }
        }
      }
    }
    diags
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use pulsar_core::SourceLocation;
  use pulsar_ir::{
    ColumnRef, EdgeKind, IrGraph, LoopKind, OrmArgs, OrmMethod, OrmNode, SQLNode, SchemaColumn,
    SchemaNode, SqlKind, TableRef,
  };

  fn make_graph(indexed_col: Option<&str>) -> IrGraph {
    let mut graph = IrGraph::new();
    let loc = SourceLocation { file: "test.ts".to_string(), line: 1, column: 1, span: None };

    let orm = OrmNode {
      method: OrmMethod::Select,
      args: OrmArgs {
        columns: vec!["id".to_string()],
        where_clause: Some("eq(users.email, \"test\")".to_string()),
        limit: None,
        include: Vec::new(),
      },
      loop_kind: LoopKind::None,
      in_callback: false,
      missing_await: false,
      location: loc.clone(),
    };

    let sql = SQLNode {
      kind: SqlKind::Select,
      columns: vec![ColumnRef { name: "id".to_string(), table: None }],
      table: Some(TableRef { name: "users".to_string(), alias: None }),
      limit: false,
      where_clause: true,
      in_callback: false,
      location: loc.clone(),
    };

    let schema = SchemaNode {
      table_name: "users".to_string(),
      columns: vec![
        SchemaColumn {
          name: "id".to_string(),
          col_type: "Int".to_string(),
          is_nullable: false,
          is_indexed: true,
          col_default: Some("autoincrement()".to_string()),
          is_unique: true,
          foreign_key: None,
        },
        SchemaColumn {
          name: "email".to_string(),
          col_type: "String".to_string(),
          is_nullable: false,
          is_indexed: indexed_col.is_some(),
          col_default: None,
          is_unique: false,
          foreign_key: None,
        },
      ],
      indexes: vec![],
    };

    let orm_id = graph.add_orm(orm);
    let sql_id = graph.add_sql(sql);
    let schema_id = graph.add_schema(schema);
    graph.add_edge(orm_id, sql_id, EdgeKind::Generates);
    graph.add_edge(sql_id, schema_id, EdgeKind::Accesses);
    graph
  }

  #[test]
  fn flags_unindexed_column() {
    let graph = make_graph(None);
    let rule = NoUnindexedFilter;
    let ctx =
      RuleContext { graph: &graph, source_text: "", file_path: "test.ts", active_rules: &[] };
    let diags = rule.run(&ctx);
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].rule_id, "no-unindexed-filter");
    assert_eq!(diags[0].severity, Severity::Warning);
    assert!(diags[0].message.contains("email"));
  }

  #[test]
  fn allows_indexed_column() {
    let graph = make_graph(Some("email"));
    let rule = NoUnindexedFilter;
    let ctx =
      RuleContext { graph: &graph, source_text: "", file_path: "test.ts", active_rules: &[] };
    let diags = rule.run(&ctx);
    assert!(diags.is_empty());
  }

  #[test]
  fn no_where_clause() {
    let mut graph = IrGraph::new();
    let loc = SourceLocation { file: "test.ts".to_string(), line: 1, column: 1, span: None };
    let orm = OrmNode {
      method: OrmMethod::Select,
      args: OrmArgs {
        columns: vec!["id".to_string()],
        where_clause: None,
        limit: None,
        include: Vec::new(),
      },
      loop_kind: LoopKind::None,
      in_callback: false,
      missing_await: false,
      location: loc,
    };
    graph.add_orm(orm);
    let rule = NoUnindexedFilter;
    let ctx =
      RuleContext { graph: &graph, source_text: "", file_path: "test.ts", active_rules: &[] };
    let diags = rule.run(&ctx);
    assert!(diags.is_empty());
  }

  #[test]
  fn no_schema_linked() {
    let mut graph = IrGraph::new();
    let loc = SourceLocation { file: "test.ts".to_string(), line: 1, column: 1, span: None };
    let orm = OrmNode {
      method: OrmMethod::Select,
      args: OrmArgs {
        columns: vec!["id".to_string()],
        where_clause: Some("eq(users.email, \"test\")".to_string()),
        limit: None,
        include: Vec::new(),
      },
      loop_kind: LoopKind::None,
      in_callback: false,
      missing_await: false,
      location: loc,
    };
    graph.add_orm(orm);
    let rule = NoUnindexedFilter;
    let ctx =
      RuleContext { graph: &graph, source_text: "", file_path: "test.ts", active_rules: &[] };
    let diags = rule.run(&ctx);
    assert!(diags.is_empty(), "no schema = no diagnostics");
  }

  #[test]
  fn empty_graph_no_diagnostics() {
    let graph = IrGraph::new();
    let rule = NoUnindexedFilter;
    let ctx =
      RuleContext { graph: &graph, source_text: "", file_path: "test.ts", active_rules: &[] };
    let diags = rule.run(&ctx);
    assert!(diags.is_empty());
  }
}
