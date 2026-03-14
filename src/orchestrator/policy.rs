use std::collections::HashSet;

/// Policy configuration for subagent spawning.
#[derive(Debug, Clone)]
pub struct SpawnPolicy {
    /// Whether subagent spawning is enabled at all.
    pub enabled: bool,
    /// Allowlist of agent IDs that can be spawned. Empty = all allowed.
    pub allowed_agents: HashSet<String>,
    /// Whether sandboxed sessions can spawn subagents.
    pub allow_sandboxed_spawn: bool,
}

impl Default for SpawnPolicy {
    fn default() -> Self {
        Self {
            enabled: true,
            allowed_agents: HashSet::new(),
            allow_sandboxed_spawn: true,
        }
    }
}

/// Validate a spawn request against policy.
pub fn check_spawn_policy(
    policy: &SpawnPolicy,
    agent_id: &str,
    requester_is_sandboxed: bool,
) -> Result<(), String> {
    if !policy.enabled {
        return Err("Subagent spawning is disabled by policy.".into());
    }

    if requester_is_sandboxed && !policy.allow_sandboxed_spawn {
        return Err("Sandboxed sessions cannot spawn subagents.".into());
    }

    if !policy.allowed_agents.is_empty() && !policy.allowed_agents.contains(agent_id) {
        return Err(format!(
            "Agent '{agent_id}' is not in the spawn allowlist. \
             Allowed: {}",
            policy
                .allowed_agents
                .iter()
                .cloned()
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_policy_allows_all() {
        let policy = SpawnPolicy::default();
        assert!(check_spawn_policy(&policy, "any-agent", false).is_ok());
    }

    #[test]
    fn disabled_policy_blocks_all() {
        let policy = SpawnPolicy {
            enabled: false,
            ..Default::default()
        };
        assert!(check_spawn_policy(&policy, "any-agent", false).is_err());
    }

    #[test]
    fn allowlist_enforced() {
        let mut allowed = HashSet::new();
        allowed.insert("researcher".into());
        let policy = SpawnPolicy {
            allowed_agents: allowed,
            ..Default::default()
        };
        assert!(check_spawn_policy(&policy, "researcher", false).is_ok());
        assert!(check_spawn_policy(&policy, "hacker", false).is_err());
    }

    #[test]
    fn sandboxed_spawn_blocked_when_disabled() {
        let policy = SpawnPolicy {
            allow_sandboxed_spawn: false,
            ..Default::default()
        };
        assert!(check_spawn_policy(&policy, "worker", true).is_err());
        assert!(check_spawn_policy(&policy, "worker", false).is_ok());
    }
}
