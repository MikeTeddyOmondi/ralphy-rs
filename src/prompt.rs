use crate::config::Config;
use crate::prd::PrdSource;

pub fn build_prompt(config: &Config, task_override: Option<&str>) -> String {
    let mut prompt = String::new();

    // Add context based on PRD source
    match &config.prd_source {
        PrdSource::Markdown { path } => {
            prompt.push_str(&format!("@{} @progress.txt\n", path.display()));
        }
        PrdSource::Yaml { path } => {
            prompt.push_str(&format!("@{} @progress.txt\n", path.display()));
        }
        PrdSource::GitHub { repo, .. } => {
            if let Some(task) = task_override {
                prompt.push_str(&format!("Task from GitHub Issue: {}\n\n", task));
                prompt.push_str("@progress.txt\n");
            }
        }
    }

    prompt.push_str("1. Find the highest-priority incomplete task and implement it.\n");

    let mut step = 2;

    if !config.skip_tests {
        prompt.push_str(&format!("{}. Write tests for the feature.\n", step));
        step += 1;
        prompt.push_str(&format!(
            "{}. Run tests and ensure they pass before proceeding.\n",
            step
        ));
        step += 1;
    }

    if !config.skip_lint {
        prompt.push_str(&format!(
            "{}. Run linting and ensure it passes before proceeding.\n",
            step
        ));
        step += 1;
    }

    // Adjust completion step based on PRD source
    match &config.prd_source {
        PrdSource::Markdown { .. } => {
            prompt.push_str(&format!(
                "{}. Update the PRD to mark the task as complete (change '- [ ]' to '- [x]').\n",
                step
            ));
        }
        PrdSource::Yaml { path } => {
            prompt.push_str(&format!(
                "{}. Update {} to mark the task as completed (set completed: true).\n",
                step,
                path.display()
            ));
        }
        PrdSource::GitHub { .. } => {
            prompt.push_str(&format!(
                "{}. The task will be marked complete automatically. Just note the completion in progress.txt.\n",
                step
            ));
        }
    }

    step += 1;

    prompt.push_str(&format!(
        "{}. Append your progress to progress.txt.\n",
        step
    ));
    step += 1;

    if !config.skip_commits {
        prompt.push_str(&format!(
            "{}. Commit your changes with a descriptive message.\n",
            step
        ));
        step += 1;
    }

    prompt.push_str("\nONLY WORK ON A SINGLE TASK.");

    if !config.skip_tests {
        prompt.push_str(" Do not proceed if tests fail.");
    }
    if !config.skip_lint {
        prompt.push_str(" Do not proceed if linting fails.");
    }

    prompt
        .push_str("\n\nIf ALL tasks in the PRD are complete, output <promise>COMPLETE</promise>.");

    prompt
}
