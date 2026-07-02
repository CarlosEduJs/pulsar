# Pulsar

<p align="center">
  <strong>A static analyzer for TypeScript, ORMs, SQL, and database schemas.</strong>
  <br>
  <em>Detect quality, performance, and consistency issues before they reach production.</em>
</p>

Pulsar is a Rust-powered static analysis tool with a familiar lint-style interface. It
parses TypeScript sources, extracts ORM calls (Drizzle), resolves SQL queries, and runs
rules that flag problematic patterns — all in a single pipeline.

- **Language-agnostic IR**: Oxc (TypeScript), sqlparser-rs (SQL), and Prisma schema
  frontends all produce the same intermediate representation.
- **Unified dependency graph**: ORM, SQL, and schema nodes linked together for cross-layer
  analysis.
- **ESLint-style output** with source context underlines and colorized severity.
- **CI-ready**: exits with code 1 when errors are found; supports JSON output for SARIF,
  GitHub Actions, or custom tooling.

## Status

**v0.2 — Query safety rules.** The pipeline now includes 5 rules covering common SQL and
ORM pitfalls. Breaking changes are expected as the API stabilizes.

| Area               | Status |
|--------------------|--------|
| TypeScript parsing | ✅ Oxc frontend |
| SQL IR             | ✅ sqlparser-rs frontend |
| Drizzle ORM        | ✅ Method chain resolution + loop detection |
| Rule engine        | ✅ 5 built-in rules |
| CLI (pretty/JSON)  | ✅ `pulsar-cli check`/`init`/`explain` |
| Config system      | ✅ `pulsar.toml` auto-detect + `--config` |
| Prisma schema      | 🚧 Placeholder |
| LSP                | 🚧 Planned |

## Quick Start

```bash
# Requires Rust 1.80+
cargo build --release

# Analyze a single file
cargo run -p pulsar-cli -- check src/queries.ts

# Analyze a whole project (respects .gitignore)
cargo run -p pulsar-cli -- check .

# JSON output (for CI / tooling)
cargo run -p pulsar-cli -- check . --format json

# Generate a default config
cargo run -p pulsar-cli -- init

# Learn about a rule
cargo run -p pulsar-cli -- explain no-select-star

# Use a custom config file
cargo run -p pulsar-cli -- check . --config my-pulsar.toml
```

### Example Output

```
  src/users.ts:5:10  error    no-select-star     Avoid implicit SELECT *.
  src/users.ts:5:10  warning  no-missing-limit    Query is missing a LIMIT clause.
  src/users.ts:5:10  warning  no-unbounded-find   Query is unbounded — add a .where() or .limit().

    const users = await db.select().from(users)
                       ^^^^^^^^^^^^^^^^^^^^^^^^

✖ 3 problems (1 error, 2 warnings, 0 infos)
```

### Exit Codes

| Code | Meaning |
|------|---------|
| `0`  | No errors found |
| `1`  | One or more errors detected |

> Warnings and infos do not cause a non-zero exit.

## Rules

| Rule | Description | Severity |
|------|-------------|----------|
| `no-select-star` | Flags `SELECT *` queries (implicit or explicit). Always specify columns. | Error |
| `no-missing-limit` | Flags queries without a `LIMIT` clause that could return unbounded results. | Warning |
| `no-unbounded-find` | Flags ORM queries lacking both a `.where()` filter and a `.limit()` bound. | Warning |
| `no-always-true-where` | Flags `.where(true)` clauses that have no filtering effect. | Error |
| `no-query-in-loop` | Flags database queries executed inside loops (N+1 prevention). | Error |

## Architecture

```
Source Files (.ts, .sql, schema.prisma)
      │
  ┌───┼───┐
  ▼   ▼   ▼
Oxc  SQL  Prisma       ← Frontends
  │   │   │
  └───┼───┘
      ▼
 PIR Graph              ← Intermediate Representation (SQLNode, OrmNode, SchemaNode)
      │
      ▼
 Rule Engine            ← Rules inspect the graph
      │
      ▼
 Diagnostic API         ← Formatters (Pretty, JSON)
      │
      ▼
 CLI / LSP / SARIF      ← Output channels
```

### Crate Overview

| Crate | Role |
|-------|------|
| `pulsar-core` | Core types (`Diagnostic`, `Severity`, `SourceLocation`) |
| `pulsar-ir` | IR types (`SQLNode`, `OrmNode`, `SchemaNode`, `IrGraph`) |
| `pulsar-frontend-oxc` | TypeScript/TSX parser via oxc → ORM + SQL nodes |
| `pulsar-frontend-sql` | SQL parser via sqlparser-rs → SQL nodes |
| `pulsar-frontend-prisma` | (placeholder) Prisma schema → schema nodes |
| `pulsar-graph` | Graph construction and linking utilities |
| `pulsar-rules` | Rule trait, engine, and all lint rules |
| `pulsar-diag` | Diagnostic formatters (pretty, JSON) |
| `pulsar-cli` | Binary entry point — file walker, pipeline orchestration |

## Configuration

Pulsar looks for `pulsar.toml` in the project root (generated via `pulsar init`):

```toml
[settings]
ignore = ["node_modules", "dist", "build"]
rules = ["no-select-star", "no-missing-limit", "no-unbounded-find", "no-always-true-where", "no-query-in-loop"]
```

## Roadmap

- **v0.3**: Schema-aware analysis (postgres introspection, Prisma frontend)
- **v0.4**: LSP integration, SARIF output, GitHub Action

See [`docs/ROADMAP.md`](docs/ROADMAP.md) for details.

## Contributing

This is an early-stage project, and contributions are very welcome.

- Open an issue to discuss bugs or feature ideas.
- Submit PRs — make sure `cargo clippy --workspace` and `cargo test --workspace` pass.
- Read [`docs/wiki.md`](docs/wiki.md) for the full architecture blueprint.

## License

MIT — see [LICENSE](LICENSE).
