use std::time::Duration;

use super::registry::{EndReason, RunOutcome, SubagentRegistry, SubagentRunRecord};

/// Maximum announce delivery retries before giving up.
const MAX_ANNOUNCE_RETRIES: u32 = 3;

/// Retry delays (exponential backoff).
const RETRY_DELAYS: [Duration; 3] = [
    Duration::from_secs(5),
    Duration::from_secs(10),
    Duration::from_secs(20),
];

/// Delivery path for announce messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeliveryPath {
    /// Direct inline delivery to the parent agent's active turn.
    Direct,
    /// Queued for deferred delivery (parent is not actively reading).
    Queued,
    /// No delivery needed (fire-and-forget mode).
    None,
}

/// Result of an announce delivery attempt.
#[derive(Debug, Clone)]
pub struct AnnounceResult {
    pub run_id: String,
    pub path: DeliveryPath,
    pub success: bool,
    pub error: Option<String>,
}

/// Format a completion announcement message from a run record.
pub fn format_announcement(record: &SubagentRunRecord) -> String {
    let label = record
        .label
        .as_deref()
        .unwrap_or(&record.child_session_key.agent_id);

    let status = match record.end_reason {
        Some(EndReason::Completed) => {
            if record.outcome.as_ref().map_or(false, |o| o.success) {
                "completed successfully"
            } else {
                "completed with errors"
            }
        }
        Some(EndReason::TimedOut) => "timed out",
        Some(EndReason::Killed) => "was terminated",
        Some(EndReason::Failed) => "failed",
        None => "ended (unknown reason)",
    };

    let mut msg = format!("[Subagent '{label}' {status}]");

    if let Some(result) = &record.frozen_result {
        let truncated = if result.len() > 4000 {
            format!("{}... (truncated)", &result[..4000])
        } else {
            result.clone()
        };
        msg.push_str(&format!("\n{truncated}"));
    } else if let Some(outcome) = &record.outcome {
        if let Some(summary) = &outcome.summary {
            msg.push_str(&format!("\n{summary}"));
        }
        if let Some(error) = &outcome.error {
            msg.push_str(&format!("\nError: {error}"));
        }
    }

    msg
}

/// Process all pending announces in the registry.
///
/// This is the push-based completion delivery system inspired by OpenClaw.
/// Children announce themselves; parents don't poll.
///
/// The `deliver` callback receives (run_id, parent_session_key_str, message)
/// and returns Ok(true) if delivered, Ok(false) if parent unavailable, Err on failure.
pub async fn process_pending_announces<F, Fut>(registry: &SubagentRegistry, deliver: F)
where
    F: Fn(String, String, String) -> Fut,
    Fut: std::future::Future<Output = Result<bool, String>>,
{
    let pending = registry.pending_announces().await;

    for record in pending {
        if record.announce_retries >= MAX_ANNOUNCE_RETRIES {
            // Give up after max retries, mark as announced to stop retrying.
            tracing::warn!(
                run_id = %record.run_id,
                "Announce delivery exhausted retries, marking as announced"
            );
            registry.mark_announced(&record.run_id).await;
            continue;
        }

        let message = format_announcement(&record);
        let parent_key = record.requester_session_key.to_key();

        match deliver(record.run_id.clone(), parent_key, message).await {
            Ok(true) => {
                registry.mark_announced(&record.run_id).await;
                tracing::info!(
                    run_id = %record.run_id,
                    child = %record.child_session_key,
                    "Announce delivered to parent"
                );
            }
            Ok(false) => {
                // Parent not available — will retry.
                registry.increment_announce_retries(&record.run_id).await;
                tracing::debug!(
                    run_id = %record.run_id,
                    retries = record.announce_retries + 1,
                    "Announce deferred, parent unavailable"
                );
            }
            Err(e) => {
                registry.increment_announce_retries(&record.run_id).await;
                tracing::warn!(
                    run_id = %record.run_id,
                    error = %e,
                    "Announce delivery failed"
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::orchestrator::registry::{EndReason, RunOutcome, SubagentRunRecord};
    use crate::orchestrator::session::{SessionKey, SessionType};
    use crate::orchestrator::spawn::{CleanupStrategy, SpawnMode};

    fn sample_record(success: bool, frozen: Option<&str>) -> SubagentRunRecord {
        SubagentRunRecord {
            run_id: "run-test".into(),
            child_session_key: SessionKey::subagent("worker"),
            requester_session_key: SessionKey::main("parent"),
            task: "test task".into(),
            label: Some("research".into()),
            model: None,
            mode: SpawnMode::Run,
            cleanup: CleanupStrategy::Delete,
            timeout_secs: None,
            depth: 0,
            created_at: 0.0,
            started_at: Some(0.1),
            ended_at: Some(1.0),
            outcome: Some(RunOutcome {
                success,
                summary: Some("summary text".into()),
                error: if success {
                    None
                } else {
                    Some("something broke".into())
                },
            }),
            end_reason: Some(if success {
                EndReason::Completed
            } else {
                EndReason::Failed
            }),
            announced: false,
            announce_retries: 0,
            frozen_result: frozen.map(String::from),
        }
    }

    #[test]
    fn format_announcement_success() {
        let record = sample_record(true, Some("the result"));
        let msg = format_announcement(&record);
        assert!(msg.contains("'research'"));
        assert!(msg.contains("completed successfully"));
        assert!(msg.contains("the result"));
    }

    #[test]
    fn format_announcement_failure_with_summary() {
        let record = sample_record(false, None);
        let msg = format_announcement(&record);
        assert!(msg.contains("failed"));
        assert!(msg.contains("summary text"));
        assert!(msg.contains("something broke"));
    }

    #[test]
    fn format_announcement_truncates_long_results() {
        let long_result = "x".repeat(5000);
        let record = sample_record(true, Some(&long_result));
        let msg = format_announcement(&record);
        assert!(msg.contains("truncated"));
        assert!(msg.len() < 5000);
    }

    #[tokio::test]
    async fn process_pending_delivers_and_marks() {
        let registry = SubagentRegistry::new();
        let parent = SessionKey::main("p");
        let run_id = registry
            .register(
                SessionKey::subagent("c"),
                parent.clone(),
                "task".into(),
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
                    summary: Some("done".into()),
                    error: None,
                },
                EndReason::Completed,
                Some("result".into()),
            )
            .await;

        let delivered = std::sync::Arc::new(tokio::sync::Mutex::new(Vec::new()));
        let delivered_clone = delivered.clone();

        process_pending_announces(&registry, |rid, _parent, msg| {
            let d = delivered_clone.clone();
            async move {
                d.lock().await.push((rid, msg));
                Ok(true)
            }
        })
        .await;

        let d = delivered.lock().await;
        assert_eq!(d.len(), 1);
        assert!(d[0].1.contains("result"));

        // Should be marked as announced now.
        assert!(registry.pending_announces().await.is_empty());
    }
}
