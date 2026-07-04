# Contributing

Thanks for your interest in Pulsar!

## Bugs & feature requests

Open an [issue](https://github.com/CarlosEduJs/pulsar/issues) to report bugs or
suggest ideas. For bugs, include a minimal reproduction if possible.

## Quick start

```bash
# Prerequisites: Rust 1.80+, bun, just
git clone https://github.com/CarlosEduJs/pulsar.git
cd pulsar
cargo build --release

# Run tests
cargo test --workspace

# Run lints
cargo clippy --all-targets -- -D warnings
cargo fmt --all --check
```

## Development

```bash
just check         # fmt + clippy (warnings as errors)
just test          # cargo test --workspace
just ci            # fmt + clippy + test + release + smoke
just www-check     # type check the website
```

See the [justfile](justfile) for all available commands.

## Pull requests

- Open a **draft PR** early — it signals work-in-progress and invites early feedback.
- Make sure CI passes before requesting review.
- The PR **title** should follow [Conventional Commits](https://www.conventionalcommits.org/)
  (e.g. `feat: add rule no-unused-import`, `fix: handle nullable columns`).
  The title is used as the commit message on squash-merge.

## Architecture

See [`docs/wiki.md`](docs/wiki.md) for the full architecture blueprint,
pipeline overview, and crate descriptions.
