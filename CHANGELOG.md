# Changelog

All notable changes to Ralphy-RS will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial Rust implementation of Ralphy
- Comprehensive CLI with clap
- Support for multiple AI engines (Claude, OpenCode, Cursor, Codex, Qwen)
- Parallel task execution
- Git integration (branch per task, auto-commits, PR creation)
- Multiple PRD sources (Markdown, YAML, GitHub Issues)
- Progress monitoring with spinners
- Desktop notifications
- Retry logic with configurable delays
- Dry-run mode
- Verbose logging
- Comprehensive test suite
- Beautiful colored terminal output

### Improvements over Bash Version
- Type-safe implementation with Rust
- Better error handling with Result types
- True async/await for parallel execution
- Cross-platform support (no bash dependency)
- 10x faster startup time
- Easier to test and maintain
- Better memory usage
- More robust JSON parsing

## [1.0.0] - 2026-01-18

### Added
- Initial release
- Feature parity with bash version
- Additional Rust-specific improvements

[Unreleased]: https://github.com/yourusername/ralphy-rs/compare/v1.0.0...HEAD
[1.0.0]: https://github.com/yourusername/ralphy-rs/releases/tag/v1.0.0
