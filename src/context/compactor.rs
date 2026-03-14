use super::budget::{estimate_tokens, TokenBudget};
use super::traits::CompactResult;
use crate::providers::ChatMessage;

/// Compacts conversation history to free token budget.
///
/// Uses a priority-based eviction strategy:
/// 1. Keep system messages always.
/// 2. Keep the most recent N messages.
/// 3. Remove oldest non-system messages until within budget.
/// 4. Optionally generate a summary of removed messages.
pub struct ContextCompactor;

impl ContextCompactor {
    /// Compact a message history to fit within the token budget.
    ///
    /// Returns the compaction result and the surviving messages.
    pub fn compact(
        messages: &[ChatMessage],
        budget: &TokenBudget,
        min_recent_messages: usize,
    ) -> (CompactResult, Vec<ChatMessage>) {
        let current_tokens: usize = messages
            .iter()
            .map(|m| estimate_tokens(&m.content))
            .sum();

        if !budget.should_compact(current_tokens) {
            return (
                CompactResult {
                    messages_removed: 0,
                    messages_retained: messages.len(),
                    summary: None,
                },
                messages.to_vec(),
            );
        }

        let target_free = budget.compaction_target(current_tokens);
        let mut freed = 0usize;
        let mut removed_count = 0;
        let mut removed_contents = Vec::new();

        // Separate system messages (always keep) from others.
        let mut system_msgs = Vec::new();
        let mut other_msgs: Vec<(usize, &ChatMessage)> = Vec::new();

        for (i, msg) in messages.iter().enumerate() {
            if msg.role == "system" {
                system_msgs.push(msg.clone());
            } else {
                other_msgs.push((i, msg));
            }
        }

        // Protect the most recent `min_recent_messages` non-system messages.
        let protected_start = if other_msgs.len() > min_recent_messages {
            other_msgs.len() - min_recent_messages
        } else {
            0
        };

        let mut retained_others = Vec::new();

        for (idx, (_, msg)) in other_msgs.iter().enumerate() {
            if idx < protected_start && freed < target_free {
                let tokens = estimate_tokens(&msg.content);
                freed += tokens;
                removed_count += 1;
                // Collect summaries for removed messages.
                if !msg.content.is_empty() {
                    let preview = if msg.content.len() > 100 {
                        format!("[{}] {}...", msg.role, &msg.content[..100])
                    } else {
                        format!("[{}] {}", msg.role, msg.content)
                    };
                    removed_contents.push(preview);
                }
            } else {
                retained_others.push((*msg).clone());
            }
        }

        // Rebuild: system messages + surviving others (in order).
        let mut result_messages = system_msgs;
        result_messages.extend(retained_others);

        let summary = if removed_contents.is_empty() {
            None
        } else {
            Some(format!(
                "[Context compacted: {} messages removed]\n{}",
                removed_count,
                removed_contents.join("\n")
            ))
        };

        (
            CompactResult {
                messages_removed: removed_count,
                messages_retained: result_messages.len(),
                summary,
            },
            result_messages,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn msg(role: &str, content: &str) -> ChatMessage {
        ChatMessage {
            role: role.to_string(),
            content: content.to_string(),
        }
    }

    #[test]
    fn no_compaction_when_under_threshold() {
        let messages = vec![msg("user", "hello"), msg("assistant", "hi")];
        let budget = TokenBudget {
            max_context_tokens: 100_000,
            compaction_threshold: 0.8,
            ..Default::default()
        };

        let (result, retained) = ContextCompactor::compact(&messages, &budget, 2);
        assert_eq!(result.messages_removed, 0);
        assert_eq!(retained.len(), 2);
    }

    #[test]
    fn compaction_removes_oldest_first() {
        // Create messages that will exceed the budget.
        let content = "x".repeat(1000); // ~250 tokens each
        let messages = vec![
            msg("system", "you are helpful"),
            msg("user", &content),     // oldest non-system
            msg("assistant", &content), // second oldest
            msg("user", &content),     // recent
            msg("assistant", &content), // most recent
        ];

        let budget = TokenBudget {
            max_context_tokens: 800,
            response_reserve: 0,
            compaction_threshold: 0.5,
            min_retain_tokens: 400,
        };

        let (result, retained) = ContextCompactor::compact(&messages, &budget, 2);
        assert!(result.messages_removed > 0);
        // System message should always survive.
        assert!(retained.iter().any(|m| m.role == "system"));
        // Most recent messages should survive.
        assert!(retained.len() >= 3); // system + 2 recent minimum
    }

    #[test]
    fn system_messages_always_kept() {
        let content = "x".repeat(2000);
        let messages = vec![
            msg("system", "keep me"),
            msg("user", &content),
            msg("assistant", &content),
        ];

        let budget = TokenBudget {
            max_context_tokens: 600,
            response_reserve: 0,
            compaction_threshold: 0.3,
            min_retain_tokens: 100,
        };

        let (_, retained) = ContextCompactor::compact(&messages, &budget, 1);
        assert!(retained.iter().any(|m| m.content == "keep me"));
    }

    #[test]
    fn summary_generated_when_messages_removed() {
        let content = "x".repeat(2000);
        let messages = vec![
            msg("system", "sys"),
            msg("user", &content),
            msg("assistant", &content),
            msg("user", "recent"),
        ];

        let budget = TokenBudget {
            max_context_tokens: 600,
            response_reserve: 0,
            compaction_threshold: 0.3,
            min_retain_tokens: 100,
        };

        let (result, _) = ContextCompactor::compact(&messages, &budget, 1);
        assert!(result.messages_removed > 0);
        assert!(result.summary.is_some());
        assert!(result.summary.unwrap().contains("compacted"));
    }
}
