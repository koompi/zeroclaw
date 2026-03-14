use async_trait::async_trait;
use serde_json::Value;
use std::time::Duration;

use crate::channels::traits::ChannelMessage;
use crate::orchestrator::session::SessionKey;
use crate::orchestrator::spawn::SpawnParams;
use crate::providers::traits::{ChatMessage, ChatResponse};
use crate::tools::traits::ToolResult;

/// Result of a modifying hook — continue with (possibly modified) data, or cancel.
#[derive(Debug, Clone)]
pub enum HookResult<T> {
    Continue(T),
    Cancel(String),
}

impl<T> HookResult<T> {
    pub fn is_cancel(&self) -> bool {
        matches!(self, HookResult::Cancel(_))
    }
}

/// Trait for hook handlers. All methods have default no-op implementations.
/// Implement only the events you care about.
#[async_trait]
pub trait HookHandler: Send + Sync {
    fn name(&self) -> &str;
    fn priority(&self) -> i32 {
        0
    }

    // --- Void hooks (parallel, fire-and-forget) ---
    async fn on_gateway_start(&self, _host: &str, _port: u16) {}
    async fn on_gateway_stop(&self) {}
    async fn on_session_start(&self, _session_id: &str, _channel: &str) {}
    async fn on_session_end(&self, _session_id: &str, _channel: &str) {}
    async fn on_llm_input(&self, _messages: &[ChatMessage], _model: &str) {}
    async fn on_llm_output(&self, _response: &ChatResponse) {}
    async fn on_after_tool_call(&self, _tool: &str, _result: &ToolResult, _duration: Duration) {}
    async fn on_message_sent(&self, _channel: &str, _recipient: &str, _content: &str) {}
    async fn on_heartbeat_tick(&self) {}

    // --- Subagent lifecycle hooks (void, parallel) ---
    async fn on_subagent_spawning(&self, _parent: &SessionKey, _params: &SpawnParams) {}
    async fn on_subagent_spawned(&self, _parent: &SessionKey, _child: &SessionKey, _run_id: &str) {}
    async fn on_subagent_ended(
        &self,
        _child: &SessionKey,
        _run_id: &str,
        _success: bool,
        _summary: Option<&str>,
    ) {
    }

    // --- Context/compaction hooks (void, parallel) ---
    async fn before_compaction(&self, _session_id: &str, _message_count: usize) {}
    async fn after_compaction(&self, _session_id: &str, _messages_removed: usize) {}

    // --- Agent lifecycle hooks (void, parallel) ---
    async fn before_agent_start(&self, _agent_id: &str, _model: &str) {}
    async fn after_agent_end(&self, _agent_id: &str, _duration: Duration) {}

    // --- Modifying hooks (sequential by priority, can cancel) ---
    async fn before_model_resolve(
        &self,
        provider: String,
        model: String,
    ) -> HookResult<(String, String)> {
        HookResult::Continue((provider, model))
    }

    async fn before_prompt_build(&self, prompt: String) -> HookResult<String> {
        HookResult::Continue(prompt)
    }

    async fn before_llm_call(
        &self,
        messages: Vec<ChatMessage>,
        model: String,
    ) -> HookResult<(Vec<ChatMessage>, String)> {
        HookResult::Continue((messages, model))
    }

    async fn before_tool_call(&self, name: String, args: Value) -> HookResult<(String, Value)> {
        HookResult::Continue((name, args))
    }

    async fn on_message_received(&self, message: ChannelMessage) -> HookResult<ChannelMessage> {
        HookResult::Continue(message)
    }

    async fn on_message_sending(
        &self,
        channel: String,
        recipient: String,
        content: String,
    ) -> HookResult<(String, String, String)> {
        HookResult::Continue((channel, recipient, content))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestHook {
        name: String,
        priority: i32,
    }

    impl TestHook {
        fn new(name: &str, priority: i32) -> Self {
            Self {
                name: name.to_string(),
                priority,
            }
        }
    }

    #[async_trait]
    impl HookHandler for TestHook {
        fn name(&self) -> &str {
            &self.name
        }
        fn priority(&self) -> i32 {
            self.priority
        }
    }

    #[test]
    fn hook_result_is_cancel() {
        let ok: HookResult<String> = HookResult::Continue("hi".into());
        assert!(!ok.is_cancel());
        let cancel: HookResult<String> = HookResult::Cancel("blocked".into());
        assert!(cancel.is_cancel());
    }

    #[test]
    fn default_priority_is_zero() {
        struct MinimalHook;
        #[async_trait]
        impl HookHandler for MinimalHook {
            fn name(&self) -> &str {
                "minimal"
            }
        }
        assert_eq!(MinimalHook.priority(), 0);
    }

    #[tokio::test]
    async fn default_modifying_hooks_pass_through() {
        let hook = TestHook::new("test", 0);
        match hook
            .before_tool_call("shell".into(), serde_json::json!({"cmd": "ls"}))
            .await
        {
            HookResult::Continue((name, _args)) => assert_eq!(name, "shell"),
            HookResult::Cancel(_) => panic!("should not cancel"),
        }
    }
}
