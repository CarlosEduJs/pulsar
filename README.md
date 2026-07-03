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

**v0.3 — Context-aware rules.** The pipeline now includes 9 rules covering SQL, ORM,
and cross-layer patterns. Breaking changes are expected as the API stabilizes.

| Area                    | Status |
|-------------------------|--------|
| TypeScript parsing      | ✅ Oxc frontend |
| SQL IR                  | ✅ sqlparser-rs frontend |
| Drizzle ORM             | ✅ Method chain resolution + loop/callback tracking |
| Raw SQL detection       | ✅ `sql\`…\`` tagged templates + `db.execute/all/get/run` |
| Rule engine             | ✅ 12 built-in rules |
| CLI (pretty/JSON)       | ✅ `pulsar-cli check`/`init`/`explain` |
| Config system           | ✅ `pulsar.toml` auto-detect + `--config` + `[database]` |
| Loop kind              | ✅ Counter vs Iteration distinction |
| Callback tracking       | ✅ `.then()`, `.map()`, `setTimeout`, etc. |
| Schema-aware rules      | ✅ Prisma frontend + 3 cross-layer rules |
| Prisma schema           | ✅ Parser for `.prisma` files |

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
| `no-query-in-loop` | Flags database queries executed inside counter loops (for, while). | Error |
| `no-query-in-callback` | Flags queries inside callbacks (`.then()`, `.map()`, `setTimeout`). | Warning |
| `no-n-plus-one` | Flags queries inside iteration loops (for-of, for-in). | Warning |
| `no-raw-sql-dangerous` | Flags raw SQL usage; Error if interpolated, Warning otherwise. | Error/Warning |
| `no-missing-await` | Flags ORM queries that lack the `await` keyword. | Error |
| `no-unindexed-filter` | Flags WHERE clauses on columns without a database index. | Warning |
| `no-unknown-column` | Flags references to columns that don't exist in the schema. | Error |
| `no-missing-foreign-key` | Flags included relations without a foreign key constraint. | Warning |

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
rules = ["no-select-star", "no-missing-limit", "no-unbounded-find", "no-always-true-where", "no-query-in-loop", "no-query-in-callback", "no-n-plus-one", "no-raw-sql-dangerous", "no-missing-await"]
```

## Roadmap

- **v0.4**: Schema-aware analysis (postgres introspection, Prisma frontend)
- **v0.5**: LSP integration, SARIF output, GitHub Action

See [`docs/ROADMAP.md`](docs/ROADMAP.md) for details.

## Contributing

This is an early-stage project, and contributions are very welcome.

- Open an issue to discuss bugs or feature ideas.
- Submit PRs — make sure `cargo clippy --workspace` and `cargo test --workspace` pass.
- Read [`docs/wiki.md`](docs/wiki.md) for the full architecture blueprint.

## License

MIT — see [LICENSE](LICENSE).
