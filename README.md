# 🤖 Ralphy-RS

**Autonomous AI coding loop - Rust edition**

Named after Ralph Wiggum from The Simpsons - simple but gets the job done! 🎯

Ralphy-RS is a Rust reimplementation of the [original Ralphy bash script](https://github.com/michaelshimeles/ralphy), providing a robust, type-safe way to run AI coding assistants (Claude Code, Codex, OpenCode, Cursor) in a loop until your PRD (Product Requirements Document) is complete.

## ✨ Features

- 🔄 **Autonomous Loop** - Works through tasks until PRD is complete
- 🤖 **Multi-Engine Support** - Claude Code, OpenCode, Cursor, Codex, Qwen-Code
- ⚡ **Parallel Execution** - Run multiple agents simultaneously
- 🌳 **Git Integration** - Branch per task, auto-commits, PR creation
- 📋 **Flexible Task Sources** - Markdown, YAML, or GitHub Issues
- 🎨 **Beautiful CLI** - Colored output, progress indicators, spinners
- 💪 **Type-Safe** - Rust's type system prevents common bugs
- 🔧 **Configurable** - Skip tests/linting, retry logic, dry-run mode
- 🔔 **Notifications** - Desktop notifications on completion

## 📦 Installation

### Prerequisites

- Rust 1.70+ (install from [rustup.rs](https://rustup.rs))
- `jq` - JSON processor
- Git
- At least one AI CLI:
  - [Claude Code](https://github.com/anthropics/claude-code)
  - [OpenCode](https://opencode.ai/docs/)
  - [Cursor](https://cursor.sh) (with `agent` in PATH)
  - Codex CLI
  - Qwen-Code

### Install from source

```bash
git clone https://github.com/MikeTeddyOmondi/ralphy-rs.git
cd ralphy-rs
cargo build --release
sudo cp target/release/ralphy /usr/local/bin/
```

Or install with cargo:

```bash
cargo install --path .
```

## 🚀 Quick Start

### 1. Create a PRD file

```markdown
# My Project

## Tasks

- [ ] Create user authentication
- [ ] Add dashboard page
- [ ] Build API endpoints
```

### 2. Run Ralphy

```bash
ralphy
```

That's it! Ralphy will autonomously work through each task.

## 📚 Usage

### Basic Commands

```bash
# Run with Claude Code (default)
ralphy

# Use different AI engines
ralphy --codex
ralphy --opencode
ralphy --cursor
ralphy --qwen

# Fast mode (skip tests and linting)
ralphy --fast

# Limit iterations
ralphy --max-iterations 5

# Preview what would happen
ralphy --dry-run --verbose
```

### Parallel Execution

Run multiple AI agents simultaneously:

```bash
# 3 agents (default)
ralphy --parallel

# 5 agents
ralphy --parallel --max-parallel 5
```

### Git Workflow

```bash
# Create a branch per task
ralphy --branch-per-task

# Auto-create PRs
ralphy --branch-per-task --create-pr

# Create draft PRs
ralphy --branch-per-task --create-pr --draft-pr

# Specify base branch
ralphy --branch-per-task --base-branch develop
```

### Task Sources

#### Markdown (default)

```bash
ralphy --prd PRD.md
```

Format:
```markdown
## Tasks
- [ ] First task
- [ ] Second task
- [x] Completed task
```

#### YAML

```bash
ralphy --yaml tasks.yaml
```

Format:
```yaml
tasks:
  - title: First task
    completed: false
    parallel_group: 1  # Optional
  - title: Second task
    completed: false
    parallel_group: 1
```

#### GitHub Issues

```bash
# All open issues
ralphy --github owner/repo

# Filter by label
ralphy --github owner/repo --github-label ready
```

## 🎯 Advanced Usage

### Skip Tests or Linting

```bash
# Skip tests
ralphy --no-tests

# Skip linting
ralphy --no-lint

# Skip both (fast mode)
ralphy --fast
```

### Retry Configuration

```bash
# Max retries per task (default: 3)
ralphy --max-retries 5

# Delay between retries (default: 5 seconds)
ralphy --retry-delay 10
```

### Verbose Output

```bash
# Show debug output
ralphy -v

# More verbose
ralphy -vv

# Even more verbose
ralphy -vvv
```

## 📖 Examples

### Complete Feature Branch Workflow

```bash
ralphy \
  --branch-per-task \
  --create-pr \
  --base-branch main \
  --parallel \
  --max-parallel 3
```

### GitHub Issues with Parallel Execution

```bash
ralphy \
  --github myorg/myrepo \
  --github-label sprint-current \
  --parallel \
  --max-parallel 4
```

### Dry Run to Preview

```bash
ralphy --dry-run --verbose
```

## 🏗️ Project Structure

```
ralphy-rs/
├── src/
│   ├── main.rs          # Entry point
│   ├── lib.rs           # Core autonomous loop
│   ├── cli.rs           # Clap CLI definitions
│   ├── config.rs        # Configuration management
│   ├── prd.rs           # PRD parsing (Markdown/YAML/GitHub)
│   ├── ai.rs            # AI engine execution
│   ├── git.rs           # Git operations
│   ├── monitor.rs       # Progress monitoring
│   ├── notifications.rs # Desktop notifications
│   └── prompt.rs        # Prompt building
├── Cargo.toml
└── README.md
```

## 🎨 Features Comparison

| Feature | Bash Version | Rust Version |
|---------|--------------|--------------|
| Multi-engine support | ✅ | ✅ |
| Parallel execution | ✅ | ✅ |
| Git integration | ✅ | ✅ |
| Type safety | ❌ | ✅ |
| Error handling | Basic | Comprehensive |
| Cross-platform | Limited | Full |
| Testing | Hard | Easy |
| Performance | Good | Excellent |

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📝 License

MIT License - see LICENSE file for details

## 🙏 Acknowledgments

- Original [Ralphy](https://github.com/michaelshimeles/ralphy) by Michael Shimeles
- Inspired by the "Ralph Wiggum loop" pattern
- Claude Code team at Anthropic
- The Rust community

## 🐛 Troubleshooting

### AI CLI not found

Make sure your AI CLI is installed and in your PATH:

```bash
which claude  # or opencode, agent, codex, qwen
```

### jq not found

Install jq:

```bash
# macOS
brew install jq

# Ubuntu/Debian
sudo apt-get install jq

# Arch
sudo pacman -S jq
```

### Not a git repository

Ralphy requires a git repository:

```bash
git init
```

### GitHub CLI required

For `--create-pr`:

```bash
# macOS
brew install gh

# Linux
# See https://cli.github.com/
```

## 📊 Performance

Ralphy-RS is significantly faster than the bash version:

- **Startup time**: ~10ms (vs ~100ms bash)
- **Parallel execution**: True async/await (vs background processes)
- **Memory usage**: ~5MB (vs ~20MB+ bash with spawned processes)

## 🔮 Future Enhancements

- [ ] MCP server integration
- [ ] Custom AI engine plugins
- [ ] Web UI for monitoring
- [ ] Task dependency graphs
- [ ] Resume from interruption
- [ ] Cost tracking per task
- [ ] Integration with CI/CD

## 💬 Support

- 🐛 [Report a bug](https://github.com/yourusername/ralphy-rs/issues)
- 💡 [Request a feature](https://github.com/yourusername/ralphy-rs/issues)
- 💬 [Discussions](https://github.com/yourusername/ralphy-rs/discussions)

---

Made with ❤️ and 🦀 Rust
