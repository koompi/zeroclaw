/// Token budget configuration for context management.
///
/// Controls how much context can be assembled for LLM calls
/// and when compaction should trigger.
#[derive(Debug, Clone)]
pub struct TokenBudget {
    /// Maximum total tokens for the assembled context.
    pub max_context_tokens: usize,
    /// Reserved tokens for the model's response.
    pub response_reserve: usize,
    /// Threshold at which compaction should trigger (as fraction of max, e.g. 0.8).
    pub compaction_threshold: f64,
    /// Minimum tokens to retain after compaction.
    pub min_retain_tokens: usize,
}

impl TokenBudget {
    /// Available tokens for context (max minus response reserve).
    pub fn available(&self) -> usize {
        self.max_context_tokens.saturating_sub(self.response_reserve)
    }

    /// Whether compaction should trigger given current usage.
    pub fn should_compact(&self, current_tokens: usize) -> bool {
        let threshold = (self.max_context_tokens as f64 * self.compaction_threshold) as usize;
        current_tokens >= threshold
    }

    /// Tokens that should be freed during compaction.
    pub fn compaction_target(&self, current_tokens: usize) -> usize {
        if current_tokens <= self.min_retain_tokens {
            return 0;
        }
        current_tokens.saturating_sub(self.min_retain_tokens)
    }
}

impl Default for TokenBudget {
    fn default() -> Self {
        Self {
            max_context_tokens: 128_000, // 128K context window
            response_reserve: 4_096,     // Reserve 4K for response
            compaction_threshold: 0.8,   // Compact at 80% full
            min_retain_tokens: 32_000,   // Keep at least 32K after compaction
        }
    }
}

/// Estimate token count for a string (rough: ~4 chars per token for English).
pub fn estimate_tokens(text: &str) -> usize {
    // This is a rough heuristic. For production, use tiktoken or similar.
    (text.len() + 3) / 4
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_budget() {
        let budget = TokenBudget::default();
        assert_eq!(budget.available(), 128_000 - 4_096);
    }

    #[test]
    fn should_compact_at_threshold() {
        let budget = TokenBudget {
            max_context_tokens: 100_000,
            compaction_threshold: 0.8,
            ..Default::default()
        };
        assert!(!budget.should_compact(79_999));
        assert!(budget.should_compact(80_000));
        assert!(budget.should_compact(100_000));
    }

    #[test]
    fn compaction_target() {
        let budget = TokenBudget {
            min_retain_tokens: 30_000,
            ..Default::default()
        };
        assert_eq!(budget.compaction_target(50_000), 20_000);
        assert_eq!(budget.compaction_target(30_000), 0);
        assert_eq!(budget.compaction_target(20_000), 0);
    }

    #[test]
    fn estimate_tokens_rough() {
        assert_eq!(estimate_tokens(""), 0);
        assert_eq!(estimate_tokens("hello"), 2); // 5 chars -> ~1.25 tokens, ceil
        assert!(estimate_tokens("a longer string with more words") > 5);
    }
}
