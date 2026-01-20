use crate::cli::{AiEngine, Cli};
use crate::prd::PrdSource;
use anyhow::{Context, Result};
use colored::*;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    pub ai_engine: AiEngine,
    pub prd_source: PrdSource,
    pub skip_tests: bool,
    pub skip_lint: bool,
    pub skip_commits: bool,
    pub max_iterations: usize,
    pub max_retries: usize,
    pub retry_delay: u64,
    pub dry_run: bool,
    pub parallel: bool,
    pub max_parallel: usize,
    pub branch_per_task: bool,
    pub base_branch: Option<String>,
    pub create_pr: bool,
    pub draft_pr: bool,
    pub verbose: u8,
    pub no_color: bool,
    pub no_notify: bool,
}

impl Config {
    pub fn from_cli(cli: Cli) -> Result<Self> {
        // Extract values that need method calls before destructuring
        let ai_engine = cli.get_ai_engine();
        let skip_tests = cli.skip_tests();
        let skip_lint = cli.skip_lint();
        let skip_commits = cli.skip_commits();

        // Destructure cli to avoid partial move issues
        let Cli {
            github,
            github_label,
            yaml,
            prd,
            max_iterations,
            max_retries,
            retry_delay,
            dry_run,
            parallel,
            max_parallel,
            branch_per_task,
            base_branch,
            create_pr,
            draft_pr,
            verbose,
            no_color,
            no_notify,
            ..
        } = cli;

        // Determine PRD source
        let prd_source = if let Some(github_repo) = github {
            PrdSource::GitHub {
                repo: github_repo,
                label: github_label,
            }
        } else if let Some(yaml_path) = yaml {
            PrdSource::Yaml { path: yaml_path }
        } else {
            PrdSource::Markdown { path: prd }
        };

        // Validate PRD file exists for file-based sources
        if let PrdSource::Markdown { ref path } | PrdSource::Yaml { ref path } = prd_source {
            if !path.exists() {
                anyhow::bail!(
                    "PRD file not found: {}\n\nCreate a PRD file with tasks marked as '- [ ] Task description'\nOr use: --yaml tasks.yaml for YAML task files\nOr use: --github owner/repo for GitHub issues",
                    path.display()
                );
            }
        }

        Ok(Self {
            ai_engine,
            prd_source,
            skip_tests,
            skip_lint,
            skip_commits,
            max_iterations,
            max_retries,
            retry_delay,
            dry_run,
            parallel,
            max_parallel,
            branch_per_task,
            base_branch,
            create_pr,
            draft_pr,
            verbose,
            no_color,
            no_notify,
        })
    }

    pub fn show_banner(&self) {
        if self.no_color {
            colored::control::set_override(false);
        }

        println!("{}", "=".repeat(60).bright_black());
        println!(
            "{} - Running until PRD is complete",
            "Ralphy".bright_cyan().bold()
        );
        println!("Engine: {}", format!("{}", self.ai_engine).bright_magenta());
        println!(
            "Source: {} ({})",
            "PRD".bright_cyan(),
            self.prd_source.display_name().bright_black()
        );

        let mut mode_parts: Vec<String> = Vec::new();
        if self.skip_tests {
            mode_parts.push("no-tests".to_string());
        }
        if self.skip_lint {
            mode_parts.push("no-lint".to_string());
        }
        if self.skip_commits {
            mode_parts.push("no-commits".to_string());
        }
        if self.dry_run {
            mode_parts.push("dry-run".to_string());
        }
        if self.parallel {
            mode_parts.push(format!("parallel:{}", self.max_parallel));
        }
        if self.branch_per_task {
            mode_parts.push("branch-per-task".to_string());
        }
        if self.create_pr {
            mode_parts.push("create-pr".to_string());
        }
        if self.max_iterations > 0 {
            mode_parts.push(format!("max:{}", self.max_iterations));
        }

        if !mode_parts.is_empty() {
            println!("Mode: {}", mode_parts.join(" ").bright_yellow());
        }

        println!("{}", "=".repeat(60).bright_black());
    }
}
