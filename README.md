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

**v0.1 — Proof of concept.** The pipeline is wired end-to-end with one rule. Breaking
changes are expected as the API stabilizes.

| Area               | Status |
|--------------------|--------|
| TypeScript parsing | ✅ Oxc frontend |
| SQL IR             | ✅ sqlparser-rs frontend |
| Drizzle ORM        | ✅ Method chain resolution |
| Rule engine        | ✅ `no-select-star` rule |
| CLI (pretty/JSON)  | ✅ `pulsar check` |
| Prisma schema      | 🚧 Placeholder |
| LSP                | 🚧 Planned |
| More rules         | 🚧 In progress |

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
```

### Example Output

```
  src/users.ts:5:10  error  no-select-star  Avoid implicit SELECT *.

    const users = await db.select().from(users)
                       ^^^^^^^^^^^^^^^^^^^^^^^^

✖ 1 problem (1 error, 0 warnings, 0 infos)
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
rules = ["no-select-star"]
```

## Roadmap

- **v0.2**: More rules (query performance, N+1 detection, missing indexes)
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
