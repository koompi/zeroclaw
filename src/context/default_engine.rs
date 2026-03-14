use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::assembler::ContextAssembler;
use super::budget::{estimate_tokens, TokenBudget};
use super::traits::{AssembleResult, CompactResult, ContextEngine, ContextSegment};

/// Default context engine implementation.
///
/// Uses in-memory segment storage per session. Integrates with the
/// persona system by allowing bootstrap segments to be injected.
/// Manages token budget and triggers compaction automatically.
/// Maximum number of sessions tracked before evicting the oldest.
const MAX_TRACKED_SESSIONS: usize = 1000;

pub struct DefaultContextEngine {
    /// Per-session stored segments (session_id → segments).
    sessions: Arc<RwLock<HashMap<String, Vec<ContextSegment>>>>,
}

impl DefaultContextEngine {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Evict oldest sessions if we exceed the max tracked sessions limit.
    async fn maybe_evict(&self) {
        let mut sessions = self.sessions.write().await;
        if sessions.len() > MAX_TRACKED_SESSIONS {
            // Remove sessions with fewest segments (least active).
            let mut by_size: Vec<(String, usize)> = sessions
                .iter()
                .map(|(k, v)| (k.clone(), v.len()))
                .collect();
            by_size.sort_by_key(|(_, size)| *size);
            let to_remove = sessions.len() - MAX_TRACKED_SESSIONS;
            for (key, _) in by_size.into_iter().take(to_remove) {
                sessions.remove(&key);
            }
        }
    }
}

impl Default for DefaultContextEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ContextEngine for DefaultContextEngine {
    async fn bootstrap(
        &self,
        session_id: &str,
        agent_id: &str,
    ) -> anyhow::Result<Vec<ContextSegment>> {
        let mut segments = Vec::new();

        // System identity segment (highest priority — always survives compaction).
        segments.push(ContextSegment {
            content: format!("Session: {session_id} | Agent: {agent_id}"),
            priority: 1.0,
            source: "system".into(),
            estimated_tokens: 20,
        });

        // Store bootstrap segments for this session.
        let mut sessions = self.sessions.write().await;
        sessions.insert(session_id.to_string(), segments.clone());

        Ok(segments)
    }

    async fn ingest(
        &self,
        session_id: &str,
        role: &str,
        content: &str,
    ) -> anyhow::Result<()> {
        let segment = ContextSegment {
            content: content.to_string(),
            // User messages get higher priority than assistant messages
            // to preserve the user's intent through compaction.
            priority: match role {
                "system" => 0.95,
                "user" => 0.7,
                "assistant" => 0.5,
                "tool" => 0.3,
                _ => 0.4,
            },
            source: format!("message:{role}"),
            estimated_tokens: estimate_tokens(content),
        };

        {
            let mut sessions = self.sessions.write().await;
            sessions
                .entry(session_id.to_string())
                .or_default()
                .push(segment);
        }

        self.maybe_evict().await;

        Ok(())
    }

    async fn assemble(
        &self,
        session_id: &str,
        _query: &str,
        budget: &TokenBudget,
    ) -> anyhow::Result<AssembleResult> {
        let sessions = self.sessions.read().await;
        let segments = sessions
            .get(session_id)
            .cloned()
            .unwrap_or_default();

        Ok(ContextAssembler::assemble(segments, budget))
    }

    async fn compact(
        &self,
        session_id: &str,
        budget: &TokenBudget,
        force: bool,
    ) -> anyhow::Result<CompactResult> {
        let mut sessions = self.sessions.write().await;
        let segments = sessions
            .entry(session_id.to_string())
            .or_default();

        let total_tokens: usize = segments.iter().map(|s| s.estimated_tokens).sum();

        if !force && !budget.should_compact(total_tokens) {
            return Ok(CompactResult {
                messages_removed: 0,
                messages_retained: segments.len(),
                summary: None,
            });
        }

        // Sort by priority ascending (lowest first — they get removed).
        segments.sort_by(|a, b| {
            a.priority
                .partial_cmp(&b.priority)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let target_free = budget.compaction_target(total_tokens);
        let mut freed = 0usize;
        let mut removed = 0;
        let mut removed_summaries = Vec::new();

        // Remove lowest-priority segments until we've freed enough.
        segments.retain(|seg| {
            if freed >= target_free {
                return true; // keep
            }
            // Never remove system segments.
            if seg.source == "system" || seg.priority >= 0.95 {
                return true;
            }
            freed += seg.estimated_tokens;
            removed += 1;
            if seg.content.len() > 80 {
                removed_summaries.push(format!("[{}] {}...", seg.source, &seg.content[..80]));
            } else {
                removed_summaries.push(format!("[{}] {}", seg.source, seg.content));
            }
            false // remove
        });

        let summary = if removed_summaries.is_empty() {
            None
        } else {
            Some(format!(
                "[Context compacted: {removed} segments removed, {freed} tokens freed]\n{}",
                removed_summaries.join("\n")
            ))
        };

        Ok(CompactResult {
            messages_removed: removed,
            messages_retained: segments.len(),
            summary,
        })
    }

    fn name(&self) -> &str {
        "default"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn bootstrap_creates_session() {
        let engine = DefaultContextEngine::new();
        let segments = engine.bootstrap("s1", "forge").await.unwrap();
        assert!(!segments.is_empty());
        assert!(segments[0].content.contains("forge"));
    }

    #[tokio::test]
    async fn ingest_and_assemble() {
        let engine = DefaultContextEngine::new();
        engine.bootstrap("s1", "oracle").await.unwrap();
        engine.ingest("s1", "user", "What is the market size?").await.unwrap();
        engine
            .ingest("s1", "assistant", "The TAM is estimated at $10B.")
            .await
            .unwrap();

        let budget = TokenBudget::default();
        let result = engine.assemble("s1", "market", &budget).await.unwrap();
        assert!(result.segments.len() >= 3);
        assert!(result.total_tokens > 0);
    }

    #[tokio::test]
    async fn compact_removes_low_priority() {
        let engine = DefaultContextEngine::new();
        engine.bootstrap("s1", "test").await.unwrap();

        // Add many low-priority segments.
        for i in 0..20 {
            engine
                .ingest("s1", "tool", &format!("tool output {i} {}", "x".repeat(100)))
                .await
                .unwrap();
        }

        let budget = TokenBudget {
            max_context_tokens: 500,
            compaction_threshold: 0.3,
            min_retain_tokens: 100,
            ..Default::default()
        };

        let result = engine.compact("s1", &budget, true).await.unwrap();
        assert!(result.messages_removed > 0);
        assert!(result.summary.is_some());
    }

    #[tokio::test]
    async fn compact_preserves_system_segments() {
        let engine = DefaultContextEngine::new();
        engine.bootstrap("s1", "archon").await.unwrap();
        engine.ingest("s1", "system", "critical system info").await.unwrap();

        let budget = TokenBudget {
            max_context_tokens: 50,
            compaction_threshold: 0.1,
            min_retain_tokens: 10,
            ..Default::default()
        };

        let result = engine.compact("s1", &budget, true).await.unwrap();
        // System segments should survive.
        let sessions = engine.sessions.read().await;
        let segments = sessions.get("s1").unwrap();
        assert!(segments.iter().any(|s| s.source == "system"));
    }
}
