# Roadmap v0.1 — Vertical Slice MVP

> **Goal**: Validate the full pipeline (TS → PIR → Graph → Rule → Diagnostic) with **one meaningful rule** end-to-end. The goal of v0.1 is not to prove that the no-select-star rule works. The goal is to prove that new rules can be added without changing the architecture.

## The Validation Test

```typescript
// test/fixtures/basic.ts
import { db } from './db';

// should warn: SELECT * is implicit
const users = await db.select().from(users);

// should be fine: explicit columns
const admins = await db.select({ id: users.id, name: users.name }).from(users);
```

Expected output:

```
  test/fixtures/basic.ts:4:30  error  no-select-star  Avoid implicit SELECT *. Specify columns explicitly.
```

## Steps

### 1. `pulsar-core` — Core Types
- `Severity` enum (`Error`, `Warning`, `Info`)
- `Diagnostic` struct (severity, message, location, rule_id)
- `Rule` trait (fn `id()`, fn `run()`)
- `SourceLocation` (file, line, column, span)

### 2. `pulsar-ir` — PIR Types
- `SQLNode` — SELECT only: columns, table, limit, where clause
- `OrmNode` — Drizzle select call: method, args, resolved query
- `SchemaNode` — minimal: table name, columns
- `NodeId`, `Edge` types for graph
- `IrGraph` — petgraph wrapper

### 3. `pulsar-frontend-sql` — SQL Parser
- Parse SQL `SELECT` strings via `sqlparser-rs`
- Map parsed AST → `SQLNode` IR
- Error handling for invalid SQL

### 4. `pulsar-frontend-oxc` — TS Frontend
- Scan `.ts` files with oxc
- Detect Drizzle `db.select()` call patterns
- Extract or reconstruct SQL from the ORM calls
- Emit `OrmNode`s linked to `SQLNode`s

### 5. `pulsar-graph` — Graph Construction
- Receive IR nodes from frontends
- Build `IrGraph`: link `OrmNode` → `SQLNode` (→ `SchemaNode` in future)
- Provide traversal API for rules

### 6. `pulsar-rules` — Rule Engine
- Rule registry (map of rule_id → `Box<dyn Rule>`)
- Engine traverses graph and calls each rule
- **`NoSelectStar`**: checks if `SQLNode` has implicit column expansion (`SELECT *`)
- Collect `Vec<Diagnostic>`

### 7. `pulsar-diag` — Diagnostics Output
- Pretty-printer: `{file}:{line}:{col}  {severity}  {rule_id}  {message}`
- JSON output for CI (base structure)

### 8. `pulsar-cli` — CLI
- `pulsar check <path>` command via `clap`
- Walk files (respect .gitignore) with `ignore`
- Wire pipeline: parse → IR → graph → rules → diag
- Print diagnostics, exit with error count

## Non-goals for v0.1
- Prisma schema parsing
- Full TypeScript type resolution
- N+1 detection (cross-query analysis)
- Config file (`pulsar.toml`) — hardcoded defaults
- LSP / SARIF output
- `no-missing-limit`, ORM rules, cross-layer rules

## Success Criterion

`cargo run -- check test/fixtures/basic.ts` produces the expected diagnostic output and exits with code 1.
