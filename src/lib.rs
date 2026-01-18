#![allow(unused)]
#![allow(dead_code)]
#![allow(unused_imports)]

pub mod ai;
pub mod cli;
pub mod config;
pub mod git;
pub mod monitor;
pub mod notifications;
pub mod prd;
pub mod prompt;

use anyhow::{Context, Result};
use colored::*;
use config::Config;
use futures::future::join_all;
use prd::PrdManager;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

pub async fn run_autonomous_loop(config: Config) -> Result<()> {
    // Pre-flight checks
    preflight_checks(&config).await?;

    // Create managers
    let prd_manager = Arc::new(PrdManager::new(config.prd_source.clone()));

    if config.parallel {
        run_parallel_loop(config, prd_manager).await
    } else {
        run_sequential_loop(config, prd_manager).await
    }
}

async fn preflight_checks(config: &Config) -> Result<()> {
    // Check AI CLI availability
    ai::check_ai_availability(config.ai_engine)?;

    // Check for jq
    if std::process::Command::new("which")
        .arg("jq")
        .stdout(std::process::Stdio::null())
        .status()?
        .success()
        == false
    {
        eprintln!(
            "{} jq is required but not installed. Install with: apt-get install jq (Debian/Ubuntu) or brew install jq (macOS)",
            "[ERROR]".red().bold()
        );
        anyhow::bail!("jq not found");
    }

    // Check for gh if PR creation is requested
    if config.create_pr {
        if std::process::Command::new("which")
            .arg("gh")
            .stdout(std::process::Stdio::null())
            .status()?
            .success()
            == false
        {
            anyhow::bail!(
                "GitHub CLI (gh) is required for --create-pr. Install from https://cli.github.com/"
            );
        }
    }

    // Check for git
    if !git::is_git_repo()? {
        anyhow::bail!("Not a git repository. Ralphy requires a git repository to track changes.");
    }

    // Create progress.txt if missing
    if !std::path::Path::new("progress.txt").exists() {
        eprintln!(
            "{} progress.txt not found, creating it...",
            "[WARN]".yellow().bold()
        );
        tokio::fs::write("progress.txt", "").await?;
    }

    Ok(())
}

async fn run_sequential_loop(config: Config, prd_manager: Arc<PrdManager>) -> Result<()> {
    let mut iteration = 0;
    let mut total_input_tokens = 0;
    let mut total_output_tokens = 0;
    let mut total_cost = 0.0;
    let mut total_duration_ms = 0u64;

    loop {
        iteration += 1;

        // Check if we've hit max iterations
        if config.max_iterations > 0 && iteration > config.max_iterations {
            println!(
                "\n{} Reached max iterations ({})",
                "[WARN]".yellow().bold(),
                config.max_iterations
            );
            break;
        }

        // Get next task
        let task = match prd_manager.get_next_task().await? {
            Some(t) => t,
            None => {
                println!("\n{} All tasks complete!", "[SUCCESS]".green().bold());
                break;
            }
        };

        // Show task info
        let remaining = prd_manager.count_remaining().await?;
        let completed = prd_manager.count_completed().await?;

        println!("\n{}", "─".repeat(60).bright_black());
        println!("{} Task {}", ">>>".bright_cyan().bold(), iteration);
        println!(
            "    Completed: {} | Remaining: {}",
            completed.to_string().bright_green(),
            remaining.to_string().bright_yellow()
        );
        println!("{}", "─".repeat(60).bright_black());

        // Execute task with retries
        let mut retry_count = 0;
        let response = loop {
            match execute_task(&config, &task, iteration).await {
                Ok(resp) => break resp,
                Err(e) => {
                    retry_count += 1;
                    if retry_count >= config.max_retries {
                        eprintln!(
                            "{} Task failed after {} attempts: {}",
                            "[ERROR]".red().bold(),
                            config.max_retries,
                            e
                        );
                        // Continue to next task instead of failing entirely
                        break ai::AiResponse {
                            text: String::new(),
                            input_tokens: 0,
                            output_tokens: 0,
                            actual_cost: None,
                            duration_ms: None,
                        };
                    }
                    eprintln!(
                        "{} Attempt {}/{} failed: {}. Retrying in {}s...",
                        "[WARN]".yellow().bold(),
                        retry_count,
                        config.max_retries,
                        e,
                        config.retry_delay
                    );
                    sleep(Duration::from_secs(config.retry_delay)).await;
                }
            }
        };

        // Update totals
        total_input_tokens += response.input_tokens;
        total_output_tokens += response.output_tokens;
        if let Some(cost) = response.actual_cost {
            total_cost += cost;
        }
        if let Some(dur) = response.duration_ms {
            total_duration_ms += dur;
        }

        // Mark task complete
        prd_manager.mark_complete(&task).await?;

        // Show completion
        println!(
            "  {} Done │ {}",
            "✓".green().bold(),
            task.chars().take(50).collect::<String>()
        );

        if !response.text.is_empty() {
            println!("\n{}", response.text);
        }
    }

    // Show summary
    show_summary(
        iteration,
        total_input_tokens,
        total_output_tokens,
        total_cost,
        total_duration_ms,
        &config,
    );

    // Send notification
    if !config.no_notify {
        notifications::notify_done("Ralphy has completed all tasks!");
    }

    Ok(())
}

async fn run_parallel_loop(config: Config, prd_manager: Arc<PrdManager>) -> Result<()> {
    println!(
        "\n{} Running {} parallel agents (each in isolated worktree)...",
        "[INFO]".blue().bold(),
        config.max_parallel.to_string().bright_cyan().bold()
    );

    let all_tasks = prd_manager.get_tasks().await?;
    if all_tasks.is_empty() {
        println!("{} No tasks to run", "[INFO]".blue().bold());
        return Ok(());
    }

    println!(
        "{} Found {} tasks to process",
        "[INFO]".blue().bold(),
        all_tasks.len()
    );

    let mut total_input_tokens = 0;
    let mut total_output_tokens = 0;
    let mut iteration = 0;

    // Process tasks in batches
    for chunk in all_tasks.chunks(config.max_parallel) {
        let batch_num = iteration / config.max_parallel + 1;
        println!(
            "\n{} Batch {}: Spawning {} parallel agents",
            "━━━".bright_black(),
            batch_num,
            chunk.len()
        );

        let mut handles = vec![];

        for task in chunk {
            iteration += 1;
            let config_clone = config.clone();
            let task_clone = task.clone();
            let prd_manager_clone = prd_manager.clone();

            let handle = tokio::spawn(async move {
                let result = execute_task(&config_clone, &task_clone, iteration).await;
                (task_clone, result)
            });

            handles.push(handle);
        }

        // Wait for all parallel tasks
        let results = join_all(handles).await;

        // Process results
        for result in results {
            match result {
                Ok((task, Ok(response))) => {
                    total_input_tokens += response.input_tokens;
                    total_output_tokens += response.output_tokens;

                    // Mark complete
                    prd_manager.mark_complete(&task).await?;

                    println!(
                        "  {} Agent completed: {}",
                        "✓".green().bold(),
                        task.chars().take(50).collect::<String>()
                    );
                }
                Ok((task, Err(e))) => {
                    eprintln!(
                        "  {} Agent failed: {} - {}",
                        "✗".red().bold(),
                        task.chars().take(50).collect::<String>(),
                        e
                    );
                }
                Err(e) => {
                    eprintln!("  {} Task join error: {}", "✗".red().bold(), e);
                }
            }
        }

        if config.max_iterations > 0 && iteration >= config.max_iterations {
            println!(
                "\n{} Reached max iterations ({})",
                "[WARN]".yellow().bold(),
                config.max_iterations
            );
            break;
        }
    }

    show_summary(
        iteration,
        total_input_tokens,
        total_output_tokens,
        0.0,
        0,
        &config,
    );

    if !config.no_notify {
        notifications::notify_done("Ralphy has completed all tasks!");
    }

    Ok(())
}

async fn execute_task(config: &Config, task: &str, iteration: usize) -> Result<ai::AiResponse> {
    if config.dry_run {
        println!("{} DRY RUN - Would execute:", "[INFO]".blue().bold());
        let prompt = prompt::build_prompt(config, Some(task));
        println!("{}", prompt.bright_black());
        return Ok(ai::AiResponse {
            text: "Dry run".to_string(),
            input_tokens: 0,
            output_tokens: 0,
            actual_cost: None,
            duration_ms: None,
        });
    }

    // Create branch if needed
    if config.branch_per_task {
        git::create_task_branch(task, config.base_branch.as_deref())?;
    }

    // Build prompt
    let prompt = prompt::build_prompt(config, Some(task));

    // Execute AI
    let executor = ai::AiExecutor::new(config.ai_engine);

    // Start progress monitor
    let monitor_handle = if !config.parallel {
        Some(tokio::spawn(monitor::monitor_progress(
            task.to_string(),
            config.ai_engine,
        )))
    } else {
        None
    };

    let response = executor.execute(&prompt).await?;

    // Stop monitor
    if let Some(handle) = monitor_handle {
        handle.abort();
    }

    // Create PR if needed
    if config.create_pr && config.branch_per_task {
        git::create_pull_request(task, config.draft_pr)?;
    }

    Ok(response)
}

fn show_summary(
    iterations: usize,
    input_tokens: usize,
    output_tokens: usize,
    actual_cost: f64,
    duration_ms: u64,
    config: &Config,
) {
    println!("\n{}", "=".repeat(60).bright_black());
    println!(
        "{} PRD complete! Finished {} task(s).",
        "✓".green().bold(),
        iterations
    );
    println!("{}", "=".repeat(60).bright_black());
    println!("\n{} Cost Summary", ">>>".bright_cyan().bold());

    match config.ai_engine {
        cli::AiEngine::Cursor => {
            println!(
                "{}",
                "Token usage not available (Cursor CLI doesn't expose this data)".bright_black()
            );
            if duration_ms > 0 {
                let dur_sec = duration_ms / 1000;
                let dur_min = dur_sec / 60;
                let dur_sec_rem = dur_sec % 60;
                if dur_min > 0 {
                    println!("Total API time: {}m {}s", dur_min, dur_sec_rem);
                } else {
                    println!("Total API time: {}s", dur_sec);
                }
            }
        }
        _ => {
            println!("Input tokens:  {}", input_tokens);
            println!("Output tokens: {}", output_tokens);
            println!("Total tokens:  {}", input_tokens + output_tokens);

            if actual_cost > 0.0 {
                println!("Actual cost:   ${:.4}", actual_cost);
            } else {
                let est_cost = calculate_cost(input_tokens, output_tokens);
                println!("Est. cost:     ${:.4}", est_cost);
            }
        }
    }

    println!("{}", "=".repeat(60).bright_black());
}

fn calculate_cost(input_tokens: usize, output_tokens: usize) -> f64 {
    (input_tokens as f64 * 0.000003) + (output_tokens as f64 * 0.000015)
}
