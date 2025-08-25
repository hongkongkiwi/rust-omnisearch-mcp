# Justfile for omnisearch-mcp
# https://github.com/casey/just

# Default recipe - show available commands
default:
    @just --list

# Development Setup
# -----------------

# Install all required tools and dependencies
setup:
    rustup component add rustfmt clippy llvm-tools-preview
    cargo install cargo-audit cargo-deny cargo-llvm-cov cargo-watch cargo-outdated --locked
    @echo "✅ Development environment setup complete!"

# Build & Test
# ------------

# Build the project in debug mode
build:
    cargo build

# Build the project in release mode
release:
    cargo build --release

# Run all tests
test:
    cargo test

# Run tests with coverage report
test-coverage:
    cargo llvm-cov --html
    @echo "📊 Coverage report generated at target/llvm-cov/html/index.html"

# Run integration tests only
test-integration:
    cargo test --test '*' -- --test-threads=1

# Run benchmarks
bench:
    cargo bench

# Watch for changes and re-run tests
watch:
    cargo watch -x test

# Code Quality
# ------------

# Run clippy for linting
lint:
    cargo clippy -- -D warnings

# Format code
fmt:
    cargo fmt

# Check formatting without modifying files
fmt-check:
    cargo fmt --check

# Run all quality checks (format, lint, test)
check: fmt-check lint test
    @echo "✅ All checks passed!"

# Security audit for dependencies
audit:
    cargo audit
    cargo deny check

# Check for outdated dependencies
outdated:
    cargo outdated

# Update dependencies
update:
    cargo update
    cargo audit

# Documentation
# -------------

# Build and open documentation
docs:
    cargo doc --open --no-deps

# Build documentation for all dependencies
docs-all:
    cargo doc --open

# Docker & Deployment
# -------------------

# Build Docker image
docker-build:
    docker build -t omnisearch-mcp:latest .

# Run Docker container
docker-run:
    docker run -it --rm \
        -e TAVILY_API_KEY=$TAVILY_API_KEY \
        -e GOOGLE_API_KEY=$GOOGLE_API_KEY \
        -e GOOGLE_SEARCH_ENGINE_ID=$GOOGLE_SEARCH_ENGINE_ID \
        -p 9090:9090 \
        omnisearch-mcp:latest

# Run with Docker Compose (includes monitoring stack)
docker-compose-up:
    docker-compose up -d
    @echo "🚀 Services started!"
    @echo "   - Prometheus: http://localhost:9090"
    @echo "   - Grafana: http://localhost:3000"
    @echo "   - Redis: localhost:6379"

# Stop Docker Compose services
docker-compose-down:
    docker-compose down

# Clean Docker resources
docker-clean:
    docker-compose down -v
    docker rmi omnisearch-mcp:latest || true

# Local Development
# -----------------

# Run the MCP server locally
run:
    cargo run --features mcp

# Run with all features enabled
run-all:
    cargo run --all-features

# Run with specific log level
run-debug:
    RUST_LOG=debug cargo run --features mcp

# Run with Redis caching enabled
run-cached:
    cargo run --features mcp,caching

# Run with metrics enabled
run-metrics:
    cargo run --features mcp,metrics

# Performance Testing
# -------------------

# Run load tests
load-test:
    @echo "🔄 Running load tests..."
    cargo test --test integration_comprehensive -- test_concurrent_operations --nocapture

# Profile CPU usage
profile-cpu:
    cargo build --release
    @echo "📊 Run: perf record -g ./target/release/omnisearch-mcp"
    @echo "📊 Then: perf report"

# Profile memory usage
profile-memory:
    cargo build --release
    valgrind --leak-check=full --show-leak-kinds=all ./target/release/omnisearch-mcp

# CI/CD Commands
# --------------

# Run CI checks locally (mimics GitHub Actions)
ci: fmt-check lint test audit
    cargo check --all-features --all-targets
    @echo "✅ CI checks passed!"

# Check MSRV compatibility
check-msrv:
    cargo +1.82 check --all-features --all-targets

# Generate and commit a changelog entry
changelog MESSAGE:
    @echo "- $(date +%Y-%m-%d): {{MESSAGE}}" >> CHANGELOG.md
    git add CHANGELOG.md
    git commit -m "docs: update changelog"

# Maintenance
# -----------

# Clean build artifacts
clean:
    cargo clean
    rm -rf target/
    @echo "🧹 Build artifacts cleaned!"

# Deep clean including caches
clean-all: clean
    rm -rf ~/.cargo/registry/cache
    rm -rf ~/.cargo/git/db
    @echo "🧹 All caches cleaned!"

# Fix common issues
fix:
    cargo fix --allow-dirty --allow-staged
    cargo fmt
    @echo "🔧 Applied automatic fixes!"

# Create a new release
release-patch:
    cargo release patch --execute

release-minor:
    cargo release minor --execute

release-major:
    cargo release major --execute

# Utility Commands
# ----------------

# Count lines of code
loc:
    @tokei src tests benches

# Show project statistics
stats:
    @echo "📊 Project Statistics:"
    @echo "Lines of code:"
    @tokei src --output json | jq '.Rust.code'
    @echo "Dependencies:"
    @cargo tree --depth 1 | wc -l
    @echo "Test count:"
    @grep -r "#\[test\]" tests src | wc -l

# Open project in browser
open:
    @open https://github.com/hongkongkiwi/rust-omnisearch-mcp

# Environment setup
env-example:
    @echo "Creating .env.example file..."
    @cat > .env.example << 'EOF'
    # API Keys
    TAVILY_API_KEY=your_tavily_api_key_here
    GOOGLE_API_KEY=your_google_api_key_here
    GOOGLE_SEARCH_ENGINE_ID=your_google_search_engine_id_here
    REDDIT_CLIENT_ID=your_reddit_client_id_here
    REDDIT_CLIENT_SECRET=your_reddit_client_secret_here
    BRIGHTDATA_USERNAME=your_brightdata_username_here
    BRIGHTDATA_PASSWORD=your_brightdata_password_here

    # Redis Configuration (optional)
    REDIS_URL=redis://localhost:6379

    # Metrics Configuration (optional)
    PROMETHEUS_PORT=9090

    # Logging
    RUST_LOG=info
    EOF
    @echo "✅ .env.example created!"

# Help & Info
# -----------

# Show detailed help for a recipe
help RECIPE:
    @just --show {{RECIPE}}

# Show system info
info:
    @echo "🦀 Rust version: $(rustc --version)"
    @echo "📦 Cargo version: $(cargo --version)"
    @echo "🎯 Target: $(rustc -vV | grep host | cut -d' ' -f2)"
    @echo "📁 Project: omnisearch-mcp v0.1.0"

# Aliases
# -------

alias b := build
alias t := test
alias r := run
alias l := lint
alias f := fmt
alias d := docs
