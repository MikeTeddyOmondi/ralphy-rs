use anyhow::{Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum PrdSource {
    Markdown { path: PathBuf },
    Yaml { path: PathBuf },
    GitHub { repo: String, label: Option<String> },
}

impl PrdSource {
    pub fn display_name(&self) -> String {
        match self {
            PrdSource::Markdown { path } => path.display().to_string(),
            PrdSource::Yaml { path } => path.display().to_string(),
            PrdSource::GitHub { repo, label } => {
                if let Some(label) = label {
                    format!("{} (label: {})", repo, label)
                } else {
                    repo.clone()
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub title: String,
    pub completed: bool,
    #[serde(default)]
    pub parallel_group: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YamlTasks {
    pub tasks: Vec<Task>,
}

pub struct PrdManager {
    source: PrdSource,
}

impl PrdManager {
    pub fn new(source: PrdSource) -> Self {
        Self { source }
    }

    /// Get all incomplete tasks
    pub async fn get_tasks(&self) -> Result<Vec<String>> {
        match &self.source {
            PrdSource::Markdown { path } => self.get_markdown_tasks(path),
            PrdSource::Yaml { path } => self.get_yaml_tasks(path),
            PrdSource::GitHub { repo, label } => {
                self.get_github_tasks(repo, label.as_deref()).await
            }
        }
    }

    /// Get the next incomplete task
    pub async fn get_next_task(&self) -> Result<Option<String>> {
        let tasks = self.get_tasks().await?;
        Ok(tasks.into_iter().next())
    }

    /// Count remaining tasks
    pub async fn count_remaining(&self) -> Result<usize> {
        Ok(self.get_tasks().await?.len())
    }

    /// Count completed tasks
    pub async fn count_completed(&self) -> Result<usize> {
        match &self.source {
            PrdSource::Markdown { path } => self.count_markdown_completed(path),
            PrdSource::Yaml { path } => self.count_yaml_completed(path),
            PrdSource::GitHub { repo, label } => {
                self.count_github_completed(repo, label.as_deref()).await
            }
        }
    }

    /// Mark a task as complete
    pub async fn mark_complete(&self, task: &str) -> Result<()> {
        match &self.source {
            PrdSource::Markdown { path } => self.mark_markdown_complete(path, task),
            PrdSource::Yaml { path } => self.mark_yaml_complete(path, task),
            PrdSource::GitHub { repo, .. } => self.mark_github_complete(repo, task).await,
        }
    }

    /// Get tasks by parallel group (YAML only)
    pub async fn get_tasks_in_group(&self, group: usize) -> Result<Vec<String>> {
        match &self.source {
            PrdSource::Yaml { path } => {
                let content = fs::read_to_string(path)
                    .with_context(|| format!("Failed to read YAML file: {}", path.display()))?;
                let yaml_tasks: YamlTasks =
                    serde_yaml::from_str(&content).with_context(|| "Failed to parse YAML")?;

                Ok(yaml_tasks
                    .tasks
                    .into_iter()
                    .filter(|t| !t.completed && t.parallel_group == group)
                    .map(|t| t.title)
                    .collect())
            }
            _ => Ok(vec![]),
        }
    }

    // ============================================
    // MARKDOWN IMPLEMENTATION
    // ============================================

    fn get_markdown_tasks(&self, path: &PathBuf) -> Result<Vec<String>> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read PRD file: {}", path.display()))?;

        let re = Regex::new(r"^- \[ \] (.+)$").unwrap();
        let tasks: Vec<String> = content
            .lines()
            .filter_map(|line| {
                re.captures(line.trim())
                    .map(|cap| cap[1].trim().to_string())
            })
            .collect();

        Ok(tasks)
    }

    fn count_markdown_completed(&self, path: &PathBuf) -> Result<usize> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read PRD file: {}", path.display()))?;

        let re = Regex::new(r"^- \[x\]").unwrap();
        Ok(content
            .lines()
            .filter(|line| re.is_match(line.trim()))
            .count())
    }

    fn mark_markdown_complete(&self, path: &PathBuf, task: &str) -> Result<()> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read PRD file: {}", path.display()))?;

        // Escape special regex characters in task
        let escaped_task = regex::escape(task);
        let pattern = format!(r"^- \[ \] {}", escaped_task);
        let re = Regex::new(&pattern).unwrap();

        let new_content = content
            .lines()
            .map(|line| {
                if re.is_match(line.trim()) {
                    line.replace("- [ ]", "- [x]")
                } else {
                    line.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n");

        fs::write(path, new_content)
            .with_context(|| format!("Failed to write PRD file: {}", path.display()))?;

        Ok(())
    }

    // ============================================
    // YAML IMPLEMENTATION
    // ============================================

    fn get_yaml_tasks(&self, path: &PathBuf) -> Result<Vec<String>> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read YAML file: {}", path.display()))?;

        let yaml_tasks: YamlTasks =
            serde_yaml::from_str(&content).with_context(|| "Failed to parse YAML")?;

        Ok(yaml_tasks
            .tasks
            .into_iter()
            .filter(|t| !t.completed)
            .map(|t| t.title)
            .collect())
    }

    fn count_yaml_completed(&self, path: &PathBuf) -> Result<usize> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read YAML file: {}", path.display()))?;

        let yaml_tasks: YamlTasks =
            serde_yaml::from_str(&content).with_context(|| "Failed to parse YAML")?;

        Ok(yaml_tasks.tasks.into_iter().filter(|t| t.completed).count())
    }

    fn mark_yaml_complete(&self, path: &PathBuf, task: &str) -> Result<()> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read YAML file: {}", path.display()))?;

        let mut yaml_tasks: YamlTasks =
            serde_yaml::from_str(&content).with_context(|| "Failed to parse YAML")?;

        for t in &mut yaml_tasks.tasks {
            if t.title == task {
                t.completed = true;
                break;
            }
        }

        let new_content =
            serde_yaml::to_string(&yaml_tasks).with_context(|| "Failed to serialize YAML")?;

        fs::write(path, new_content)
            .with_context(|| format!("Failed to write YAML file: {}", path.display()))?;

        Ok(())
    }

    // ============================================
    // GITHUB IMPLEMENTATION
    // ============================================

    async fn get_github_tasks(&self, repo: &str, label: Option<&str>) -> Result<Vec<String>> {
        // This would use the GitHub CLI or API
        // For now, returning a placeholder
        // In a real implementation, you'd call:
        // gh issue list --repo {repo} --state open --json number,title --label {label}

        let mut cmd = tokio::process::Command::new("gh");
        cmd.arg("issue")
            .arg("list")
            .arg("--repo")
            .arg(repo)
            .arg("--state")
            .arg("open")
            .arg("--json")
            .arg("number,title");

        if let Some(label) = label {
            cmd.arg("--label").arg(label);
        }

        let output = cmd.output().await.context("Failed to execute gh command")?;

        if !output.status.success() {
            anyhow::bail!(
                "gh command failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        let stdout = String::from_utf8(output.stdout)?;
        let issues: Vec<serde_json::Value> = serde_json::from_str(&stdout)?;

        Ok(issues
            .into_iter()
            .filter_map(|issue| {
                let number = issue["number"].as_u64()?;
                let title = issue["title"].as_str()?;
                Some(format!("{}:{}", number, title))
            })
            .collect())
    }

    async fn count_github_completed(&self, repo: &str, label: Option<&str>) -> Result<usize> {
        let mut cmd = tokio::process::Command::new("gh");
        cmd.arg("issue")
            .arg("list")
            .arg("--repo")
            .arg(repo)
            .arg("--state")
            .arg("closed")
            .arg("--json")
            .arg("number");

        if let Some(label) = label {
            cmd.arg("--label").arg(label);
        }

        let output = cmd.output().await.context("Failed to execute gh command")?;

        if !output.status.success() {
            anyhow::bail!("gh command failed");
        }

        let stdout = String::from_utf8(output.stdout)?;
        let issues: Vec<serde_json::Value> = serde_json::from_str(&stdout)?;

        Ok(issues.len())
    }

    async fn mark_github_complete(&self, repo: &str, task: &str) -> Result<()> {
        // Extract issue number from "number:title" format
        let issue_num = task.split(':').next().context("Invalid task format")?;

        let output = tokio::process::Command::new("gh")
            .arg("issue")
            .arg("close")
            .arg(issue_num)
            .arg("--repo")
            .arg(repo)
            .output()
            .await
            .context("Failed to close GitHub issue")?;

        if !output.status.success() {
            anyhow::bail!(
                "Failed to close issue: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        Ok(())
    }
}
