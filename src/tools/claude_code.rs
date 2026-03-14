use super::traits::{Tool, ToolResult};
use async_trait::async_trait;
use serde_json::json;
use std::path::PathBuf;
use std::time::Duration;

/// Execution mode for Claude Code invocations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClaudeCodeMode {
    /// Plan mode: analyze and propose changes without executing.
    Plan,
    /// Implement mode: write code, create files, run tests.
    Implement,
    /// Review mode: review code for quality, security, and correctness.
    Review,
    /// Fix mode: diagnose and fix bugs or failing tests.
    Fix,
}

impl ClaudeCodeMode {
    fn as_str(&self) -> &str {
        match self {
            Self::Plan => "plan",
            Self::Implement => "implement",
            Self::Review => "review",
            Self::Fix => "fix",
        }
    }

    fn system_prefix(&self) -> &str {
        match self {
            Self::Plan => "You are in PLAN mode. Analyze the task and propose a detailed implementation plan. Do NOT write code yet. Output the plan as structured markdown.",
            Self::Implement => "You are in IMPLEMENT mode. Write production-quality code. Follow the project's existing patterns. Run tests after changes.",
            Self::Review => "You are in REVIEW mode. Review the specified code for: correctness, security vulnerabilities (OWASP top 10), performance issues, and adherence to project conventions. Output findings as structured markdown.",
            Self::Fix => "You are in FIX mode. Diagnose the issue, identify root cause, and apply the minimal fix. Run tests to verify the fix. Do not refactor unrelated code.",
        }
    }
}

/// Default timeout for Claude Code subprocess (5 minutes).
const DEFAULT_TIMEOUT_SECS: u64 = 300;

/// Tool that wraps the `claude` CLI for code generation, review, and fixing.
///
/// This is the primary bridge between ZeroClaw's orchestrator and Claude Code's
/// coding capabilities. Subagents in the Builder cluster use this tool to
/// implement features, review PRs, fix bugs, and plan architecture.
pub struct ClaudeCodeTool {
    /// Working directory for Claude Code invocations.
    workspace_dir: PathBuf,
    /// Timeout per invocation.
    timeout: Duration,
    /// Whether to use `--dangerously-skip-permissions` (only in sandboxed envs).
    skip_permissions: bool,
    /// Model override for cost-aware routing.
    /// When set, passes `--model <model>` to Claude Code CLI.
    /// Personas set this based on task complexity (e.g. Haiku for plan, Sonnet for implement).
    model_override: Option<String>,
}

impl ClaudeCodeTool {
    pub fn new(workspace_dir: PathBuf) -> Self {
        Self {
            workspace_dir,
            timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECS),
            skip_permissions: false,
            model_override: None,
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn with_skip_permissions(mut self, skip: bool) -> Self {
        self.skip_permissions = skip;
        self
    }

    /// Set a model override for cost-aware routing.
    /// e.g. `"claude-haiku-4-5"` for plan/review, `"claude-sonnet-4-6"` for implement.
    pub fn with_model(mut self, model: String) -> Self {
        self.model_override = Some(model);
        self
    }

    /// Build the command arguments for a Claude Code invocation.
    fn build_args(&self, mode: ClaudeCodeMode, task: &str, context: &str) -> Vec<String> {
        let mut args = vec!["--print".to_string()];

        if self.skip_permissions {
            args.push("--dangerously-skip-permissions".to_string());
        }

        // Cost-aware model routing: use persona's preferred model if set.
        if let Some(ref model) = self.model_override {
            args.push("--model".to_string());
            args.push(model.clone());
        }

        // Build the full prompt with mode prefix and optional context.
        let mode_prefix = mode.system_prefix();
        let full_prompt = if context.is_empty() {
            format!("{mode_prefix}\n\n## Task\n{task}")
        } else {
            format!("{mode_prefix}\n\n## Context\n{context}\n\n## Task\n{task}")
        };

        args.push("--prompt".to_string());
        args.push(full_prompt);

        args
    }

    /// Execute Claude Code as a subprocess and capture output.
    async fn run_claude(
        &self,
        mode: ClaudeCodeMode,
        task: &str,
        context: &str,
    ) -> Result<String, String> {
        let args = self.build_args(mode, task, context);

        let result = tokio::time::timeout(self.timeout, async {
            tokio::process::Command::new("claude")
                .args(&args)
                .current_dir(&self.workspace_dir)
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .output()
                .await
        })
        .await;

        match result {
            Ok(Ok(output)) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();

                if output.status.success() {
                    if stdout.trim().is_empty() {
                        Ok("[Claude Code completed with no output]".to_string())
                    } else {
                        Ok(stdout)
                    }
                } else {
                    let error_msg = if stderr.trim().is_empty() {
                        format!(
                            "Claude Code exited with status {}. Output: {}",
                            output.status,
                            if stdout.trim().is_empty() {
                                "(empty)"
                            } else {
                                stdout.trim()
                            }
                        )
                    } else {
                        format!(
                            "Claude Code error (status {}): {}",
                            output.status,
                            stderr.trim()
                        )
                    };
                    Err(error_msg)
                }
            }
            Ok(Err(e)) => {
                // Check if claude CLI exists.
                if e.kind() == std::io::ErrorKind::NotFound {
                    Err(
                        "Claude Code CLI ('claude') not found. Install it: npm install -g @anthropic-ai/claude-code"
                            .to_string(),
                    )
                } else {
                    Err(format!("Failed to spawn Claude Code: {e}"))
                }
            }
            Err(_) => Err(format!(
                "Claude Code timed out after {}s",
                self.timeout.as_secs()
            )),
        }
    }
}

#[async_trait]
impl Tool for ClaudeCodeTool {
    fn name(&self) -> &str {
        "claude_code"
    }

    fn description(&self) -> &str {
        "Invoke Claude Code to plan, implement, review, or fix code in the workspace. \
         Modes: 'plan' (analyze and propose), 'implement' (write code), \
         'review' (audit code quality/security), 'fix' (diagnose and repair bugs). \
         The tool runs Claude Code as a subprocess with full workspace access."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "additionalProperties": false,
            "properties": {
                "mode": {
                    "type": "string",
                    "enum": ["plan", "implement", "review", "fix"],
                    "description": "Execution mode: plan, implement, review, or fix"
                },
                "task": {
                    "type": "string",
                    "minLength": 1,
                    "description": "The task description or prompt for Claude Code"
                },
                "context": {
                    "type": "string",
                    "description": "Optional context (relevant code, error logs, PR description)"
                },
                "model": {
                    "type": "string",
                    "description": "Optional model override for cost-aware routing (e.g. 'claude-haiku-4-5' for light tasks)"
                }
            },
            "required": ["mode", "task"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        let mode_str = args
            .get("mode")
            .and_then(|v| v.as_str())
            .unwrap_or("implement");

        let mode = match mode_str {
            "plan" => ClaudeCodeMode::Plan,
            "implement" => ClaudeCodeMode::Implement,
            "review" => ClaudeCodeMode::Review,
            "fix" => ClaudeCodeMode::Fix,
            _ => {
                return Ok(ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some(format!(
                        "Invalid mode '{mode_str}'. Use: plan, implement, review, fix"
                    )),
                });
            }
        };

        let task = match args.get("task").and_then(|v| v.as_str()) {
            Some(t) if !t.trim().is_empty() => t.trim(),
            _ => {
                return Ok(ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some("'task' parameter is required and must not be empty".into()),
                });
            }
        };

        let context = args
            .get("context")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        match self.run_claude(mode, task, context).await {
            Ok(output) => Ok(ToolResult {
                success: true,
                output: format!(
                    "[Claude Code ({mode})] {output}",
                    mode = mode.as_str()
                ),
                error: None,
            }),
            Err(e) => Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(e),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_metadata() {
        let tool = ClaudeCodeTool::new(PathBuf::from("/tmp"));
        assert_eq!(tool.name(), "claude_code");
        assert!(!tool.description().is_empty());

        let schema = tool.parameters_schema();
        assert!(schema["properties"]["mode"].is_object());
        assert!(schema["properties"]["task"].is_object());
        assert!(schema["properties"]["context"].is_object());
    }

    #[test]
    fn build_args_plan_mode() {
        let tool = ClaudeCodeTool::new(PathBuf::from("/workspace"));
        let args = tool.build_args(ClaudeCodeMode::Plan, "design the API", "");
        assert!(args.contains(&"--print".to_string()));
        assert!(args.iter().any(|a| a.contains("PLAN mode")));
        assert!(args.iter().any(|a| a.contains("design the API")));
    }

    #[test]
    fn build_args_with_context() {
        let tool = ClaudeCodeTool::new(PathBuf::from("/workspace"));
        let args = tool.build_args(
            ClaudeCodeMode::Fix,
            "fix the auth bug",
            "Error: token expired",
        );
        assert!(args.iter().any(|a| a.contains("FIX mode")));
        assert!(args.iter().any(|a| a.contains("token expired")));
        assert!(args.iter().any(|a| a.contains("fix the auth bug")));
    }

    #[test]
    fn build_args_skip_permissions() {
        let tool = ClaudeCodeTool::new(PathBuf::from("/workspace")).with_skip_permissions(true);
        let args = tool.build_args(ClaudeCodeMode::Implement, "write tests", "");
        assert!(args.contains(&"--dangerously-skip-permissions".to_string()));
    }

    #[tokio::test]
    async fn execute_rejects_empty_task() {
        let tool = ClaudeCodeTool::new(PathBuf::from("/tmp"));
        let result = tool
            .execute(json!({"mode": "plan", "task": ""}))
            .await
            .unwrap();
        assert!(!result.success);
        assert!(result.error.unwrap().contains("required"));
    }

    #[tokio::test]
    async fn execute_rejects_invalid_mode() {
        let tool = ClaudeCodeTool::new(PathBuf::from("/tmp"));
        let result = tool
            .execute(json!({"mode": "destroy", "task": "test"}))
            .await
            .unwrap();
        assert!(!result.success);
        assert!(result.error.unwrap().contains("Invalid mode"));
    }
}
