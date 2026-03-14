/// Default maximum spawn depth to prevent runaway recursion.
pub const DEFAULT_MAX_SPAWN_DEPTH: u32 = 10;

/// Default maximum concurrent children per parent session.
pub const DEFAULT_MAX_CHILDREN_PER_PARENT: usize = 5;

/// Configuration for spawn depth and concurrency limits.
#[derive(Debug, Clone)]
pub struct DepthConfig {
    /// Maximum depth in the spawn chain (0 = top-level, N = Nth nested subagent).
    pub max_spawn_depth: u32,
    /// Maximum concurrent active children per parent session.
    pub max_children_per_parent: usize,
}

impl Default for DepthConfig {
    fn default() -> Self {
        Self {
            max_spawn_depth: DEFAULT_MAX_SPAWN_DEPTH,
            max_children_per_parent: DEFAULT_MAX_CHILDREN_PER_PARENT,
        }
    }
}

/// Check if a spawn at the given depth is allowed.
pub fn check_depth(current_depth: u32, config: &DepthConfig) -> Result<(), String> {
    if current_depth >= config.max_spawn_depth {
        return Err(format!(
            "Spawn depth limit reached ({current_depth}/{max}). Cannot spawn further subagents.",
            max = config.max_spawn_depth
        ));
    }
    Ok(())
}

/// Check if the parent has capacity for another concurrent child.
pub fn check_concurrency(active_children: usize, config: &DepthConfig) -> Result<(), String> {
    if active_children >= config.max_children_per_parent {
        return Err(format!(
            "Concurrent children limit reached ({active_children}/{max}). \
             Wait for existing subagents to complete.",
            max = config.max_children_per_parent
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn depth_zero_allowed() {
        let config = DepthConfig::default();
        assert!(check_depth(0, &config).is_ok());
    }

    #[test]
    fn depth_at_limit_rejected() {
        let config = DepthConfig {
            max_spawn_depth: 3,
            ..Default::default()
        };
        assert!(check_depth(2, &config).is_ok());
        assert!(check_depth(3, &config).is_err());
        assert!(check_depth(10, &config).is_err());
    }

    #[test]
    fn concurrency_within_limit_allowed() {
        let config = DepthConfig::default();
        assert!(check_concurrency(0, &config).is_ok());
        assert!(check_concurrency(4, &config).is_ok());
    }

    #[test]
    fn concurrency_at_limit_rejected() {
        let config = DepthConfig {
            max_children_per_parent: 3,
            ..Default::default()
        };
        assert!(check_concurrency(2, &config).is_ok());
        assert!(check_concurrency(3, &config).is_err());
    }
}
