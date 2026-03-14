use async_trait::async_trait;

use super::budget::TokenBudget;

/// A segment of context with a priority score for ranking.
#[derive(Debug, Clone)]
pub struct ContextSegment {
    /// The content text.
    pub content: String,
    /// Priority score (higher = more important, more likely to survive compaction).
    pub priority: f64,
    /// Source label (e.g. "memory", "rag", "system", "persona").
    pub source: String,
    /// Estimated token count for this segment.
    pub estimated_tokens: usize,
}

/// Result of a context assembly operation.
#[derive(Debug, Clone)]
pub struct AssembleResult {
    /// Assembled segments in priority order.
    pub segments: Vec<ContextSegment>,
    /// Total estimated tokens across all segments.
    pub total_tokens: usize,
    /// Number of segments dropped due to budget.
    pub dropped_count: usize,
}

/// Result of a context compaction operation.
#[derive(Debug, Clone)]
pub struct CompactResult {
    /// Number of messages removed.
    pub messages_removed: usize,
    /// Number of messages retained.
    pub messages_retained: usize,
    /// Summary of compacted content (if generated).
    pub summary: Option<String>,
}

/// Core context engine trait for managing agent context.
///
/// Inspired by OpenClaw's pluggable context engine pattern.
/// Implementations handle context assembly (what goes into the prompt),
/// ingestion (processing new messages), and compaction (shrinking
/// context when approaching token limits).
#[async_trait]
pub trait ContextEngine: Send + Sync {
    /// Bootstrap context for a new session.
    /// Called once when a session starts. Returns initial context segments
    /// (e.g. persona prompt, workspace info, relevant memories).
    async fn bootstrap(
        &self,
        session_id: &str,
        agent_id: &str,
    ) -> anyhow::Result<Vec<ContextSegment>>;

    /// Ingest a new message into the context engine.
    /// Called after each user or assistant message. The engine can
    /// store embeddings, update indexes, etc.
    async fn ingest(
        &self,
        session_id: &str,
        role: &str,
        content: &str,
    ) -> anyhow::Result<()>;

    /// Assemble context for the next LLM call.
    /// Returns ranked segments that fit within the token budget.
    async fn assemble(
        &self,
        session_id: &str,
        query: &str,
        budget: &TokenBudget,
    ) -> anyhow::Result<AssembleResult>;

    /// Compact the session context to free token budget.
    /// Called when approaching context limits. Removes old/low-priority
    /// messages and optionally summarizes them.
    async fn compact(
        &self,
        session_id: &str,
        budget: &TokenBudget,
        force: bool,
    ) -> anyhow::Result<CompactResult>;

    /// Engine name for diagnostics.
    fn name(&self) -> &str;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn context_segment_creation() {
        let seg = ContextSegment {
            content: "test content".into(),
            priority: 1.0,
            source: "memory".into(),
            estimated_tokens: 10,
        };
        assert_eq!(seg.priority, 1.0);
        assert_eq!(seg.source, "memory");
    }
}
