<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="./assets/banner.png">
    <img alt="Pulsar" src="./assets/banner.png" width="600">
  </picture>
</p>

<p align="center">
  <strong>A Rust-powered static analyzer for TypeScript ORM code.</strong>
  <br>
  Detects quality, performance, and consistency issues before they reach production.
</p>

<p align="center">
  <a href="https://github.com/CarlosEduJs/pulsar/actions/workflows/ci.yml"><img src="https://github.com/CarlosEduJs/pulsar/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="https://github.com/CarlosEduJs/pulsar/releases"><img src="https://img.shields.io/badge/version-0.4.0-blue" alt="Version"></a>
</p>

## Quick Start

```bash
# Requires Rust 1.80+
cargo build --release

# Analyze a file or project
cargo run -p pulsar-cli -- check src/queries.ts
cargo run -p pulsar-cli -- check .

# JSON output (for CI / tooling)
cargo run -p pulsar-cli -- check . --format json

# Generate a default config
cargo run -p pulsar-cli -- init
```

See the [Getting Started guide](www/content/docs/guide/getting-started.mdx) for a full walkthrough.

### Example

```
  src/users.ts:5:10  error    no-select-star     Avoid implicit SELECT *.
  src/users.ts:5:10  warning  no-missing-limit    Query is missing a LIMIT clause.

    const users = await db.select().from(users)
                       ^^^^^^^^^^^^^^^^^^^^^^^^

✖ 2 problems (1 error, 1 warning, 0 infos)
```

## Documentation

- [Guide](www/content/docs/guide/) — getting started, CLI, configuration, CI integration
- [Rules](www/content/docs/rules/) — all 12 built-in lint rules with examples
- [Tutorials](www/content/docs/tutorials/) — setup, schema-aware analysis, CI/CD, fixing violations
- [Concepts](www/content/docs/concepts/) — IR graph, schema-aware analysis internals

## Roadmap

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

## Development

### Prerequisites

- Rust 1.80+
- [bun](https://bun.sh) (for the website)

### Commands

```bash
just build        # cargo build
just test         # cargo test --workspace
just clippy       # cargo clippy --all-targets
just check        # fmt + clippy (warnings as errors)
just ci           # fmt + clippy + test + release + smoke
just smoke        # release build + smoke tests
```

### Website

```bash
just www-dev       # bun run dev
just www-build     # bun run build
just www-check     # bun run types:check
just www-lint      # bun run lint
just www-fmt       # bun run fmt
```

## Contributing

Contributions are welcome! Open an issue to discuss bugs or feature ideas. Make sure `cargo clippy --workspace` and `cargo test --workspace` pass before submitting a PR.

## License

MIT — see [LICENSE](LICENSE).
