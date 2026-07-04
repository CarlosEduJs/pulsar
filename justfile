# Development

fmt:
  cargo fmt --all

clippy:
  cargo clippy --all-targets

check: fmt clippy
  cargo clippy --all-targets -- -D warnings

test:
  cargo test --workspace

# Build

build:
  cargo build

release:
  cargo build --release

clean:
  cargo clean

#Run

# Run CLI check against a path (default: test/fixtures/)
run path="test/fixtures/":
  cargo run -p pulsar-cli -- check {{path}}

# Run CLI with JSON format
run-json path="test/fixtures/":
  cargo run -p pulsar-cli -- check {{path}} --format json

# Smoke Tests
# ===========

# Smoke test helpers
_smoke-diags fixture count:
  @result=$$(cargo run --release -p pulsar-cli -- check {{fixture}} 2>&1) || true; \
  echo "$$result"; \
  echo "$$result" | grep -q "no-select-star" || (echo "FAIL: missing rule id"; exit 1); \
  echo "$$result" | grep -q "{{count}} error" || (echo "FAIL: expected {{count}} error(s)"; exit 1)

_smoke-clean fixture:
  @output=$$(cargo run --release -p pulsar-cli -- check {{fixture}} 2>/dev/null); \
  [ -z "$$output" ] || (echo "FAIL: expected no stdout"; exit 1)

# Run all smoke tests
smoke:
  @cargo build --release -p pulsar-cli
  just _smoke-diags test/fixtures/basic.ts 1
  just _smoke-diags test/fixtures/with-where.ts 1
  just _smoke-diags test/fixtures/with-limit.ts 1
  just _smoke-diags test/fixtures/mixed-star-explicit.ts 2
  just _smoke-clean test/fixtures/clean.ts
  just _smoke-clean test/fixtures/no-issues.ts
  @result=$$(cargo run --release -p pulsar-cli -- check test/fixtures/invalid-syntax.ts 2>&1); \
  echo "$$result"; \
  echo "$$result" | grep -q "Error parsing" || (echo "FAIL: expected parse error"; exit 1)
  @echo "=== JSON basic.ts ==="
  @cargo run --release -p pulsar-cli -- check test/fixtures/basic.ts --format json 2>/dev/null | jq -e 'length > 0'
  @echo "=== JSON no-issues.ts ==="
  @cargo run --release -p pulsar-cli -- check test/fixtures/no-issues.ts --format json 2>/dev/null | jq -e 'length == 0'
  @echo "All smoke tests passed"

# CI (mirrors what CI runs)

ci: fmt clippy test release smoke
  @echo "All CI checks passed"

# Explain

# Show rule documentation
explain rule:
  cargo run -p pulsar-cli -- explain {{rule}}

# Init

# Generate pulsar.toml in current directory
init:
  cargo run -p pulsar-cli -- init

# Generate pulsar.toml in current directory (alias)
config: init

# Dist

# Run cargo dist plan
dist-plan:
  dist plan --output-format=json

# Setup

# Install development tools (just)
setup:
  cargo install just

# Website (www/)

# Start dev server
www-dev:
  cd www && bun run dev

# Production build
www-build:
  cd www && bun run build

# Type check (MDX generation + tsc)
www-check:
  cd www && bun run types:check

# Lint
www-lint:
  cd www && bun run lint

# Format
www-fmt:
  cd www && bun run fmt

# Format check
www-fmt-check:
  cd www && bun run fmt:check
