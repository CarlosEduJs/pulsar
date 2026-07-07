#![allow(clippy::multiple_crate_versions)]

use std::path::Path;
use std::process;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use ignore::WalkBuilder;
use oxc::span::SourceType;
use pulsar_core::Severity;
use pulsar_diag::{DiagnosticFormatter, JsonFormatter, PrettyFormatter};
use pulsar_frontend_oxc::extract;
use pulsar_frontend_prisma::PrismaSchemaProvider;
use pulsar_ir::SchemaProvider;
mod config;
mod registry;

#[derive(Parser)]
#[command(name = "pulsar", about = "Static analyzer for TypeScript + ORM + SQL", version)]
struct Cli {
  #[command(subcommand)]
  command: Command,
}

#[derive(Subcommand)]
enum Command {
  /// Analyze source files for issues
  Check(CheckArgs),
  /// Generate a default pulsar.toml config file
  Init,
  /// Explain a specific lint rule
  Explain { rule: String },
}

#[derive(clap::Args)]
struct CheckArgs {
  /// Path to a file or directory to analyze
  path: Option<String>,
  /// Output format: pretty or json
  #[arg(short, long, default_value = "pretty")]
  format: String,
  /// Path to pulsar.toml config file
  #[arg(long)]
  config: Option<String>,
}

fn main() -> Result<()> {
  let cli = Cli::parse();
  match cli.command {
    Command::Check(args) => run_check(args),
    Command::Init => run_init(),
    Command::Explain { rule } => {
      run_explain(&rule);
      Ok(())
    }
  }
}

fn run_check(args: CheckArgs) -> Result<()> {
  let path = args.path.unwrap_or_else(|| ".".to_string());
  let format = args.format;
  let config_path = args.config.as_ref().map(Path::new);

  let config = config::PulsarConfig::load(config_path).with_context(|| "failed to load config")?;
  let engine = registry::resolve_rules(&config.settings.rules);

  // Load schema if configured
  let schema_tables = if let Some(schema_path) = &config.database.schema {
    match PrismaSchemaProvider::from_file(schema_path) {
      Ok(provider) => match provider.load() {
        Ok(tables) => {
          eprintln!("Loaded schema from {} ({} tables)", schema_path, tables.len());
          Some(tables)
        }
        Err(e) => {
          return Err(anyhow::anyhow!("Failed to parse schema {schema_path}: {e}"));
        }
      },
      Err(e) => {
        return Err(anyhow::anyhow!("Failed to read schema {schema_path}: {e}"));
      }
    }
  } else {
    None
  };

  let mut walker_builder = WalkBuilder::new(&path);
  walker_builder.standard_filters(true);

  if !config.settings.ignore.is_empty() {
    let ignore_list = config.settings.ignore;
    walker_builder.filter_entry(move |entry| {
      let Some(file_name) = entry.file_name().to_str() else { return true };
      !ignore_list.iter().any(|pat| file_name == pat)
    });
  }

  let walker = walker_builder.build();

  let mut file_diagnostics: Vec<(String, String, Vec<pulsar_core::Diagnostic>)> = Vec::new();
  let mut parse_failures: usize = 0;

  for result in walker {
    let entry = result?;

    if !entry.file_type().is_some_and(|ft| ft.is_file()) {
      continue;
    }

    let entry_path = entry.path();
    if !entry_path.extension().and_then(|e| e.to_str()).is_some_and(|e| e == "ts" || e == "tsx") {
      continue;
    }

    let source = std::fs::read_to_string(entry_path)
      .with_context(|| format!("failed to read {}", entry_path.display()))?;

    let Ok(source_type) = SourceType::from_path(entry_path) else {
      continue;
    };

    let file_path_str = entry_path.to_string_lossy().to_string();

    let mut graph = match extract(&source, source_type, &file_path_str) {
      Ok(graph) => graph,
      Err(e) => {
        eprintln!("Error parsing {}: {e}", entry_path.display());
        parse_failures += 1;
        continue;
      }
    };

    // Load schema into this file's graph if available
    if let Some(ref tables) = schema_tables {
      graph.load_schema(tables);
    }

    let diagnostics = engine.run(&graph, &source, &file_path_str);
    if !diagnostics.is_empty() {
      file_diagnostics.push((file_path_str, source, diagnostics));
    }
  }

  let formatter: Box<dyn DiagnosticFormatter> = match format.as_str() {
    "json" => Box::new(JsonFormatter),
    _ => Box::new(PrettyFormatter),
  };

  let errors: usize = file_diagnostics
    .iter()
    .flat_map(|(_, _, diags)| diags.iter())
    .filter(|d| d.severity == Severity::Error)
    .count();

  if format.as_str() == "json" {
    let all: Vec<pulsar_core::Diagnostic> =
      file_diagnostics.iter().flat_map(|(_, _, diags)| diags.iter()).cloned().collect();
    println!("{}", formatter.format(&all, ""));
  } else {
    if file_diagnostics.is_empty() {
      return Ok(());
    }
    for (_file_path, source, diags) in &file_diagnostics {
      print!("{}", formatter.format(diags, source));
    }
  }

  if errors > 0 || parse_failures > 0 {
    process::exit(1);
  }

  Ok(())
}

fn run_init() -> Result<()> {
  let config = "\
[settings]
# Directories/files to ignore (in addition to .gitignore)
ignore = [\"node_modules\", \"dist\", \"build\"]

# Enabled rules (empty = all built-in rules)
rules = [\"no-select-star\", \"no-missing-limit\", \"no-unbounded-find\", \"no-always-true-where\", \"no-query-in-loop\", \"no-query-in-callback\", \"no-n-plus-one\", \"no-raw-sql-dangerous\", \"no-missing-await\", \"no-unindexed-filter\", \"no-unknown-column\", \"no-missing-foreign-key\"]

[database]
# Path to Prisma schema file (enables schema-aware rules)
# schema = \"./prisma/schema.prisma\"
";
  std::fs::write("pulsar.toml", config).context("failed to write pulsar.toml")?;
  eprintln!("Created pulsar.toml");
  Ok(())
}

fn run_explain(rule: &str) {
  let builtins = registry::builtin_rules();
  if let Some(ctor) = builtins.get(rule) {
    let r = ctor();
    println!("{}\n\n{}", r.id(), r.docs());
  } else {
    eprintln!("Unknown rule: {rule}");
    process::exit(1);
  }
}
