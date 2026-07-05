use pulsar_core::{Diagnostic, Severity};
use pulsar_ir::NodeKind;

use crate::rule::{Rule, RuleContext};
use crate::util::extract_where_column_names;

pub struct NoUnknownColumn;

impl Rule for NoUnknownColumn {
  fn id(&self) -> &'static str {
    "no-unknown-column"
  }

  fn docs(&self) -> &'static str {
    "Flags references to columns that do not exist in the database schema.\n\
    \n\
    When a schema file is provided, this rule checks that every column \
    referenced in `.select()` and `.where()` exists in the target table. \
    Typos and renamed columns are caught before runtime."
  }

  fn run(&self, ctx: &RuleContext) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    for id in ctx.graph.node_indices() {
      let NodeKind::Orm(orm) = ctx.graph.node(id).expect("node should exist") else { continue };

      let Some(schema) = ctx.graph.schema_for_orm(id) else { continue };

      // Check selected columns
      for col_name in &orm.args.columns {
        if !schema.columns.iter().any(|c| c.name == *col_name) {
          diags.push(Diagnostic {
            severity: Severity::Error,
            message: format!(
              "Column `{col_name}` selected in query but does not exist in schema table `{}`.",
              schema.table_name,
            ),
            location: orm.location.clone(),
            rule_id: self.id().to_string(),
          });
        }
      }

      // Check where clause columns
      if let Some(ref where_clause) = orm.args.where_clause {
        for col_name in extract_where_column_names(where_clause) {
          if !schema.columns.iter().any(|c| c.name == col_name) {
            diags.push(Diagnostic {
              severity: Severity::Error,
              message: format!(
                "Column `{col_name}` used in WHERE clause but does not exist in schema table `{}`.",
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

  fn make_graph(select_wrong: bool, where_wrong: bool) -> IrGraph {
    let mut graph = IrGraph::new();
    let loc = SourceLocation { file: "test.ts".to_string(), line: 1, column: 1, span: None };

    let select_cols =
      if select_wrong { vec!["nonexistent".to_string()] } else { vec!["id".to_string()] };
    let where_clause = if where_wrong {
      Some("eq(users.bad_col, 1)".to_string())
    } else {
      Some("eq(users.id, 1)".to_string())
    };

    let orm = OrmNode {
      method: OrmMethod::Select,
      args: OrmArgs { columns: select_cols, where_clause, limit: None, include: Vec::new() },
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
      location: loc,
    };

    let schema = SchemaNode {
      table_name: "users".to_string(),
      columns: vec![SchemaColumn {
        name: "id".to_string(),
        col_type: "Int".to_string(),
        is_nullable: false,
        is_indexed: true,
        col_default: None,
        is_unique: true,
        foreign_key: None,
      }],
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
  fn flags_wrong_select_column() {
    let graph = make_graph(true, false);
    let rule = NoUnknownColumn;
    let ctx =
      RuleContext { graph: &graph, source_text: "", file_path: "test.ts", active_rules: &[] };
    let diags = rule.run(&ctx);
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].rule_id, "no-unknown-column");
    assert_eq!(diags[0].severity, Severity::Error);
    assert!(diags[0].message.contains("nonexistent"));
  }

  #[test]
  fn flags_wrong_where_column() {
    let graph = make_graph(false, true);
    let rule = NoUnknownColumn;
    let ctx =
      RuleContext { graph: &graph, source_text: "", file_path: "test.ts", active_rules: &[] };
    let diags = rule.run(&ctx);
    assert_eq!(diags.len(), 1);
    assert!(diags[0].message.contains("bad_col"));
  }

  #[test]
  fn allows_correct_columns() {
    let graph = make_graph(false, false);
    let rule = NoUnknownColumn;
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
    let rule = NoUnknownColumn;
    let ctx =
      RuleContext { graph: &graph, source_text: "", file_path: "test.ts", active_rules: &[] };
    let diags = rule.run(&ctx);
    assert!(diags.is_empty(), "no schema = no diagnostics");
  }

  #[test]
  fn empty_graph_no_diagnostics() {
    let graph = IrGraph::new();
    let rule = NoUnknownColumn;
    let ctx =
      RuleContext { graph: &graph, source_text: "", file_path: "test.ts", active_rules: &[] };
    let diags = rule.run(&ctx);
    assert!(diags.is_empty());
  }
}
