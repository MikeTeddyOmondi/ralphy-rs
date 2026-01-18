use clap::{ArgAction, Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "ralphy",
    version,
    about = "Autonomous AI coding loop - Rust edition",
    long_about = "Runs AI coding assistants (Claude Code, Codex, OpenCode, Cursor) in a loop until your PRD is complete.\n\n\
                  Named after Ralph Wiggum from The Simpsons - simple but gets the job done! 🎯"
)]
#[command(after_help = "EXAMPLES:\n  \
    ralphy                                    # Run with Claude Code\n  \
    ralphy --codex                            # Run with Codex CLI\n  \
    ralphy --opencode                         # Run with OpenCode\n  \
    ralphy --cursor                           # Run with Cursor agent\n  \
    ralphy --parallel --max-parallel 4        # Run 4 tasks concurrently\n  \
    ralphy --branch-per-task --create-pr      # Feature branch workflow\n  \
    ralphy --yaml tasks.yaml                  # Use YAML task file\n  \
    ralphy --github owner/repo                # Fetch from GitHub issues\n  \
    ralphy --fast                             # Skip tests and linting\n  \
    ralphy --dry-run --verbose                # Preview what would happen\n\
")]
pub struct Cli {
    // ============================================
    // AI ENGINE OPTIONS
    // ============================================
    /// Use Claude Code (default)
    #[arg(long, conflicts_with_all = ["opencode", "cursor", "codex", "qwen"])]
    pub claude: bool,

    /// Use OpenCode
    #[arg(long, conflicts_with_all = ["claude", "cursor", "codex", "qwen"])]
    pub opencode: bool,

    /// Use Cursor agent
    #[arg(long, alias = "agent", conflicts_with_all = ["claude", "opencode", "codex", "qwen"])]
    pub cursor: bool,

    /// Use Codex CLI
    #[arg(long, conflicts_with_all = ["claude", "opencode", "cursor", "qwen"])]
    pub codex: bool,

    /// Use Qwen-Code
    #[arg(long, conflicts_with_all = ["claude", "opencode", "cursor", "codex"])]
    pub qwen: bool,

    // ============================================
    // WORKFLOW OPTIONS
    // ============================================
    /// Skip writing and running tests
    #[arg(long, alias = "skip-tests")]
    pub no_tests: bool,

    /// Skip linting
    #[arg(long, alias = "skip-lint")]
    pub no_lint: bool,

    /// Skip both tests and linting (shorthand for --no-tests --no-lint)
    #[arg(long)]
    pub fast: bool,

    // ============================================
    // EXECUTION OPTIONS
    // ============================================
    /// Stop after N iterations (0 = unlimited)
    #[arg(long, default_value = "0", value_name = "N")]
    pub max_iterations: usize,

    /// Max retries per task on failure
    #[arg(long, default_value = "3", value_name = "N")]
    pub max_retries: usize,

    /// Seconds between retries
    #[arg(long, default_value = "5", value_name = "N")]
    pub retry_delay: u64,

    /// Show what would be done without executing
    #[arg(long)]
    pub dry_run: bool,

    // ============================================
    // PARALLEL EXECUTION
    // ============================================
    /// Run independent tasks in parallel
    #[arg(long)]
    pub parallel: bool,

    /// Max concurrent tasks (only with --parallel)
    #[arg(long, default_value = "3", value_name = "N", requires = "parallel")]
    pub max_parallel: usize,

    // ============================================
    // GIT BRANCH OPTIONS
    // ============================================
    /// Create a new git branch for each task
    #[arg(long)]
    pub branch_per_task: bool,

    /// Base branch to create task branches from (default: current branch)
    #[arg(long, value_name = "NAME", requires = "branch_per_task")]
    pub base_branch: Option<String>,

    /// Create a pull request after each task (requires gh CLI)
    #[arg(long, requires = "branch_per_task")]
    pub create_pr: bool,

    /// Create PRs as drafts
    #[arg(long, requires = "create_pr")]
    pub draft_pr: bool,

    // ============================================
    // PRD SOURCE OPTIONS
    // ============================================
    /// PRD file path (markdown format with checkboxes)
    #[arg(
        long,
        value_name = "FILE",
        default_value = "PRD.md",
        conflicts_with_all = ["yaml", "github"]
    )]
    pub prd: PathBuf,

    /// Use YAML task file instead of markdown
    #[arg(
        long,
        value_name = "FILE",
        conflicts_with_all = ["prd", "github"]
    )]
    pub yaml: Option<PathBuf>,

    /// Fetch tasks from GitHub issues (format: owner/repo)
    #[arg(
        long,
        value_name = "REPO",
        conflicts_with_all = ["prd", "yaml"]
    )]
    pub github: Option<String>,

    /// Filter GitHub issues by label
    #[arg(long, value_name = "TAG", requires = "github")]
    pub github_label: Option<String>,

    // ============================================
    // OTHER OPTIONS
    // ============================================
    /// Show debug output
    #[arg(short, long, action = ArgAction::Count)]
    pub verbose: u8,

    /// Disable colored output
    #[arg(long)]
    pub no_color: bool,

    /// Disable notifications
    #[arg(long)]
    pub no_notify: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum AiEngine {
    Claude,
    OpenCode,
    Cursor,
    Codex,
    Qwen,
}

impl std::fmt::Display for AiEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AiEngine::Claude => write!(f, "Claude Code"),
            AiEngine::OpenCode => write!(f, "OpenCode"),
            AiEngine::Cursor => write!(f, "Cursor"),
            AiEngine::Codex => write!(f, "Codex"),
            AiEngine::Qwen => write!(f, "Qwen-Code"),
        }
    }
}

impl Cli {
    pub fn get_ai_engine(&self) -> AiEngine {
        if self.opencode {
            AiEngine::OpenCode
        } else if self.cursor {
            AiEngine::Cursor
        } else if self.codex {
            AiEngine::Codex
        } else if self.qwen {
            AiEngine::Qwen
        } else {
            AiEngine::Claude
        }
    }

    pub fn get_prd_file(&self) -> PathBuf {
        if let Some(ref yaml) = self.yaml {
            yaml.clone()
        } else {
            self.prd.clone()
        }
    }

    pub fn skip_tests(&self) -> bool {
        self.no_tests || self.fast
    }

    pub fn skip_lint(&self) -> bool {
        self.no_lint || self.fast
    }
}
