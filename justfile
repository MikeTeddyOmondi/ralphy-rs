# Justfile for Ralphy-RS
# Install just: cargo install just

# Default recipe (list all commands)
default:
    @just --list

# Build the project
build:
    cargo build

# Build in release mode
build-release:
    cargo build --release

# Run the project with default settings
run *ARGS:
    cargo run -- {{ARGS}}

# Run tests
test:
    cargo test

# Run tests with output
test-verbose:
    cargo test -- --nocapture

# Run clippy linter
lint:
    cargo clippy -- -D warnings

# Format code
fmt:
    cargo fmt

# Check formatting
fmt-check:
    cargo fmt -- --check

# Run all checks (fmt, clippy, test)
check: fmt-check lint test

# Install locally
install:
    cargo install --path .

# Clean build artifacts
clean:
    cargo clean

# Build and install
install-build: build-release install

# Run with example PRD
example:
    cargo run -- --prd examples/PRD.md --dry-run -v

# Run with example YAML
example-yaml:
    cargo run -- --yaml examples/tasks.yaml --dry-run -v

# Generate documentation
docs:
    cargo doc --open

# Watch for changes and run tests
watch:
    cargo watch -x test

# Run with different engines
run-claude:
    cargo run -- --claude --dry-run

run-opencode:
    cargo run -- --opencode --dry-run

run-cursor:
    cargo run -- --cursor --dry-run

run-codex:
    cargo run -- --codex --dry-run

# Parallel execution examples
run-parallel:
    cargo run -- --parallel --max-parallel 4 --dry-run

run-parallel-branches:
    cargo run -- --parallel --branch-per-task --dry-run
