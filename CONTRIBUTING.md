# Contributing to Ralphy-RS

First off, thank you for considering contributing to Ralphy-RS! 🎉

## Code of Conduct

Be respectful, inclusive, and constructive in all interactions.

## How Can I Contribute?

### Reporting Bugs

Before creating bug reports, please check existing issues. When creating a bug report, include:

- **Clear title and description**
- **Steps to reproduce**
- **Expected vs actual behavior**
- **Environment details** (OS, Rust version, AI CLI version)
- **Relevant logs or screenshots**

### Suggesting Enhancements

Enhancement suggestions are tracked as GitHub issues. When creating an enhancement suggestion, include:

- **Clear title and description**
- **Use case** - Why is this enhancement useful?
- **Proposed solution**
- **Alternatives considered**

### Pull Requests

1. Fork the repo and create your branch from `main`
2. Make your changes
3. Add tests for any new functionality
4. Ensure the test suite passes: `cargo test`
5. Format your code: `cargo fmt`
6. Run clippy: `cargo clippy`
7. Update documentation as needed
8. Write a clear commit message

## Development Setup

### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install development tools
cargo install cargo-watch
cargo install just

# Install required system dependencies
# macOS
brew install jq

# Ubuntu/Debian
sudo apt-get install jq
```

### Building

```bash
# Clone your fork
git clone https://github.com/YOUR_USERNAME/ralphy-rs.git
cd ralphy-rs

# Build
cargo build

# Run tests
cargo test

# Run with examples
cargo run -- --prd examples/PRD.md --dry-run
```

### Running Tests

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_markdown_prd_parsing

# Watch mode
cargo watch -x test
```

### Code Style

We follow the official Rust style guidelines:

- Run `cargo fmt` before committing
- Run `cargo clippy` and fix warnings
- Write clear, self-documenting code
- Add comments for complex logic
- Use meaningful variable names

### Commit Messages

Follow the [Conventional Commits](https://www.conventionalcommits.org/) specification:

```
feat: add support for custom AI engines
fix: resolve panic when PRD file is empty
docs: update installation instructions
test: add tests for YAML parsing
refactor: simplify prompt building logic
```

## Project Structure

```
ralphy-rs/
├── src/
│   ├── main.rs          # CLI entry point
│   ├── lib.rs           # Core library
│   ├── cli.rs           # Command-line parsing
│   ├── config.rs        # Configuration
│   ├── prd.rs           # PRD parsing
│   ├── ai.rs            # AI engine execution
│   ├── git.rs           # Git operations
│   ├── monitor.rs       # Progress monitoring
│   ├── notifications.rs # Desktop notifications
│   └── prompt.rs        # Prompt building
├── tests/
│   └── integration_tests.rs
├── examples/
│   ├── PRD.md
│   └── tasks.yaml
└── Cargo.toml
```

## Testing Guidelines

- Write tests for new features
- Maintain or improve code coverage
- Test edge cases and error conditions
- Use meaningful test names
- Add integration tests for user-facing features

## Documentation

- Update README.md for user-facing changes
- Add inline documentation for public APIs
- Include examples in doc comments
- Update CHANGELOG.md

## Release Process

1. Update version in `Cargo.toml`
2. Update CHANGELOG.md
3. Create a git tag: `git tag v1.0.0`
4. Push tag: `git push origin v1.0.0`
5. Create GitHub release

## Getting Help

- 💬 [GitHub Discussions](https://github.com/yourusername/ralphy-rs/discussions)
- 🐛 [Issue Tracker](https://github.com/yourusername/ralphy-rs/issues)

## Recognition

Contributors will be recognized in:
- README.md
- Release notes
- GitHub contributors page

Thank you for contributing! 🦀❤️
