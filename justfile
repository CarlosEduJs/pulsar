# Development
# ===========

# Format workspace + fuzz targets
fmt:
    cargo fmt --all
    -cargo +nightly fmt --manifest-path fuzz/Cargo.toml --all

# Clippy workspace (all targets including tests)
clippy:
    cargo clippy --workspace --all-targets -- -D warnings

# Format + clippy
check: fmt clippy

# Run workspace tests + integration tests + proptests
test:
    cargo test --workspace

# Test with verbose output (show names)
test-verbose:
    cargo test --workspace -- --nocapture

# Quick check: fmt + clippy + test (no release build)
quick: fmt clippy test

# Build
# =====

build:
    cargo build

release:
    cargo build --release

clean:
    cargo clean

# Run
# ===

# Run CLI check against a path (default: test/fixtures/)
run path="test/fixtures/":
    cargo run -p pulsar-cli -- check {{ path }}

# Run CLI with JSON format
run-json path="test/fixtures/":
    cargo run -p pulsar-cli -- check {{ path }} --format json

# Run CLI with schema config
run-with-schema path="test/fixtures/schema/":
    cargo run -p pulsar-cli -- check {{ path }} --config test/fixtures/schema/pulsar.toml

# Run CLI on a single rule fixture (detects select-star violations by default)
#   just run-rule no-missing-limit
# just run-rule no-unindexed-filter --config test/fixtures/schema/pulsar.toml
run-rule rule args="":
    cargo run -p pulsar-cli -- check test/fixtures/{{ rule }} {{ args }}

# Explain
# =======

# Show rule documentation
explain rule:
    cargo run -p pulsar-cli -- explain {{ rule }}

# Init / Config
# =============

# Generate pulsar.toml in current directory
init:
    cargo run -p pulsar-cli -- init

# Generate pulsar.toml in current directory (alias)
config: init

# Dist
# ====

# Run cargo dist plan
dist-plan:
    dist plan --output-format=json

# Publish Homebrew formula to the tap repo (requires GH_TOKEN with repo scope)
dist-publish-tap tag:
    dist host --tag={{ tag }} --steps=publish --output-format=json

# Full dist publish (release + homebrew tap)
dist-publish tag:
    dist host --tag={{ tag }} --steps=upload --steps=release --steps=publish --output-format=json

# Setup
# =====

# Install development tools
setup:
    cargo install just
    cargo +nightly install cargo-fuzz --locked

# Fuzz Testing
# ============

# Build all fuzz targets (needs nightly)
fuzz:
    cargo +nightly fuzz build

# Run a specific fuzz target
#   just fuzz-run fuzz_sql_parser
# just fuzz-run fuzz_combined
fuzz-run target="fuzz_sql_parser":
    cargo +nightly fuzz run {{ target }}

# Run a fuzz target for a limited time (seconds)
# just fuzz-run-for fuzz_sql_parser 30
fuzz-run-for target="fuzz_sql_parser" seconds="10":
    cargo +nightly fuzz run {{ target }} -- -max_total_time={{ seconds }}

# Run each fuzz target briefly (quick smoke check)
fuzz-smoke:
    cargo +nightly fuzz run fuzz_sql_parser -- -runs=500
    cargo +nightly fuzz run fuzz_prisma_parser -- -runs=500
    cargo +nightly fuzz run fuzz_oxc_frontend -- -runs=500
    cargo +nightly fuzz run fuzz_combined -- -runs=500

# Clean fuzz artifacts
fuzz-clean:
    rm -rf fuzz/corpus fuzz/artifacts fuzz/target fuzz/coverage

# Demos
# =====

# Run demo: basic rules
demo:
    -cargo run --release -p pulsar-cli -- check demos/

# Run demo: schema-aware rules
demo-schema path="demos/schema-aware/":
    -cargo run --release -p pulsar-cli -- check {{ path }} --config demos/schema/pulsar.toml

# Smoke Tests
# ===========

# Smoke test helpers
_smoke-diags fixture count:
    @result=$$(cargo run --release -p pulsar-cli -- check {{ fixture }} 2>&1) || true; \
    echo "$$result"; \
    echo "$$result" | grep -q "no-select-star" || (echo "FAIL: missing rule id"; exit 1); \
    echo "$$result" | grep -q "{{ count }} error" || (echo "FAIL: expected {{ count }} error(s)"; exit 1)

_smoke-clean fixture:
    @output=$$(cargo run --release -p pulsar-cli -- check {{ fixture }} 2>/dev/null); \
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

# Smoke tests with Prisma schema
smoke-schema:
    @cargo build --release -p pulsar-cli
    @echo "=== no-unindexed-filter ==="
    cargo run --release -p pulsar-cli -- check test/fixtures/no-unindexed-filter --config test/fixtures/schema/pulsar.toml 2>/dev/null | grep -q "no-unindexed-filter" && echo "PASS"
    @echo "=== no-unknown-column ==="
    cargo run --release -p pulsar-cli -- check test/fixtures/no-unknown-column --config test/fixtures/schema/pulsar.toml 2>/dev/null | grep -q "no-unknown-column" && echo "PASS"
    @echo "=== no-missing-foreign-key (clean) ==="
    cargo run --release -p pulsar-cli -- check test/fixtures/no-missing-foreign-key/clean.ts --config test/fixtures/schema/pulsar.toml 2>/dev/null | grep -q . && echo "FAIL: expected clean" || echo "PASS"

# CI
# ==

# Full CI pipeline (mirrors GitHub Actions)
ci: fmt clippy test release smoke
    @echo "All CI checks passed"

# Count tests per category
test-count:
    @echo "=== Unit tests ==="
    @cargo test --workspace -- --list 2>/dev/null | wc -l
    @echo "=== Integration tests ==="
    @cargo test -p pulsar-integration-tests -- --list 2>/dev/null | wc -l
    @echo "=== Fuzz targets ==="
    @ls fuzz/fuzz_targets/*.rs | wc -l

# Website (www/)
# ==============

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
