# Development Guide

This guide covers development setup, testing, and contributing to omnisearch-mcp.

## Prerequisites

- Rust 1.75 or later
- Cargo (comes with Rust)
- Git

## Setting Up Development Environment

### 1. Clone the Repository

```bash
git clone https://github.com/hongkongkiwi/rust-omnisearch-mcp.git
cd rust-omnisearch-mcp
```

### 2. Install Rust

If you don't have Rust installed:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### 3. Set Up Environment Variables

Create a `.env` file for development:

```bash
# .env
TAVILY_API_KEY=your-dev-key
GOOGLE_API_KEY=your-dev-key
# Add other keys as needed
```

### 4. Build and Run

```bash
# Debug build (faster compilation, slower runtime)
cargo build
cargo run

# Release build (slower compilation, faster runtime)
cargo build --release
./target/release/omnisearch-mcp
```

## Project Structure

```
omnisearch-mcp/
├── src/
│   ├── main.rs              # Entry point
│   ├── server/              # MCP server implementation
│   │   ├── mod.rs          # Server module
│   │   ├── handlers.rs     # Request handlers
│   │   └── tools.rs        # Tool definitions
│   ├── providers/          # Search provider implementations
│   │   ├── mod.rs         # Provider module
│   │   ├── google/        # Google provider
│   │   ├── reddit/        # Reddit provider
│   │   ├── duckduckgo/    # DuckDuckGo provider
│   │   └── ...           # Other providers
│   ├── common/            # Shared utilities
│   │   ├── types.rs      # Common types
│   │   ├── http.rs       # HTTP utilities
│   │   └── macros.rs     # Helper macros
│   └── config/           # Configuration
│       └── mod.rs        # Config module
├── tests/               # Integration tests
├── docs/               # Documentation
└── Cargo.toml         # Project dependencies
```

## Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_google_provider

# Run tests in parallel
cargo test -- --test-threads=4
```

### Test Coverage

Install cargo-tarpaulin for coverage analysis:

```bash
cargo install cargo-tarpaulin
```

Run coverage:

```bash
# Generate coverage report
cargo tarpaulin --ignore-tests

# Generate HTML report
cargo tarpaulin --ignore-tests --out Html

# Generate Cobertura XML (for CI)
cargo tarpaulin --ignore-tests --out Xml
```

### Writing Tests

Example test structure:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_provider_search() {
        // Setup
        let provider = GoogleProvider::new("test-key".to_string());

        // Execute
        let results = provider.search("rust programming", 10).await;

        // Assert
        assert!(results.is_ok());
        assert!(!results.unwrap().is_empty());
    }
}
```

## Adding a New Provider

### 1. Create Provider Module

Create a new module in `src/providers/`:

```rust
// src/providers/newprovider/mod.rs
pub mod search;

use crate::common::types::{SearchResult, ProviderError};

pub struct NewProvider {
    api_key: String,
}

impl NewProvider {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }
}
```

### 2. Implement Search Logic

```rust
// src/providers/newprovider/search.rs
impl NewProvider {
    pub async fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>, ProviderError> {
        // Implement search logic
        todo!()
    }
}
```

### 3. Register Provider

Add to `src/providers/mod.rs`:

```rust
pub mod newprovider;

// In the factory function
if let Ok(api_key) = env::var("NEWPROVIDER_API_KEY") {
    providers.push(Box::new(NewProvider::new(api_key)));
}
```

### 4. Add Tests

Create tests in `tests/newprovider_test.rs`:

```rust
#[tokio::test]
async fn test_newprovider_search() {
    // Test implementation
}
```

## Debugging

### Enable Debug Logging

```bash
RUST_LOG=debug cargo run
```

Logging levels:

- `error`: Only errors
- `warn`: Warnings and errors
- `info`: General information
- `debug`: Detailed debugging
- `trace`: Very detailed tracing

### Using VS Code

`.vscode/launch.json`:

```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "lldb",
      "request": "launch",
      "name": "Debug omnisearch-mcp",
      "cargo": {
        "args": ["build", "--bin=omnisearch-mcp"],
        "filter": {
          "name": "omnisearch-mcp",
          "kind": "bin"
        }
      },
      "args": [],
      "cwd": "${workspaceFolder}",
      "env": {
        "RUST_LOG": "debug",
        "TAVILY_API_KEY": "your-dev-key"
      }
    }
  ]
}
```

## Performance Optimization

### Profiling

Use cargo-flamegraph for performance profiling:

```bash
cargo install flamegraph
cargo build --release
flamegraph -- target/release/omnisearch-mcp
```

### Benchmarking

Add benchmarks in `benches/`:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_search(c: &mut Criterion) {
    c.bench_function("google search", |b| {
        b.iter(|| {
            // Benchmark code
        });
    });
}

criterion_group!(benches, benchmark_search);
criterion_main!(benches);
```

Run benchmarks:

```bash
cargo bench
```

## Code Quality

### Linting

```bash
# Check code with clippy
cargo clippy -- -D warnings

# Format code
cargo fmt

# Check format without changing
cargo fmt -- --check
```

### Pre-commit Hook

Create `.git/hooks/pre-commit`:

```bash
#!/bin/sh
cargo fmt -- --check
cargo clippy -- -D warnings
cargo test
```

## Contributing

### 1. Fork and Clone

```bash
git clone https://github.com/your-username/rust-omnisearch-mcp.git
cd rust-omnisearch-mcp
git remote add upstream https://github.com/hongkongkiwi/rust-omnisearch-mcp.git
```

### 2. Create Feature Branch

```bash
git checkout -b feature/your-feature
```

### 3. Make Changes

- Follow existing code style
- Add tests for new functionality
- Update documentation

### 4. Test Thoroughly

```bash
cargo test
cargo clippy
cargo fmt
```

### 5. Submit Pull Request

- Clear description of changes
- Reference any related issues
- Ensure CI passes

## Release Process

### 1. Update Version

In `Cargo.toml`:

```toml
[package]
version = "0.2.0"
```

### 2. Build Release

```bash
cargo build --release
cargo test --release
```

### 3. Create Tag

```bash
git tag v0.2.0
git push origin v0.2.0
```

### 4. Create GitHub Release

Upload the binary from `target/release/omnisearch-mcp`

## Troubleshooting Development Issues

### Common Problems

1. **Compilation errors**: Update Rust: `rustup update`
2. **Dependency issues**: Clean and rebuild: `cargo clean && cargo build`
3. **Test failures**: Check environment variables are set
4. **Slow compilation**: Use `cargo check` for quick syntax checking

### Getting Help

- Open an issue on GitHub
- Check existing issues for solutions
- Review test files for usage examples
