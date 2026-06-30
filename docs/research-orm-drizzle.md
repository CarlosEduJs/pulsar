# Research: Drizzle ORM

> Reference for understanding Drizzle ORM's query builder API so we can detect and extract queries from TypeScript source code via Oxc.

## Overview

Drizzle ORM is a TypeScript ORM known for its SQL-like query builder API. Queries are built using a fluent, chainable syntax that closely mirrors SQL. It generates a single SQL query per chain, making it highly predictable for static analysis.

## Query Builder API

### Select

**Basic select (implicit `SELECT *`):**

```typescript
const users = await db.select().from(users);
// SELECT * FROM "users"
```

**Select with explicit columns:**

```typescript
const users = await db.select({
  id: users.id,
  name: users.name,
}).from(users);
// SELECT "id", "name" FROM "users"
```

### Where

```typescript
import { eq, gt, and, or, like, inArray } from 'drizzle-orm';

db.select().from(users).where(eq(users.id, 1));
db.select().from(users).where(gt(users.age, 18));
db.select().from(users).where(and(eq(users.active, true), gt(users.age, 18)));
db.select().from(users).where(or(eq(users.role, 'admin'), eq(users.role, 'moderator')));
db.select().from(users).where(like(users.email, '%@gmail.com'));
db.select().from(users).where(inArray(users.id, [1, 2, 3]));
```

The `.where()` method accepts either a single filter expression or `undefined` (which is ignored), enabling optional filtering.

### Limit & Offset

```typescript
db.select().from(users).limit(10);
db.select().from(users).limit(10).offset(20);
```

### Order By

```typescript
import { asc, desc } from 'drizzle-orm';

db.select().from(users).orderBy(asc(users.name));
db.select().from(users).orderBy(desc(users.createdAt));
```

### Joins

```typescript
db.select()
  .from(posts)
  .leftJoin(comments, eq(posts.id, comments.post_id));

db.select()
  .from(users)
  .innerJoin(posts, eq(users.id, posts.authorId));
```

## Relational Queries API

Drizzle also provides a higher-level API for fetching nested data without writing explicit joins:

```typescript
// Find all with filters
const users = await db.query.users.findMany({
  where: (users, { eq, gt }) => and(eq(users.active, true), gt(users.age, 18)),
  limit: 10,
  offset: 0,
  orderBy: (users, { desc }) => [desc(users.createdAt)],
  with: {
    posts: true,
    profile: true,
  },
});

// Find first (adds LIMIT 1)
const user = await db.query.users.findFirst({
  where: (users, { eq }) => eq(users.id, 1),
  with: {
    posts: {
      limit: 5,
      where: (posts, { gte }) => gte(posts.createdAt, new Date('2024-01-01')),
    },
  },
});
```

Methods: `.findMany()`, `.findFirst()` (which automatically adds `LIMIT 1`).

## Filter Operators

All operators are importable from `drizzle-orm`:

| Operator | SQL |
|---|---|
| `eq(col, val)` | `col = val` |
| `ne(col, val)` | `col <> val` |
| `gt(col, val)` | `col > val` |
| `gte(col, val)` | `col >= val` |
| `lt(col, val)` | `col < val` |
| `lte(col, val)` | `col <= val` |
| `like(col, pat)` | `col LIKE pat` |
| `ilike(col, pat)` | `col ILIKE pat` |
| `inArray(col, arr)` | `col IN (arr)` |
| `notInArray(col, arr)` | `col NOT IN (arr)` |
| `isNull(col)` | `col IS NULL` |
| `isNotNull(col)` | `col IS NOT NULL` |
| `and(...exprs)` | `(expr1 AND expr2 AND ...)` |
| `or(...exprs)` | `(expr1 OR expr2 OR ...)` |
| `not(expr)` | `NOT (expr)` |

## SQL Generation

Drizzle generates exactly one SQL query per chain. It uses tagged template literals internally (`sql\`...\``). All user-provided values are automatically parameterized (no string interpolation in generated SQL).

The generated SQL can be inspected at runtime:

```typescript
const query = db.select().from(users).where(eq(users.id, 1)).toSQL();
// { sql: 'SELECT * FROM "users" WHERE "users"."id" = $1', params: [1] }
```

## AST Patterns for Detection

From the perspective of the Oxc frontend, here are the key patterns we need to detect:

### Pattern 1: `db.select().from(table)`

AST chain:
```
CallExpression                    → db.select().from(table)  [outer call]
  callee: CallExpression          → db.select().from          [method access + inner call]
    callee: StaticMemberExpression → db.select().from          [property access]
      object: StaticMemberExpression → db.select               [property access]
        object: IdentifierReference → db
        property: IdentifierName  → "select"
      property: IdentifierName   → "from"
    arguments: [table]
  arguments: [] (or [{...}])
```

### Pattern 2: `db.select({ cols }).from(table)` (explicit columns)

Same chain, but the inner `CallExpression` (the `select()` call) has arguments:

```typescript
db.select({ id: users.id, name: users.name }).from(users);
//         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
//         ObjectExpression as argument to select()
```

### Pattern 3: `.where(eq(col, val))`

```typescript
CallExpression                    → .where(eq(users.id, 1))
  callee: StaticMemberExpression → .where
  arguments: [CallExpression]    → eq(users.id, 1)
```

### Pattern 4: Method chaining

```typescript
db.select().from(users).where(eq(...)).limit(10).offset(20);
```

Each `.method()` is a `CallExpression` where:
- `callee` is a `StaticMemberExpression` on the previous call result
- `arguments` contain the method parameters

### Pattern 5: Relational queries (`db.query.users.findMany()`)

```typescript
CallExpression
  callee: StaticMemberExpression
    object: StaticMemberExpression
      object: StaticMemberExpression
        object: IdentifierReference → db
        property: IdentifierName → "query"
      property: IdentifierName → "users"
    property: IdentifierName → "findMany"
  arguments: [{ ... config ... }]
```

## Key Takeaways for Pulsar

1. **`db.select()` without arguments** = `SELECT *` (implicit column expansion)
2. **`db.select({...})` with object argument** = explicit columns (can extract column names from object keys)
3. **Method chain is predictable**: `.from()` always comes after `.select()`, then `.where()`, `.limit()`, `.offset()`, `.orderBy()`, `.leftJoin()`
4. **Relational queries** (`db.query.*.findMany()`) are a separate API that also needs detection
5. **All user values are parameterized** — no SQL injection from Drizzle itself, but detection of raw `sql\`...\`` templates is a separate concern
6. **`.toSQL()`** can be used at runtime to introspect generated SQL, but for static analysis we reconstruct the SQL from the method chain pattern
