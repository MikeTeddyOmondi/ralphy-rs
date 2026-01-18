use crate::cli::AiEngine;
use anyhow::{Context, Result};
use serde_json::Value;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

#[derive(Debug, Clone)]
pub struct AiResponse {
    pub text: String,
    pub input_tokens: usize,
    pub output_tokens: usize,
    pub actual_cost: Option<f64>,
    pub duration_ms: Option<u64>,
}

pub struct AiExecutor {
    engine: AiEngine,
}

impl AiExecutor {
    pub fn new(engine: AiEngine) -> Self {
        Self { engine }
    }

    pub async fn execute(&self, prompt: &str) -> Result<AiResponse> {
        match self.engine {
            AiEngine::Claude => self.execute_claude(prompt).await,
            AiEngine::OpenCode => self.execute_opencode(prompt).await,
            AiEngine::Cursor => self.execute_cursor(prompt).await,
            AiEngine::Codex => self.execute_codex(prompt).await,
            AiEngine::Qwen => self.execute_qwen(prompt).await,
        }
    }

    async fn execute_claude(&self, prompt: &str) -> Result<AiResponse> {
        let mut child = Command::new("claude")
            .arg("--dangerously-skip-permissions")
            .arg("--verbose")
            .arg("--output-format")
            .arg("stream-json")
            .arg("-p")
            .arg(prompt)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("Failed to spawn claude command")?;

        let stdout = child.stdout.take().context("Failed to capture stdout")?;
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();

        let mut response_text = String::new();
        let mut input_tokens = 0;
        let mut output_tokens = 0;

        while let Some(line) = lines.next_line().await? {
            if let Ok(json) = serde_json::from_str::<Value>(&line) {
                // Parse stream-json format
                if let Some(msg_type) = json["type"].as_str() {
                    match msg_type {
                        "result" => {
                            if let Some(result) = json["result"].as_str() {
                                response_text = result.to_string();
                            }
                            if let Some(usage) = json["usage"].as_object() {
                                input_tokens = usage["input_tokens"].as_u64().unwrap_or(0) as usize;
                                output_tokens =
                                    usage["output_tokens"].as_u64().unwrap_or(0) as usize;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        let status = child.wait().await?;
        if !status.success() {
            anyhow::bail!("Claude command failed with status: {}", status);
        }

        Ok(AiResponse {
            text: response_text,
            input_tokens,
            output_tokens,
            actual_cost: None,
            duration_ms: None,
        })
    }

    async fn execute_opencode(&self, prompt: &str) -> Result<AiResponse> {
        let mut child = Command::new("opencode")
            .arg("run")
            .arg("--format")
            .arg("json")
            .arg(prompt)
            .env("OPENCODE_PERMISSION", r#"{"*":"allow"}"#)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("Failed to spawn opencode command")?;

        let stdout = child.stdout.take().context("Failed to capture stdout")?;
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();

        let mut response_text = String::new();
        let mut input_tokens = 0;
        let mut output_tokens = 0;
        let mut actual_cost = None;

        while let Some(line) = lines.next_line().await? {
            if let Ok(json) = serde_json::from_str::<Value>(&line) {
                if let Some(msg_type) = json["type"].as_str() {
                    match msg_type {
                        "text" => {
                            if let Some(text) = json["part"]["text"].as_str() {
                                response_text.push_str(text);
                            }
                        }
                        "step_finish" => {
                            if let Some(tokens) = json["part"]["tokens"].as_object() {
                                input_tokens = tokens["input"].as_u64().unwrap_or(0) as usize;
                                output_tokens = tokens["output"].as_u64().unwrap_or(0) as usize;
                            }
                            if let Some(cost) = json["part"]["cost"].as_f64() {
                                actual_cost = Some(cost);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        let status = child.wait().await?;
        if !status.success() {
            anyhow::bail!("OpenCode command failed with status: {}", status);
        }

        Ok(AiResponse {
            text: if response_text.is_empty() {
                "Task completed".to_string()
            } else {
                response_text
            },
            input_tokens,
            output_tokens,
            actual_cost,
            duration_ms: None,
        })
    }

    async fn execute_cursor(&self, prompt: &str) -> Result<AiResponse> {
        let mut child = Command::new("agent")
            .arg("--print")
            .arg("--force")
            .arg("--output-format")
            .arg("stream-json")
            .arg(prompt)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("Failed to spawn agent command")?;

        let stdout = child.stdout.take().context("Failed to capture stdout")?;
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();

        let mut response_text = String::new();
        let mut duration_ms = None;

        while let Some(line) = lines.next_line().await? {
            if let Ok(json) = serde_json::from_str::<Value>(&line) {
                if let Some(msg_type) = json["type"].as_str() {
                    match msg_type {
                        "result" => {
                            if let Some(result) = json["result"].as_str() {
                                response_text = result.to_string();
                            }
                            if let Some(dur) = json["duration_ms"].as_u64() {
                                duration_ms = Some(dur);
                            }
                        }
                        "assistant" => {
                            if response_text.is_empty() || response_text == "Task completed" {
                                if let Some(content) = json["message"]["content"].as_array() {
                                    if let Some(first) = content.first() {
                                        if let Some(text) = first["text"].as_str() {
                                            response_text = text.to_string();
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        let status = child.wait().await?;
        if !status.success() {
            anyhow::bail!("Cursor agent command failed with status: {}", status);
        }

        Ok(AiResponse {
            text: if response_text.is_empty() {
                "Task completed".to_string()
            } else {
                response_text
            },
            input_tokens: 0,
            output_tokens: 0,
            actual_cost: None,
            duration_ms,
        })
    }

    async fn execute_codex(&self, prompt: &str) -> Result<AiResponse> {
        use tempfile::NamedTempFile;

        let temp_file = NamedTempFile::new()?;
        let temp_path = temp_file.path().to_path_buf();

        let mut child = Command::new("codex")
            .arg("exec")
            .arg("--full-auto")
            .arg("--json")
            .arg("--output-last-message")
            .arg(&temp_path)
            .arg(prompt)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("Failed to spawn codex command")?;

        let status = child.wait().await?;
        if !status.success() {
            anyhow::bail!("Codex command failed with status: {}", status);
        }

        let response_text = tokio::fs::read_to_string(&temp_path)
            .await
            .unwrap_or_else(|_| String::new());

        Ok(AiResponse {
            text: response_text,
            input_tokens: 0,
            output_tokens: 0,
            actual_cost: None,
            duration_ms: None,
        })
    }

    async fn execute_qwen(&self, prompt: &str) -> Result<AiResponse> {
        let mut child = Command::new("qwen")
            .arg("--output-format")
            .arg("stream-json")
            .arg("--approval-mode")
            .arg("yolo")
            .arg("-p")
            .arg(prompt)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("Failed to spawn qwen command")?;

        let stdout = child.stdout.take().context("Failed to capture stdout")?;
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();

        let mut response_text = String::new();
        let mut input_tokens = 0;
        let mut output_tokens = 0;

        while let Some(line) = lines.next_line().await? {
            if let Ok(json) = serde_json::from_str::<Value>(&line) {
                if let Some(msg_type) = json["type"].as_str() {
                    if msg_type == "result" {
                        if let Some(result) = json["result"].as_str() {
                            response_text = result.to_string();
                        }
                        if let Some(usage) = json["usage"].as_object() {
                            input_tokens = usage["input_tokens"].as_u64().unwrap_or(0) as usize;
                            output_tokens = usage["output_tokens"].as_u64().unwrap_or(0) as usize;
                        }
                    }
                }
            }
        }

        let status = child.wait().await?;
        if !status.success() {
            anyhow::bail!("Qwen command failed with status: {}", status);
        }

        Ok(AiResponse {
            text: if response_text.is_empty() {
                "Task completed".to_string()
            } else {
                response_text
            },
            input_tokens,
            output_tokens,
            actual_cost: None,
            duration_ms: None,
        })
    }
}

pub fn check_ai_availability(engine: AiEngine) -> Result<()> {
    let cmd_name = match engine {
        AiEngine::Claude => "claude",
        AiEngine::OpenCode => "opencode",
        AiEngine::Cursor => "agent",
        AiEngine::Codex => "codex",
        AiEngine::Qwen => "qwen",
    };

    let status = std::process::Command::new("which")
        .arg(cmd_name)
        .stdout(Stdio::null())
        .status()?;

    if !status.success() {
        anyhow::bail!(
            "{} CLI not found. Please install {} first.",
            engine,
            match engine {
                AiEngine::Claude => "Claude Code from https://github.com/anthropics/claude-code",
                AiEngine::OpenCode => "OpenCode from https://opencode.ai/docs/",
                AiEngine::Cursor => "Cursor and ensure 'agent' is in your PATH",
                AiEngine::Codex => "Codex CLI",
                AiEngine::Qwen => "Qwen-Code",
            }
        );
    }

    Ok(())
}
