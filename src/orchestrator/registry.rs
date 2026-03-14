use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

use super::session::SessionKey;
use super::spawn::{CleanupStrategy, SpawnMode};

/// Outcome of a subagent run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunOutcome {
    /// Whether the run succeeded.
    pub success: bool,
    /// Summary text from the subagent.
    pub summary: Option<String>,
    /// Error message if failed.
    pub error: Option<String>,
}

/// Reason the run ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EndReason {
    /// Completed normally.
    Completed,
    /// Timed out.
    TimedOut,
    /// Killed by parent or system.
    Killed,
    /// Failed with error.
    Failed,
}

/// Record of a single subagent run, tracked in the registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubagentRunRecord {
    /// Unique run identifier.
    pub run_id: String,
    /// Session key for the child.
    pub child_session_key: SessionKey,
    /// Session key of the requester (parent).
    pub requester_session_key: SessionKey,
    /// Task description.
    pub task: String,
    /// Optional label.
    pub label: Option<String>,
    /// Model used.
    pub model: Option<String>,
    /// Spawn mode.
    pub mode: SpawnMode,
    /// Cleanup strategy.
    pub cleanup: CleanupStrategy,
    /// Timeout in seconds.
    pub timeout_secs: Option<u64>,
    /// Depth in the spawn chain (0 = top-level).
    pub depth: u32,

    // Lifecycle timestamps (as seconds since process start).
    pub created_at: f64,
    pub started_at: Option<f64>,
    pub ended_at: Option<f64>,

    /// Run outcome (set on completion).
    pub outcome: Option<RunOutcome>,
    /// Why the run ended.
    pub end_reason: Option<EndReason>,

    /// Whether the announce delivery succeeded.
    pub announced: bool,
    /// Number of announce retry attempts.
    pub announce_retries: u32,
    /// Frozen result text for announce delivery.
    pub frozen_result: Option<String>,
}

/// Thread-safe in-memory subagent registry.
///
/// Tracks all active and recently completed subagent runs.
/// Inspired by OpenClaw's subagent-registry pattern.
#[derive(Clone)]
pub struct SubagentRegistry {
    inner: Arc<RwLock<RegistryInner>>,
    epoch: Instant,
}

struct RegistryInner {
    runs: HashMap<String, SubagentRunRecord>,
    /// Map from parent session key string to list of child run IDs.
    children: HashMap<String, Vec<String>>,
    /// Maximum completed runs to retain before pruning.
    max_completed: usize,
}

impl SubagentRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self::with_capacity(100)
    }

    /// Create a registry with a specific completed-run retention limit.
    pub fn with_capacity(max_completed: usize) -> Self {
        Self {
            inner: Arc::new(RwLock::new(RegistryInner {
                runs: HashMap::new(),
                children: HashMap::new(),
                max_completed,
            })),
            epoch: Instant::now(),
        }
    }

    /// Register a new subagent run. Returns the run ID.
    pub async fn register(
        &self,
        child_session_key: SessionKey,
        requester_session_key: SessionKey,
        task: String,
        label: Option<String>,
        model: Option<String>,
        mode: SpawnMode,
        cleanup: CleanupStrategy,
        timeout_secs: Option<u64>,
        depth: u32,
    ) -> String {
        let run_id = format!("run-{}", super::session::SessionKey::subagent("_").instance_id);
        let now = self.epoch.elapsed().as_secs_f64();

        let record = SubagentRunRecord {
            run_id: run_id.clone(),
            child_session_key,
            requester_session_key: requester_session_key.clone(),
            task,
            label,
            model,
            mode,
            cleanup,
            timeout_secs,
            depth,
            created_at: now,
            started_at: None,
            ended_at: None,
            outcome: None,
            end_reason: None,
            announced: false,
            announce_retries: 0,
            frozen_result: None,
        };

        let mut inner = self.inner.write().await;
        let parent_key = requester_session_key.to_key();
        inner
            .children
            .entry(parent_key)
            .or_default()
            .push(run_id.clone());
        inner.runs.insert(run_id.clone(), record);

        run_id
    }

    /// Mark a run as started.
    pub async fn mark_started(&self, run_id: &str) {
        let mut inner = self.inner.write().await;
        if let Some(record) = inner.runs.get_mut(run_id) {
            record.started_at = Some(self.epoch.elapsed().as_secs_f64());
        }
    }

    /// Mark a run as ended with an outcome.
    pub async fn mark_ended(
        &self,
        run_id: &str,
        outcome: RunOutcome,
        reason: EndReason,
        frozen_result: Option<String>,
    ) {
        let mut inner = self.inner.write().await;
        if let Some(record) = inner.runs.get_mut(run_id) {
            record.ended_at = Some(self.epoch.elapsed().as_secs_f64());
            record.outcome = Some(outcome);
            record.end_reason = Some(reason);
            record.frozen_result = frozen_result;
        }
    }

    /// Mark a run's announce as delivered.
    pub async fn mark_announced(&self, run_id: &str) {
        let mut inner = self.inner.write().await;
        if let Some(record) = inner.runs.get_mut(run_id) {
            record.announced = true;
        }
    }

    /// Increment announce retry count.
    pub async fn increment_announce_retries(&self, run_id: &str) {
        let mut inner = self.inner.write().await;
        if let Some(record) = inner.runs.get_mut(run_id) {
            record.announce_retries += 1;
        }
    }

    /// Get a snapshot of a run record.
    pub async fn get(&self, run_id: &str) -> Option<SubagentRunRecord> {
        let inner = self.inner.read().await;
        inner.runs.get(run_id).cloned()
    }

    /// List all runs for a given parent session.
    pub async fn list_for_parent(&self, parent_session_key: &SessionKey) -> Vec<SubagentRunRecord> {
        let inner = self.inner.read().await;
        let key = parent_session_key.to_key();
        inner
            .children
            .get(&key)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| inner.runs.get(id).cloned())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Count active (not yet ended) runs for a parent session.
    pub async fn active_count_for_parent(&self, parent_session_key: &SessionKey) -> usize {
        let inner = self.inner.read().await;
        let key = parent_session_key.to_key();
        inner
            .children
            .get(&key)
            .map(|ids| {
                ids.iter()
                    .filter(|id| {
                        inner
                            .runs
                            .get(*id)
                            .map_or(false, |r| r.ended_at.is_none())
                    })
                    .count()
            })
            .unwrap_or(0)
    }

    /// List runs pending announce delivery.
    pub async fn pending_announces(&self) -> Vec<SubagentRunRecord> {
        let inner = self.inner.read().await;
        inner
            .runs
            .values()
            .filter(|r| r.ended_at.is_some() && !r.announced)
            .cloned()
            .collect()
    }

    /// Prune old completed+announced runs beyond retention limit.
    pub async fn prune(&self) {
        let mut inner = self.inner.write().await;
        let mut completed: Vec<(String, f64)> = inner
            .runs
            .iter()
            .filter(|(_, r)| r.ended_at.is_some() && r.announced)
            .map(|(id, r)| (id.clone(), r.ended_at.unwrap_or(0.0)))
            .collect();

        if completed.len() <= inner.max_completed {
            return;
        }

        completed.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        let to_remove = completed.len() - inner.max_completed;
        for (id, _) in completed.into_iter().take(to_remove) {
            if let Some(record) = inner.runs.remove(&id) {
                let parent_key = record.requester_session_key.to_key();
                if let Some(children) = inner.children.get_mut(&parent_key) {
                    children.retain(|cid| cid != &id);
                }
            }
        }
    }

    /// Total number of tracked runs (active + completed).
    pub async fn total_count(&self) -> usize {
        let inner = self.inner.read().await;
        inner.runs.len()
    }
}

impl Default for SubagentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::orchestrator::session::{SessionKey, SessionType};

    #[tokio::test]
    async fn register_and_retrieve() {
        let registry = SubagentRegistry::new();
        let parent = SessionKey::main("orchestrator");
        let child = SessionKey::subagent("worker");

        let run_id = registry
            .register(
                child.clone(),
                parent.clone(),
                "do research".into(),
                Some("research-task".into()),
                None,
                SpawnMode::Run,
                CleanupStrategy::Delete,
                Some(120),
                0,
            )
            .await;

        let record = registry.get(&run_id).await.unwrap();
        assert_eq!(record.task, "do research");
        assert_eq!(record.child_session_key.agent_id, "worker");
        assert_eq!(record.depth, 0);
        assert!(record.ended_at.is_none());
    }

    #[tokio::test]
    async fn lifecycle_tracking() {
        let registry = SubagentRegistry::new();
        let parent = SessionKey::main("main");
        let child = SessionKey::subagent("sub");

        let run_id = registry
            .register(
                child,
                parent,
                "task".into(),
                None,
                None,
                SpawnMode::Run,
                CleanupStrategy::Delete,
                None,
                0,
            )
            .await;

        registry.mark_started(&run_id).await;
        let record = registry.get(&run_id).await.unwrap();
        assert!(record.started_at.is_some());

        registry
            .mark_ended(
                &run_id,
                RunOutcome {
                    success: true,
                    summary: Some("done".into()),
                    error: None,
                },
                EndReason::Completed,
                Some("result text".into()),
            )
            .await;

        let record = registry.get(&run_id).await.unwrap();
        assert!(record.ended_at.is_some());
        assert!(record.outcome.as_ref().unwrap().success);
        assert_eq!(record.frozen_result.as_deref(), Some("result text"));
    }

    #[tokio::test]
    async fn active_count_for_parent() {
        let registry = SubagentRegistry::new();
        let parent = SessionKey::main("parent");

        let id1 = registry
            .register(
                SessionKey::subagent("a"),
                parent.clone(),
                "t1".into(),
                None,
                None,
                SpawnMode::Run,
                CleanupStrategy::Delete,
                None,
                0,
            )
            .await;

        let _id2 = registry
            .register(
                SessionKey::subagent("b"),
                parent.clone(),
                "t2".into(),
                None,
                None,
                SpawnMode::Run,
                CleanupStrategy::Delete,
                None,
                0,
            )
            .await;

        assert_eq!(registry.active_count_for_parent(&parent).await, 2);

        registry
            .mark_ended(
                &id1,
                RunOutcome {
                    success: true,
                    summary: None,
                    error: None,
                },
                EndReason::Completed,
                None,
            )
            .await;

        assert_eq!(registry.active_count_for_parent(&parent).await, 1);
    }

    #[tokio::test]
    async fn pending_announces() {
        let registry = SubagentRegistry::new();
        let parent = SessionKey::main("p");
        let run_id = registry
            .register(
                SessionKey::subagent("c"),
                parent,
                "t".into(),
                None,
                None,
                SpawnMode::Run,
                CleanupStrategy::Delete,
                None,
                0,
            )
            .await;

        assert!(registry.pending_announces().await.is_empty());

        registry
            .mark_ended(
                &run_id,
                RunOutcome {
                    success: true,
                    summary: None,
                    error: None,
                },
                EndReason::Completed,
                None,
            )
            .await;

        assert_eq!(registry.pending_announces().await.len(), 1);

        registry.mark_announced(&run_id).await;
        assert!(registry.pending_announces().await.is_empty());
    }

    #[tokio::test]
    async fn prune_removes_old_completed() {
        let registry = SubagentRegistry::with_capacity(1);
        let parent = SessionKey::main("p");

        for i in 0..3 {
            let run_id = registry
                .register(
                    SessionKey::subagent(&format!("c{i}")),
                    parent.clone(),
                    format!("task {i}"),
                    None,
                    None,
                    SpawnMode::Run,
                    CleanupStrategy::Delete,
                    None,
                    0,
                )
                .await;
            registry
                .mark_ended(
                    &run_id,
                    RunOutcome {
                        success: true,
                        summary: None,
                        error: None,
                    },
                    EndReason::Completed,
                    None,
                )
                .await;
            registry.mark_announced(&run_id).await;
        }

        assert_eq!(registry.total_count().await, 3);
        registry.prune().await;
        assert_eq!(registry.total_count().await, 1);
    }
}
