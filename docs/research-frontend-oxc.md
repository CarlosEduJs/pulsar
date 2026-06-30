# Research: Oxc (Oxidation Compiler)

> Reference for understanding Oxc's parser and AST API so we can build `pulsar-frontend-oxc`.

## What is Oxc

Oxc is a collection of high-performance Rust crates for JavaScript/TypeScript tooling. It includes a parser, AST types, semantic analysis, linter, transformer, minifier, and codegen — all written in Rust. We use it in Pulsar to parse TypeScript source files and extract Drizzle ORM calls from the AST.

## Architecture

The `oxc` crate at v0.44 is a **facade crate** that re-exports APIs from sub-crates:

| Module path | Sub-crate | Purpose |
|---|---|---|
| `oxc::allocator` | `oxc_allocator` | Arena allocator (bump allocator) |
| `oxc::ast` | `oxc_ast` | AST types and visitor traits |
| `oxc::parser` | `oxc_parser` | Parser entry point |
| `oxc::span` | `oxc_span` | Source locations (`Span`) |
| `oxc::syntax` | `oxc_syntax` | Language syntax utilities |
| `oxc::diagnostics` | `oxc_diagnostics` | Error reporting |
| `oxc::semantic` | `oxc_semantic` | Semantic analysis (feature-gated) |
| `oxc::codegen` | `oxc_codegen` | Code generation (feature-gated) |
| `oxc::transformer` | `oxc_transformer` | AST transforms (feature-gated) |

For v0.1 of Pulsar, we only need `oxc::allocator`, `oxc::parser`, `oxc::ast`, and `oxc::span`.

All AST nodes are **arena-allocated** using `oxc_allocator::Allocator` (a bump allocator). The lifetime `'a` ties all AST nodes to the allocator instance.

## Parsing TypeScript

### Entry Point

```rust
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;

let allocator = Allocator::default();
let source_text = "const x: number = 1;";
let source_type = SourceType::ts();

let ret = Parser::new(&allocator, source_text, source_type).parse();
let program = ret.program;         // Program<'a>, the root AST node
let errors = ret.errors;           // Vec<OxcDiagnostic>, syntax errors
```

### SourceType

```rust
SourceType::ts();                      // .ts file
SourceType::tsx();                     // .tsx file
SourceType::js();                      // .js file
SourceType::jsx();                     // .jsx file
SourceType::mjs();                     // .mjs file
SourceType::cjs();                     // .cjs file
SourceType::from_path("path/file.ts"); // infer from extension
```

### ParserReturn

```rust
pub struct ParserReturn<'a> {
    pub program: Program<'a>,
    pub module_record: ModuleRecord<'a>,
    pub errors: Vec<OxcDiagnostic>,
    pub panicked: bool,
}
```

### Program

```rust
pub struct Program<'a> {
    pub span: Span,
    pub source_type: SourceType,
    pub source_text: &'a str,
    pub comments: Comments<'a>,
    pub hashbang: Option<Hashbang<'a>>,
    pub directives: Vec<'a, Directive<'a>>,
    pub body: Vec<'a, Statement<'a>>,
    pub scope_id: Cell<Option<ScopeId>>,
}
```

## Key AST Types

All types live in `oxc_ast::ast` (re-exported as `oxc::ast`).

### `Expression` Enum

A 16-byte enum covering all expression types. Key variants we care about:

```rust
pub enum Expression<'a> {
    // Variants inherited from MemberExpression:
    ComputedMemberExpression(Box<'a, ComputedMemberExpression<'a>>),  // arr[0]
    StaticMemberExpression(Box<'a, StaticMemberExpression<'a>>),      // obj.prop
    PrivateFieldExpression(Box<'a, PrivateFieldExpression<'a>>),      // obj.#field

    // Direct variants:
    IdentifierReference(Box<'a, IdentifierReference<'a>>),            // variable names
    StringLiteral(Box<'a, StringLiteral<'a>>),                        // "hello"
    CallExpression(Box<'a, CallExpression<'a>>),                      // fn()
    ArrowFunctionExpression(Box<'a, ArrowFunctionExpression<'a>>),    // () => {}
    ObjectExpression(Box<'a, ObjectExpression<'a>>),                  // { key: val }
    // ... ~40 more variants
}
```

The `inherit_variants!` macro flattens `MemberExpression` variants directly into `Expression`, so you can match `Expression::StaticMemberExpression(...)` without going through a nested enum.

### `CallExpression`

Our primary target — represents every function/method call like `db.select()`.

```rust
pub struct CallExpression<'a> {
    pub span: Span,
    pub callee: Expression<'a>,                           // the thing being called
    pub type_parameters: Option<Box<'a, TSTypeParameterInstantiation<'a>>>,
    pub arguments: Vec<'a, Argument<'a>>,                 // function arguments
    pub optional: bool,                                   // optional chaining `?.()`
}
```

### `StaticMemberExpression`

Represents property access with a dot: `db.select`, `users.id`.

```rust
pub struct StaticMemberExpression<'a> {
    pub span: Span,
    pub object: Expression<'a>,         // the left side (e.g., `db`)
    pub property: IdentifierName<'a>,   // the property name (e.g., `select`)
    pub optional: bool,
}
```

### `IdentifierReference`

Represents a variable name being referenced: `db`, `users`, `eq`.

```rust
pub struct IdentifierReference<'a> {
    pub span: Span,
    pub name: Atom<'a>,
    pub reference_id: Cell<Option<ReferenceId>>,
}
```

`Atom<'a>` is a newtype around `&'a str`, stored in the allocator arena.

### StringLiteral

```rust
pub struct StringLiteral<'a> {
    pub span: Span,
    pub value: Atom<'a>,          // the parsed string value
    pub raw: Option<Atom<'a>>,    // the raw source text (None if synthetic)
}
```

### ObjectExpression

Represents `{ key: value }` object literals, used in `db.select({ id: users.id })`.

```rust
pub struct ObjectExpression<'a> {
    pub span: Span,
    pub properties: Vec<'a, ObjectPropertyKind<'a>>,
}
```

### Argument

Wraps either a `SpreadElement` or any `Expression`:

```rust
pub enum Argument<'a> {
    SpreadElement(Box<'a, SpreadElement<'a>>),
    Expression(Expression<'a>),
}
```

### IdentifierName

Used for property names in member expressions and object literal keys.

```rust
pub struct IdentifierName<'a> {
    pub span: Span,
    pub name: Atom<'a>,
}
```

## Statement Enum

Top-level statements in a `Program` body. Key variants:

```rust
pub enum Statement<'a> {
    ExpressionStatement(Box<'a, ExpressionStatement<'a>>),  // standalone expression
    VariableDeclaration(Box<'a, VariableDeclaration<'a>>),  // const/let/var
    ReturnStatement(Box<'a, ReturnStatement<'a>>),
    IfStatement(Box<'a, IfStatement<'a>>),
    ForStatement(Box<'a, ForStatement<'a>>),
    // ... inherits Declaration and ModuleDeclaration variants
}
```

Export/import declarations live under `Declaration` and `ModuleDeclaration` which are flattened into `Statement` via `inherit_variants!`.

## Traversal

### `Visit` Trait

Provides immutable traversal of the AST with pre-order and post-order hooks:

```rust
pub trait Visit<'a>: Sized {
    // Hook called BEFORE children are visited
    fn enter_node(&mut self, kind: AstKind<'a>) {}

    // Hook called AFTER children are visited
    fn leave_node(&mut self, kind: AstKind<'a>) {}

    // Per-node visitor methods (default impl walks children)
    fn visit_program(&mut self, it: &Program<'a>) { walk_program(self, it); }
    fn visit_call_expression(&mut self, it: &CallExpression<'a>) { ... }
    fn visit_expression(&mut self, it: &Expression<'a>) { ... }
    // ... 200+ methods, one per AST node type
}
```

Each `walk_*` function:
1. Creates an `AstKind` variant wrapping a reference to the node
2. Calls `visitor.enter_node(kind)`
3. Visits all child fields
4. Calls `visitor.leave_node(kind)`

### `AstKind` Enum

An enum with ~150 variants, each wrapping a reference to a specific AST node type. Used for untyped matching in `enter_node`/`leave_node`:

```rust
pub enum AstKind<'a> {
    Program(&'a Program<'a>),
    CallExpression(&'a CallExpression<'a>),
    StaticMemberExpression(&'a StaticMemberExpression<'a>),
    IdentifierReference(&'a IdentifierReference<'a>),
    ObjectExpression(&'a ObjectExpression<'a>),
    ArrowFunctionExpression(&'a ArrowFunctionExpression<'a>),
    StringLiteral(&'a StringLiteral<'a>),
    // ...
}
```

Typical usage pattern:

```rust
impl<'a> Visit<'a> for MyVisitor {
    fn enter_node(&mut self, kind: AstKind<'a>) {
        match kind {
            AstKind::CallExpression(expr) => { /* handle call */ }
            _ => {}
        }
    }
}
```

### Semantic Traversal (Alternative)

Oxc also provides a flat node iteration API through the semantic layer:

```rust
let semantic = SemanticBuilder::new().build(&program);
for node in semantic.nodes().iter() {
    match node.kind() {
        AstKind::CallExpression(expr) => { /* handle call */ }
        _ => {}
    }
}
```

This may be useful for simpler, ad-hoc analysis without implementing the full `Visit` trait.

## Source Locations (`Span`)

```rust
pub struct Span {
    pub start: u32,   // zero-based byte offset from start of source
    pub end: u32,     // zero-based byte offset (exclusive)
}
```

Key properties:
- **byte offsets**, not line/column — no built-in line/col conversion
- Every AST node has `pub span: Span` as its first field
- `u32` — supports files up to 4 GiB

Useful methods:
```rust
Span::new(start, end);           // create from offsets
span.size();                     // byte length (end - start)
span.source_text(text);          // extract source substring
span.label("msg");               // create LabeledSpan for diagnostics
source_text[span];               // index into &str with Span
GetSpan::span(&node);            // trait implemented by all AST nodes
```

To convert byte offsets to line/column, we need to do it ourselves:
- Maintain a mapping of byte offsets to line numbers when reading the file
- Or compute on the fly by scanning the source text

## Patterns for Pulsar

### Detecting `db.select()` Chains

The typical detection approach:

1. **Parse** the source file with `Parser`
2. **Visit** each `CallExpression` in the AST
3. For each call, check if `callee` is a `StaticMemberExpression`
4. Walk up the chain to see if the call belongs to a `db.select()...from()...` pattern
5. Extract relevant info (columns from select args, table from from arg, where conditions, limit value)

### Example: `db.select().from(users)`

```
CallExpression                          → outer: db.select().from(users)
  callee: StaticMemberExpression        → .from
    object: CallExpression              → inner: db.select()
      callee: StaticMemberExpression    → .select
        object: IdentifierReference     → db
        property: "select"
      arguments: []                     → select() has no args → SELECT *
    property: "from"
  arguments: [IdentifierReference(users)] → table
```

### Example: `db.select({ id: users.id }).from(users).where(eq(users.id, 1))`

```
CallExpression                          → .where(...)
  callee: StaticMemberExpression
    object: CallExpression               → .from(...)
      callee: StaticMemberExpression
        object: CallExpression           → .select({...})
          callee: StaticMemberExpression → .select
            object: IdentifierReference  → db
            property: "select"
          arguments: [ObjectExpression]  → { id: users.id }
        property: "from"
      arguments: [IdentifierReference]   → table
    property: "where"
  arguments: [CallExpression]           → eq(users.id, 1)
```

### Extracting Arguments

- **`select()` without args** → implicit `SELECT *`
- **`select({...})`** → explicit columns from object property keys
- **`.from(arg)`** → table name from `IdentifierReference`
- **`.where(arg)`** → filter expression (needs analysis of the expression tree)
- **`.limit(n)`** → value from `NumericLiteral`
- **`.offset(n)`** → value from `NumericLiteral`

### Error Handling

The parser returns `errors: Vec<OxcDiagnostic>` for syntax errors. For our v0.1:
- If parsing fails completely (panicked), skip the file
- If there are minor syntax errors, we can still traverse the partial AST

## Key Takeaways for Pulsar

1. **Parser setup** is straightforward: `Allocator` + `Parser::new()` + `SourceType::from_path()`
2. **Method chains** are nested `CallExpression` → `StaticMemberExpression` → `CallExpression` → ...
3. **`AstKind::CallExpression`** is the primary match target in the visitor
4. **No line/col** from Span — we need to compute it ourselves from byte offsets
5. **`Atom<'a>`** wraps `&'a str` — use `.as_str()` or `.to_string()` for extraction
6. **`inherit_variants!`** flattens `MemberExpression` into `Expression` — match directly on `Expression::StaticMemberExpression(...)`
7. **Arena allocation** means all AST nodes borrow from the `Allocator` — the allocator must outlive the traversal
