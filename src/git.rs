use anyhow::{Context, Result};
use std::process::Command;

pub fn is_git_repo() -> Result<bool> {
    let output = Command::new("git")
        .arg("rev-parse")
        .arg("--git-dir")
        .output()?;

    Ok(output.status.success())
}

pub fn create_task_branch(task: &str, base_branch: Option<&str>) -> Result<String> {
    let branch_name = format!("ralphy/{}", slugify(task));

    // Get base branch or current
    let base = match base_branch {
        Some(b) => b.to_string(),
        None => Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "main".to_string()),
    };

    // Stash changes if any
    Command::new("git")
        .args(["stash", "push", "-m", "ralphy-autostash"])
        .output()?;

    // Checkout base branch
    Command::new("git").arg("checkout").arg(&base).output()?;

    // Pull latest
    Command::new("git")
        .args(["pull", "origin", &base])
        .output()
        .ok();

    // Create and checkout new branch
    let status = Command::new("git")
        .args(["checkout", "-b", &branch_name])
        .status()?;

    if !status.success() {
        // Branch might exist, just checkout
        Command::new("git")
            .args(["checkout", &branch_name])
            .status()?;
    }

    // Pop stash if we stashed
    Command::new("git").args(["stash", "pop"]).output().ok();

    Ok(branch_name)
}

pub fn create_pull_request(task: &str, draft: bool) -> Result<String> {
    let current_branch = get_current_branch()?;

    // Push branch
    let push_status = Command::new("git")
        .args(["push", "-u", "origin", &current_branch])
        .status()?;

    if !push_status.success() {
        anyhow::bail!("Failed to push branch {}", current_branch);
    }

    // Create PR
    let mut cmd = Command::new("gh");
    cmd.args([
        "pr",
        "create",
        "--title",
        task,
        "--body",
        "Automated implementation by Ralphy",
    ]);

    if draft {
        cmd.arg("--draft");
    }

    let output = cmd.output()?;

    if !output.status.success() {
        anyhow::bail!(
            "Failed to create PR: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let pr_url = String::from_utf8(output.stdout)?;
    Ok(pr_url.trim().to_string())
}

fn get_current_branch() -> Result<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .context("Failed to get current branch")?;

    if !output.status.success() {
        anyhow::bail!("Failed to get current branch");
    }

    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}

fn slugify(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
        .chars()
        .take(50)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slugify() {
        assert_eq!(
            slugify("Create User Authentication"),
            "create-user-authentication"
        );
        assert_eq!(slugify("Add API Endpoints!"), "add-api-endpoints");
        assert_eq!(slugify("Fix bug in parser.rs"), "fix-bug-in-parser-rs");
    }
}
