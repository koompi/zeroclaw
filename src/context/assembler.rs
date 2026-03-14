use super::budget::TokenBudget;
use super::traits::{AssembleResult, ContextSegment};

/// Assembles context segments into a prioritized, budget-aware collection.
///
/// Segments are sorted by priority (highest first) and packed into the
/// available token budget. Lower-priority segments are dropped when
/// the budget is exceeded.
pub struct ContextAssembler;

impl ContextAssembler {
    /// Assemble segments within the token budget.
    pub fn assemble(
        mut segments: Vec<ContextSegment>,
        budget: &TokenBudget,
    ) -> AssembleResult {
        // Sort by priority descending (highest priority first).
        segments.sort_by(|a, b| {
            b.priority
                .partial_cmp(&a.priority)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let available = budget.available();
        let mut total_tokens = 0;
        let mut included = Vec::new();
        let mut dropped_count = 0;

        for segment in segments {
            if total_tokens + segment.estimated_tokens <= available {
                total_tokens += segment.estimated_tokens;
                included.push(segment);
            } else {
                dropped_count += 1;
            }
        }

        AssembleResult {
            segments: included,
            total_tokens,
            dropped_count,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_segment(content: &str, priority: f64, tokens: usize) -> ContextSegment {
        ContextSegment {
            content: content.into(),
            priority,
            source: "test".into(),
            estimated_tokens: tokens,
        }
    }

    #[test]
    fn assemble_fits_within_budget() {
        let segments = vec![
            make_segment("low", 0.1, 100),
            make_segment("high", 0.9, 100),
            make_segment("mid", 0.5, 100),
        ];

        let budget = TokenBudget {
            max_context_tokens: 250,
            response_reserve: 0,
            ..Default::default()
        };

        let result = ContextAssembler::assemble(segments, &budget);
        assert_eq!(result.segments.len(), 2);
        assert_eq!(result.dropped_count, 1);
        // High priority should be first.
        assert_eq!(result.segments[0].content, "high");
        assert_eq!(result.segments[1].content, "mid");
        assert_eq!(result.total_tokens, 200);
    }

    #[test]
    fn assemble_all_fit() {
        let segments = vec![
            make_segment("a", 1.0, 50),
            make_segment("b", 0.5, 50),
        ];

        let budget = TokenBudget {
            max_context_tokens: 200,
            response_reserve: 0,
            ..Default::default()
        };

        let result = ContextAssembler::assemble(segments, &budget);
        assert_eq!(result.segments.len(), 2);
        assert_eq!(result.dropped_count, 0);
    }

    #[test]
    fn assemble_respects_response_reserve() {
        let segments = vec![make_segment("a", 1.0, 100)];

        let budget = TokenBudget {
            max_context_tokens: 150,
            response_reserve: 100,
            ..Default::default()
        };

        // Available = 150 - 100 = 50, but segment needs 100.
        let result = ContextAssembler::assemble(segments, &budget);
        assert_eq!(result.segments.len(), 0);
        assert_eq!(result.dropped_count, 1);
    }

    #[test]
    fn assemble_empty_segments() {
        let result = ContextAssembler::assemble(vec![], &TokenBudget::default());
        assert!(result.segments.is_empty());
        assert_eq!(result.total_tokens, 0);
        assert_eq!(result.dropped_count, 0);
    }
}
