use async_trait::async_trait;

use super::depth::DepthConfig;
use super::policy::SpawnPolicy;
use super::registry::SubagentRegistry;
use super::session::SessionKey;
use super::spawn::{SpawnParams, SpawnResult};

/// Core orchestrator trait for multi-agent coordination.
///
/// The orchestrator manages the lifecycle of subagent spawning,
/// execution, and result delivery. It owns the registry, enforces
/// policy/depth constraints, and coordinates the announce system.
#[async_trait]
pub trait Orchestrator: Send + Sync {
    /// Spawn a new subagent with the given parameters.
    ///
    /// The orchestrator validates policy, depth, and concurrency limits
    /// before creating the child session and registering the run.
    async fn spawn(
        &self,
        requester: &SessionKey,
        params: SpawnParams,
        current_depth: u32,
    ) -> SpawnResult;

    /// Kill an active subagent run.
    async fn kill(&self, run_id: &str) -> Result<(), String>;

    /// List active subagent runs for a parent session.
    async fn list_children(
        &self,
        parent: &SessionKey,
    ) -> Vec<super::registry::SubagentRunRecord>;

    /// Get the subagent registry (for direct queries).
    fn registry(&self) -> &SubagentRegistry;

    /// Get the depth configuration.
    fn depth_config(&self) -> &DepthConfig;

    /// Get the spawn policy.
    fn spawn_policy(&self) -> &SpawnPolicy;
}

/// Default orchestrator implementation.
pub struct DefaultOrchestrator {
    registry: SubagentRegistry,
    depth_config: DepthConfig,
    spawn_policy: SpawnPolicy,
}

impl DefaultOrchestrator {
    pub fn new() -> Self {
        Self {
            registry: SubagentRegistry::new(),
            depth_config: DepthConfig::default(),
            spawn_policy: SpawnPolicy::default(),
        }
    }

    pub fn with_config(depth_config: DepthConfig, spawn_policy: SpawnPolicy) -> Self {
        Self {
            registry: SubagentRegistry::new(),
            depth_config,
            spawn_policy,
        }
    }
}

impl Default for DefaultOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Orchestrator for DefaultOrchestrator {
    async fn spawn(
        &self,
        requester: &SessionKey,
        params: SpawnParams,
        current_depth: u32,
    ) -> SpawnResult {
        // 1. Check policy.
        let agent_id = params
            .agent_id
            .as_deref()
            .unwrap_or(&requester.agent_id);
        if let Err(e) = super::policy::check_spawn_policy(&self.spawn_policy, agent_id, false) {
            return SpawnResult::forbidden(e);
        }

        // 2. Check depth.
        if let Err(e) = super::depth::check_depth(current_depth, &self.depth_config) {
            return SpawnResult::forbidden(e);
        }

        // 3. Check concurrency.
        let active = self.registry.active_count_for_parent(requester).await;
        if let Err(e) = super::depth::check_concurrency(active, &self.depth_config) {
            return SpawnResult::forbidden(e);
        }

        // 4. Create child session and register run.
        let child_key = SessionKey::subagent(agent_id);
        let run_id = self
            .registry
            .register(
                child_key.clone(),
                requester.clone(),
                params.task,
                params.label,
                params.model,
                params.mode,
                params.cleanup,
                params.timeout_secs,
                current_depth + 1,
            )
            .await;

        self.registry.mark_started(&run_id).await;

        SpawnResult::accepted(child_key, run_id, params.mode)
    }

    async fn kill(&self, run_id: &str) -> Result<(), String> {
        let record = self
            .registry
            .get(run_id)
            .await
            .ok_or_else(|| format!("Run '{run_id}' not found"))?;

        if record.ended_at.is_some() {
            return Err(format!("Run '{run_id}' already ended"));
        }

        self.registry
            .mark_ended(
                run_id,
                super::registry::RunOutcome {
                    success: false,
                    summary: None,
                    error: Some("Killed by parent".into()),
                },
                super::registry::EndReason::Killed,
                None,
            )
            .await;

        Ok(())
    }

    async fn list_children(
        &self,
        parent: &SessionKey,
    ) -> Vec<super::registry::SubagentRunRecord> {
        self.registry.list_for_parent(parent).await
    }

    fn registry(&self) -> &SubagentRegistry {
        &self.registry
    }

    fn depth_config(&self) -> &DepthConfig {
        &self.depth_config
    }

    fn spawn_policy(&self) -> &SpawnPolicy {
        &self.spawn_policy
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::orchestrator::spawn::SpawnMode;

    fn default_params(task: &str) -> SpawnParams {
        SpawnParams {
            task: task.into(),
            label: None,
            agent_id: Some("worker".into()),
            model: None,
            mode: SpawnMode::Run,
            sandbox: super::super::spawn::SandboxMode::Inherit,
            timeout_secs: None,
            expects_completion: true,
            cleanup: super::super::spawn::CleanupStrategy::Delete,
            allowed_tools: Vec::new(),
        }
    }

    #[tokio::test]
    async fn spawn_and_list() {
        let orch = DefaultOrchestrator::new();
        let parent = SessionKey::main("main");

        let result = orch.spawn(&parent, default_params("research"), 0).await;
        assert_eq!(result.status, super::super::spawn::SpawnStatus::Accepted);
        assert!(result.child_session_key.is_some());

        let children = orch.list_children(&parent).await;
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].task, "research");
    }

    #[tokio::test]
    async fn spawn_respects_depth_limit() {
        let orch = DefaultOrchestrator::with_config(
            super::super::depth::DepthConfig {
                max_spawn_depth: 2,
                ..Default::default()
            },
            SpawnPolicy::default(),
        );
        let parent = SessionKey::main("main");

        let r1 = orch.spawn(&parent, default_params("t1"), 0).await;
        assert_eq!(r1.status, super::super::spawn::SpawnStatus::Accepted);

        let r2 = orch.spawn(&parent, default_params("t2"), 1).await;
        assert_eq!(r2.status, super::super::spawn::SpawnStatus::Accepted);

        let r3 = orch.spawn(&parent, default_params("t3"), 2).await;
        assert_eq!(r3.status, super::super::spawn::SpawnStatus::Forbidden);
    }

    #[tokio::test]
    async fn spawn_respects_concurrency_limit() {
        let orch = DefaultOrchestrator::with_config(
            super::super::depth::DepthConfig {
                max_children_per_parent: 2,
                ..Default::default()
            },
            SpawnPolicy::default(),
        );
        let parent = SessionKey::main("main");

        let _r1 = orch.spawn(&parent, default_params("t1"), 0).await;
        let _r2 = orch.spawn(&parent, default_params("t2"), 0).await;
        let r3 = orch.spawn(&parent, default_params("t3"), 0).await;
        assert_eq!(r3.status, super::super::spawn::SpawnStatus::Forbidden);
    }

    #[tokio::test]
    async fn kill_marks_run_ended() {
        let orch = DefaultOrchestrator::new();
        let parent = SessionKey::main("main");

        let result = orch.spawn(&parent, default_params("task"), 0).await;
        let run_id = result.run_id.unwrap();

        orch.kill(&run_id).await.unwrap();

        let record = orch.registry().get(&run_id).await.unwrap();
        assert!(record.ended_at.is_some());
        assert_eq!(
            record.end_reason,
            Some(super::super::registry::EndReason::Killed)
        );
    }

    #[tokio::test]
    async fn kill_already_ended_fails() {
        let orch = DefaultOrchestrator::new();
        let parent = SessionKey::main("main");

        let result = orch.spawn(&parent, default_params("task"), 0).await;
        let run_id = result.run_id.unwrap();

        orch.kill(&run_id).await.unwrap();
        assert!(orch.kill(&run_id).await.is_err());
    }
}
