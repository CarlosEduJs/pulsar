# Pulsar

> Pulsar is a static analyzer with a linter interface.
>
> It performs deep analysis across TypeScript, ORMs, SQL and database schemas while presenting diagnostics in a familiar lint-style format.

## Goal

- The main objective is to detect quality, performance, and consistency issues between TypeScript, ORMs, SQL, and the schema before the application is executed.

- Pulsar is designed to be used in CI/CD pipelines, pre-commit hooks, and as a part of the development workflow to ensure code quality and consistency.

## Pipeline

```
                Source Files
     (.ts, .sql, schema.prisma, ...)
                      │
      ┌───────────────┼────────────────┐
      ▼               ▼                ▼
   Oxc Frontend   SQL Frontend   Prisma Frontend
      │               │                │
      └───────────────┼────────────────┘
                      ▼
             Pulsar Intermediate Representation (PIR)
                      │
                      ▼
            Graph Construction & Linking
                      │
          ┌───────────┼────────────┐
          ▼           ▼            ▼
      ORM Graph   SQL Graph   Schema Graph
          └───────────┼────────────┘
                      ▼
             Unified Dependency Graph
                      │
                      ▼
                Rule Engine
                      │
                      ▼
               Diagnostics API
                      │
                      ▼
        CLI / LSP / SARIF / GitHub Actions
```

## Project Structure

```
pulsar/
├── Cargo.toml
├── crates/
│   ├── pulsar-core/           # Core traits, types, PIR definitions
│   ├── pulsar-frontend-oxc/   # Oxc parser → TS/JS AST → PIR
│   ├── pulsar-frontend-sql/   # sqlparser-rs → SQL AST → PIR
│   ├── pulsar-frontend-prisma/# Prisma schema parser → PIR
│   ├── pulsar-ir/             # PIR types, graph structures
│   ├── pulsar-graph/          # Graph construction & linking
│   ├── pulsar-rules/          # Rule engine + all rules
│   ├── pulsar-diag/           # Diagnostics API + formatting
│   └── pulsar-cli/            # CLI binary
```

## Intermediate Representation (PIR)

PIR is the bridge between frontends and analysis. It is a hybrid representation composed of three specialized graph types:

### ORM Graph

Nodes representing ORM operations extracted from TypeScript (e.g., Drizzle queries). Each node captures:

- Method call (`findMany`, `findFirst`, `insert`, `update`, `delete`, `select`, etc.)
- Arguments (where clauses, orderBy, limit, offset, include, etc.)
- Source location
- Connection to the underlying SQL it generates

### SQL Graph

Nodes representing SQL queries parsed from raw SQL strings in the codebase or reconstructed from ORM calls. Each node captures:

- Query kind (SELECT, INSERT, UPDATE, DELETE)
- Columns referenced
- Tables referenced
- JOINs, WHERE conditions, LIMIT/OFFSET
- Parameterization info

### Schema Graph

Nodes representing database schema objects reconstructed from Prisma schemas, DDL files, or introspection. Each node captures:

- Tables, columns, types, nullability, defaults
- Indexes (columns, unique, partial)
- Foreign keys, relations
- Constraints

### Unified Dependency Graph

The three graphs are linked into a unified graph where edges represent relationships like:

```
ORM call ──► SQL query it generates
SQL query ──► Tables/columns it accesses
ORM call ──► Schema entities it maps to
```

This enables cross-layer analysis without losing context.

## Rules

### SQL Layer

| Rule | Category | Bad | Good | Detection |
|---|---|---|---|---|
| `no-select-star` | Performance | `SELECT * FROM users` | `SELECT id, name FROM users` | Checks for implicit column expansion |
| `no-missing-limit` | Performance | `SELECT * FROM orders` | `SELECT * FROM orders LIMIT 100` | Checks absence of LIMIT clause |
| `no-always-true-where` | Correctness | `WHERE 1=1` | `WHERE status = 'active'` | Constant expression evaluation |
| `no-string-interpolation` | Security | `WHERE name = '${input}'` | `WHERE name = $1` | Detects string concatenation in SQL |

### ORM Layer

| Rule | Category | Bad | Good | Detection |
|---|---|---|---|---|
| `no-n+1` | Performance | Loop with per-iteration `findUnique` | `findMany` with `include` | Detects repeated queries that could be batched |
| `no-missing-await` | Correctness | `db.select().from(users)` without await | `await db.select().from(users)` | Checks if promise is not awaited |
| `no-query-in-loop` | Performance | `for (u of users) { await db.find(...) }` | `await db.findMany(...)` | Detects queries inside loops |
| `no-unbounded-find` | Performance | `db.findMany()` without limit/take | `db.findMany({ take: 50 })` | Detects unbounded collection queries |
| `no-implicit-join` | Performance | Implicit cross-entity access in loop | Explicit `include`/`join` | Detects lazy relationship loading |
| `no-raw-input-in-where` | Security | `where(name, op.eq(rawInput))` | `where(name, op.eq(safe(input)))` | Taint tracking from user input |

### Cross Layer

| Rule | Category | Bad | Good | Detection |
|---|---|---|---|---|
| `no-unindexed-filter` | Performance | `WHERE email = ...` with no index | `WHERE id = ...` (indexed PK) | Cross-references filtered columns with schema indexes |
| `no-implicit-select-star` | Correctness | `db.select().from(users)` | `db.select({ id, name }).from(users)` | Detects select without explicit projection |

## Configuration

`pulsar.toml`:

```toml
[rules]
no-select-star = "error"
no-missing-limit = "warn"
no-n-plus-one = "error"
no-unindexed-filter = "warn"

[database]
schema = "./schema.prisma"
dialect = "postgresql"

include = ["src/**/*.ts"]
exclude = ["node_modules", "dist"]
```

## CLI Usage

```bash
pulsar check                       # Run analysis on current directory
pulsar check ./src                 # Scan specific directory
pulsar check --config custom.toml  # Custom config
pulsar check --format json         # JSON output (for CI)
pulsar check --format sarif        # SARIF output (GitHub Code Scanning)
pulsar --init                      # Generate default pulsar.toml
pulsar explain no-select-star      # Explain a specific rule
```

## Diagnostics API

The Diagnostics API is the single output interface of the rule engine. It enables multiple consumers:

- **CLI** — human-readable colored output (ESLint-style)
- **LSP** — in-editor diagnostics
- **SARIF** — GitHub Code Scanning, CI integrations
- **GitHub Actions** — annotations via `warning`/`error` commands
- **JSON** — custom tooling and dashboards

## Development

- Built with Rust for performance and correctness
- Uses `oxc` for TypeScript/JavaScript parsing (blazing fast, Rust-native)
- Uses `sqlparser-rs` for SQL parsing (PostgreSQL dialect)
- Snapshot-based testing with `insta`
- Designed for incremental adoption — run on specific files or whole projects

## Manifest

> Modern programming languages come with excellent compilers and linters, yet the persistence layer has largely been left behind — still depending on tests, benchmarks, and painful production surprises to surface problems. Pulsar was built to close that gap, treating TypeScript, ORMs, SQL, and schema not as separate concerns, but as parts of one unified program.
