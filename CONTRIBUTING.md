# Contributing to Omnisearch MCP

Thank you for your interest in contributing to Omnisearch MCP! This document provides guidelines and setup instructions for contributors.

## Development Setup

### Prerequisites

- [Rust](https://rustup.rs/) (latest stable version)
- [Git](https://git-scm.com/)
- [Python](https://python.org/) 3.7+ (for pre-commit hooks)

### Setting Up the Development Environment

1. **Clone the repository:**

   ```bash
   git clone https://github.com/hongkongkiwi/rust-omnisearch-mcp.git
   cd rust-omnisearch-mcp
   ```

2. **Install Rust dependencies:**

   ```bash
   cargo build
   ```

3. **Install pre-commit hooks (recommended):**

   ```bash
   # Install pre-commit (if not already installed)
   pip install pre-commit
   # or using brew on macOS: brew install pre-commit

   # Install the git hook scripts
   pre-commit install

   # Optionally install pre-push hooks
   pre-commit install --hook-type pre-push
   ```

4. **Install additional tools (optional but recommended):**

   ```bash
   # Security audit tool
   cargo install cargo-audit

   # Better error messages
   cargo install cargo-expand
   ```

### Pre-commit Hooks

This project uses pre-commit hooks to ensure code quality and consistency. The hooks will automatically run before each commit and include:

- **Code formatting:** `cargo fmt`
- **Linting:** `cargo clippy`
- **Basic validation:** `cargo check`
- **Security audits:** `cargo audit` (on pre-push)
- **Tests:** `cargo test` (on pre-push)
- **File formatting:** JSON, YAML, Markdown
- **GitHub Actions validation:** `actionlint`

#### Manual Hook Execution

You can run the pre-commit hooks manually:

```bash
# Run all hooks on all files
pre-commit run --all-files

# Run specific hook
pre-commit run cargo-fmt --all-files

# Run only pre-push hooks
pre-commit run --hook-stage pre-push --all-files
```

#### Skipping Hooks

If you need to skip hooks temporarily:

```bash
# Skip all hooks
git commit --no-verify -m "your commit message"

# Skip specific hook
SKIP=cargo-clippy git commit -m "your commit message"
```

## Development Guidelines

### Code Style

- Follow standard Rust formatting (`cargo fmt`)
- Address all clippy warnings (`cargo clippy`)
- Write meaningful commit messages
- Add tests for new functionality
- Update documentation as needed

### Testing

- Run tests before committing: `cargo test`
- Add unit tests for new functions
- Add integration tests for new features
- Ensure all tests pass in CI

### Commit Messages

Use clear, descriptive commit messages:

```
feat: add new search provider for DuckDuckGo
fix: handle timeout errors in HTTP requests
docs: update installation instructions
test: add integration tests for search functionality
```

### Pull Requests

1. Fork the repository
2. Create a feature branch: `git checkout -b feature-name`
3. Make your changes following the guidelines above
4. Ensure all tests pass: `cargo test`
5. Commit your changes with descriptive messages
6. Push to your fork: `git push origin feature-name`
7. Create a pull request

### Environment Variables

For local development, you may want to set up environment variables for API keys:

```bash
# Copy the example environment file
cp .env.example .env

# Edit .env with your API keys
# Note: .env is gitignored and should not be committed
```

## Architecture

The project is structured as follows:

- `src/` - Main source code
  - `common/` - Shared utilities and types
  - `config/` - Configuration management
  - `providers/` - Search provider implementations
  - `server/` - MCP server implementation
- `tests/` - Test files
- `.github/` - GitHub Actions workflows

## Getting Help

If you have questions or need help:

1. Check existing [GitHub Issues](https://github.com/hongkongkiwi/rust-omnisearch-mcp/issues)
2. Create a new issue with detailed information
3. Review the documentation in the README

## License

By contributing to this project, you agree that your contributions will be licensed under the same license as the project.
