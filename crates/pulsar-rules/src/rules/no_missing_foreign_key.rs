use pulsar_core::{Diagnostic, Severity};
use pulsar_ir::NodeKind;

use crate::rule::{Rule, RuleContext};

pub struct NoMissingForeignKey;

impl Rule for NoMissingForeignKey {
  fn id(&self) -> &'static str {
    "no-missing-foreign-key"
  }

  fn docs(&self) -> &'static str {
    "Flags included relations that lack a foreign key in the database schema.\n\
    \n\
    When using `.include()` or `.with()` to load related data, the relationship \
    should be backed by a foreign key constraint. Without it, referential \
    integrity is not enforced at the database level."
  }

  fn run(&self, ctx: &RuleContext) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    for id in ctx.graph.node_indices() {
      let NodeKind::Orm(orm) = ctx.graph.node(id).expect("node should exist") else { continue };

      if orm.args.include.is_empty() {
        continue;
      }

      let Some(schema) = ctx.graph.schema_for_orm(id) else { continue };

      for included in &orm.args.include {
        // Check if the schema has at least one FK reference
        let has_fk = schema.columns.iter().any(|c| c.foreign_key.is_some());

        if !has_fk {
          diags.push(Diagnostic {
            severity: Severity::Warning,
            message: format!(
              "Relation `{included}` included in query but table `{}` has no foreign key — consider adding one.",
              schema.table_name,
            ),
            location: orm.location.clone(),
            rule_id: self.id().to_string(),
          });
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
    ColumnRef, EdgeKind, ForeignKeyRef, IrGraph, LoopKind, OrmArgs, OrmMethod, OrmNode,
    SchemaColumn, SchemaNode, SQLNode, SqlKind, TableRef,
  };

  fn make_graph(has_fk: bool) -> IrGraph {
    let mut graph = IrGraph::new();
    let loc = SourceLocation { file: "test.ts".to_string(), line: 1, column: 1, span: None };

    let orm = OrmNode {
      method: OrmMethod::Select,
      args: OrmArgs {
        columns: vec!["id".to_string()],
        where_clause: None,
        limit: None,
        include: vec!["author".to_string()],
      },
      loop_kind: LoopKind::None,
      in_callback: false,
      missing_await: false,
      location: loc.clone(),
    };

    let sql = SQLNode {
      kind: SqlKind::Select,
      columns: vec![ColumnRef { name: "id".to_string(), table: None }],
      table: Some(TableRef { name: "posts".to_string(), alias: None }),
      limit: false,
      where_clause: false,
      in_callback: false,
      location: loc.clone(),
    };

    let schema = SchemaNode {
      table_name: "posts".to_string(),
      columns: vec![
        SchemaColumn {
          name: "id".to_string(),
          col_type: "Int".to_string(),
          is_nullable: false,
          is_indexed: true,
          col_default: None,
          is_unique: true,
          foreign_key: None,
        },
        SchemaColumn {
          name: "author_id".to_string(),
          col_type: "Int".to_string(),
          is_nullable: true,
          is_indexed: false,
          col_default: None,
          is_unique: false,
          foreign_key: if has_fk {
            Some(ForeignKeyRef { ref_table: "User".to_string(), ref_column: "id".to_string() })
          } else {
            None
          },
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
  fn flags_missing_fk() {
    let graph = make_graph(false);
    let rule = NoMissingForeignKey;
    let ctx =
      RuleContext { graph: &graph, source_text: "", file_path: "test.ts", active_rules: &[] };
    let diags = rule.run(&ctx);
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].rule_id, "no-missing-foreign-key");
    assert_eq!(diags[0].severity, Severity::Warning);
    assert!(diags[0].message.contains("author"));
  }

  #[test]
  fn allows_with_fk() {
    let graph = make_graph(true);
    let rule = NoMissingForeignKey;
    let ctx =
      RuleContext { graph: &graph, source_text: "", file_path: "test.ts", active_rules: &[] };
    let diags = rule.run(&ctx);
    assert!(diags.is_empty());
  }

  #[test]
  fn no_include_no_diag() {
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
    let rule = NoMissingForeignKey;
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
        include: vec!["author".to_string()],
      },
      loop_kind: LoopKind::None,
      in_callback: false,
      missing_await: false,
      location: loc,
    };
    graph.add_orm(orm);
    let rule = NoMissingForeignKey;
    let ctx =
      RuleContext { graph: &graph, source_text: "", file_path: "test.ts", active_rules: &[] };
    let diags = rule.run(&ctx);
    assert!(diags.is_empty(), "no schema = no diagnostics");
  }

  #[test]
  fn empty_graph_no_diagnostics() {
    let graph = IrGraph::new();
    let rule = NoMissingForeignKey;
    let ctx =
      RuleContext { graph: &graph, source_text: "", file_path: "test.ts", active_rules: &[] };
    let diags = rule.run(&ctx);
    assert!(diags.is_empty());
  }
}
