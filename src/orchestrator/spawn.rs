use serde::{Deserialize, Serialize};

use super::session::SessionKey;

/// Spawn mode for subagents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpawnMode {
    /// One-shot execution: run task, announce result, clean up.
    Run,
    /// Persistent session: stays alive for follow-up interactions.
    Session,
}

impl Default for SpawnMode {
    fn default() -> Self {
        Self::Run
    }
}

/// Sandbox inheritance mode for spawned subagents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SandboxMode {
    /// Inherit parent's sandbox settings.
    Inherit,
    /// Require sandboxed execution.
    Require,
}

impl Default for SandboxMode {
    fn default() -> Self {
        Self::Inherit
    }
}

/// Parameters for spawning a subagent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnParams {
    /// Task description for the subagent.
    pub task: String,
    /// Optional human-readable label for logging/debugging.
    pub label: Option<String>,
    /// Target agent ID (from delegate agent configs). Defaults to requester's agent.
    pub agent_id: Option<String>,
    /// Model override (e.g. "anthropic/claude-sonnet-4-6").
    pub model: Option<String>,
    /// Spawn mode.
    #[serde(default)]
    pub mode: SpawnMode,
    /// Sandbox mode.
    #[serde(default)]
    pub sandbox: SandboxMode,
    /// Per-run timeout in seconds.
    pub timeout_secs: Option<u64>,
    /// Whether the spawner expects a completion message back.
    #[serde(default = "default_true")]
    pub expects_completion: bool,
    /// Post-completion cleanup strategy.
    #[serde(default)]
    pub cleanup: CleanupStrategy,
    /// Allowed tools for this subagent (empty = inherit parent's full set).
    #[serde(default)]
    pub allowed_tools: Vec<String>,
}

fn default_true() -> bool {
    true
}

/// Post-completion cleanup strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CleanupStrategy {
    /// Delete session transcript after completion.
    Delete,
    /// Keep session transcript for inspection.
    Keep,
}

impl Default for CleanupStrategy {
    fn default() -> Self {
        Self::Delete
    }
}

/// Result of a spawn attempt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnResult {
    /// Whether the spawn was accepted.
    pub status: SpawnStatus,
    /// Session key for the spawned child (if accepted).
    pub child_session_key: Option<SessionKey>,
    /// Run ID for idempotency tracking.
    pub run_id: Option<String>,
    /// Spawn mode applied.
    pub mode: Option<SpawnMode>,
    /// Error message (if rejected).
    pub error: Option<String>,
    /// Human-readable note about the spawn.
    pub note: Option<String>,
}

/// Spawn attempt status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpawnStatus {
    Accepted,
    Forbidden,
    Error,
}

impl SpawnResult {
    pub fn accepted(child_session_key: SessionKey, run_id: String, mode: SpawnMode) -> Self {
        Self {
            status: SpawnStatus::Accepted,
            child_session_key: Some(child_session_key),
            run_id: Some(run_id),
            mode: Some(mode),
            error: None,
            note: Some("Subagent will announce on completion.".into()),
        }
    }

    pub fn forbidden(reason: String) -> Self {
        Self {
            status: SpawnStatus::Forbidden,
            child_session_key: None,
            run_id: None,
            mode: None,
            error: Some(reason),
            note: None,
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            status: SpawnStatus::Error,
            child_session_key: None,
            run_id: None,
            mode: None,
            error: Some(message),
            note: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spawn_mode_default_is_run() {
        assert_eq!(SpawnMode::default(), SpawnMode::Run);
    }

    #[test]
    fn spawn_result_accepted() {
        let key = SessionKey::subagent("worker");
        let result = SpawnResult::accepted(key.clone(), "run-001".into(), SpawnMode::Run);
        assert_eq!(result.status, SpawnStatus::Accepted);
        assert_eq!(result.child_session_key.unwrap().agent_id, key.agent_id);
        assert!(result.error.is_none());
    }

    #[test]
    fn spawn_result_forbidden() {
        let result = SpawnResult::forbidden("policy denied".into());
        assert_eq!(result.status, SpawnStatus::Forbidden);
        assert!(result.child_session_key.is_none());
        assert_eq!(result.error.as_deref(), Some("policy denied"));
    }

    #[test]
    fn cleanup_strategy_default_is_delete() {
        assert_eq!(CleanupStrategy::default(), CleanupStrategy::Delete);
    }
}
