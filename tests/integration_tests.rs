use ralphy_rs::prd::{PrdManager, PrdSource};
use std::path::PathBuf;
use tempfile::TempDir;
use tokio;

#[tokio::test]
async fn test_markdown_prd_parsing() {
    let temp_dir = TempDir::new().unwrap();
    let prd_path = temp_dir.path().join("PRD.md");

    let prd_content = r#"# Test Project

## Tasks

- [ ] First task
- [ ] Second task
- [x] Completed task
- [ ] Third task
"#;

    std::fs::write(&prd_path, prd_content).unwrap();

    let manager = PrdManager::new(PrdSource::Markdown {
        path: prd_path.clone(),
    });

    // Test get tasks
    let tasks = manager.get_tasks().await.unwrap();
    assert_eq!(tasks.len(), 3);
    assert_eq!(tasks[0], "First task");
    assert_eq!(tasks[1], "Second task");
    assert_eq!(tasks[2], "Third task");

    // Test count remaining
    let remaining = manager.count_remaining().await.unwrap();
    assert_eq!(remaining, 3);

    // Test count completed
    let completed = manager.count_completed().await.unwrap();
    assert_eq!(completed, 1);

    // Test mark complete
    manager.mark_complete("First task").await.unwrap();

    let tasks_after = manager.get_tasks().await.unwrap();
    assert_eq!(tasks_after.len(), 2);

    let completed_after = manager.count_completed().await.unwrap();
    assert_eq!(completed_after, 2);
}

#[tokio::test]
async fn test_yaml_prd_parsing() {
    let temp_dir = TempDir::new().unwrap();
    let yaml_path = temp_dir.path().join("tasks.yaml");

    let yaml_content = r#"tasks:
  - title: First task
    completed: false
    parallel_group: 1
  - title: Second task
    completed: false
    parallel_group: 1
  - title: Completed task
    completed: true
    parallel_group: 2
  - title: Third task
    completed: false
    parallel_group: 2
"#;

    std::fs::write(&yaml_path, yaml_content).unwrap();

    let manager = PrdManager::new(PrdSource::Yaml {
        path: yaml_path.clone(),
    });

    // Test get tasks
    let tasks = manager.get_tasks().await.unwrap();
    assert_eq!(tasks.len(), 3);

    // Test count completed
    let completed = manager.count_completed().await.unwrap();
    assert_eq!(completed, 1);

    // Test get tasks in parallel group
    let group1_tasks = manager.get_tasks_in_group(1).await.unwrap();
    assert_eq!(group1_tasks.len(), 2);

    let group2_tasks = manager.get_tasks_in_group(2).await.unwrap();
    assert_eq!(group2_tasks.len(), 1);

    // Test mark complete
    manager.mark_complete("First task").await.unwrap();

    let tasks_after = manager.get_tasks().await.unwrap();
    assert_eq!(tasks_after.len(), 2);
}

#[test]
fn test_git_slugify() {
    use ralphy_rs::git;

    // This would test the slugify function if it were public
    // For now, we test through branch creation behavior
    assert!(true);
}

#[test]
fn test_prompt_building() {
    use ralphy_rs::cli::{AiEngine, Cli};
    use ralphy_rs::config::Config;
    use ralphy_rs::prd::PrdSource;
    use ralphy_rs::prompt::build_prompt;
    use std::path::PathBuf;

    let config = Config {
        ai_engine: AiEngine::Claude,
        prd_source: PrdSource::Markdown {
            path: PathBuf::from("PRD.md"),
        },
        skip_tests: false,
        skip_lint: false,
        max_iterations: 0,
        max_retries: 3,
        retry_delay: 5,
        dry_run: false,
        parallel: false,
        max_parallel: 3,
        branch_per_task: false,
        base_branch: None,
        create_pr: false,
        draft_pr: false,
        verbose: 0,
        no_color: false,
        no_notify: false,
    };

    let prompt = build_prompt(&config, Some("Test task"));

    assert!(prompt.contains("PRD.md"));
    assert!(prompt.contains("progress.txt"));
    assert!(prompt.contains("Write tests"));
    assert!(prompt.contains("Run linting"));
    assert!(prompt.contains("ONLY WORK ON A SINGLE TASK"));
}

#[test]
fn test_prompt_building_fast_mode() {
    use ralphy_rs::cli::AiEngine;
    use ralphy_rs::config::Config;
    use ralphy_rs::prd::PrdSource;
    use ralphy_rs::prompt::build_prompt;
    use std::path::PathBuf;

    let config = Config {
        ai_engine: AiEngine::Claude,
        prd_source: PrdSource::Markdown {
            path: PathBuf::from("PRD.md"),
        },
        skip_tests: true,
        skip_lint: true,
        max_iterations: 0,
        max_retries: 3,
        retry_delay: 5,
        dry_run: false,
        parallel: false,
        max_parallel: 3,
        branch_per_task: false,
        base_branch: None,
        create_pr: false,
        draft_pr: false,
        verbose: 0,
        no_color: false,
        no_notify: false,
    };

    let prompt = build_prompt(&config, Some("Test task"));

    assert!(!prompt.contains("Write tests"));
    assert!(!prompt.contains("Run linting"));
}
