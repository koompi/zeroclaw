use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

use super::render::HtmlRenderer;
use super::traits::{Canvas, CanvasAction, CanvasContent, CanvasUpdate};

/// Generate a random access token for canvas session authentication.
fn generate_canvas_token() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    let pid = std::process::id();
    static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let count = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    // Mix multiple entropy sources for a hard-to-guess (but not cryptographic) token.
    format!("{:08x}{:08x}{:08x}", nanos ^ pid, pid.wrapping_mul(2654435761), count)
}

/// Canvas session state.
struct CanvasSession {
    /// Current rendered HTML content.
    current_html: String,
    /// Channel for sending actions from the canvas back to the agent.
    action_tx: mpsc::Sender<CanvasAction>,
    /// Channel for receiving actions.
    action_rx: Option<mpsc::Receiver<CanvasAction>>,
}

/// Gateway-based Canvas implementation.
///
/// Serves interactive canvas HTML through the ZeroClaw gateway.
/// Users access canvases via URLs served by the gateway HTTP server.
/// Actions are routed back to agents via an in-memory channel.
///
/// Integration path:
/// 1. Agent calls `render()` → HTML generated and stored
/// 2. Gateway serves HTML at `/__zeroclaw__/canvas/<session_id>`
/// 3. User interacts with form/dashboard
/// 4. Browser JS sends action via WebSocket or POST
/// 5. Agent receives action via `receive_action()`
pub struct GatewayCanvas {
    /// Active canvas sessions keyed by session ID.
    sessions: Arc<RwLock<HashMap<String, CanvasSession>>>,
    /// Session ID for this canvas instance.
    session_id: String,
    /// Base URL for the gateway (e.g. "http://localhost:3000").
    gateway_base_url: String,
    /// Access token for this canvas session (prevents unauthorized viewing).
    access_token: String,
}

impl GatewayCanvas {
    pub fn new(session_id: String, gateway_base_url: String) -> Self {
        let access_token = generate_canvas_token();
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            session_id,
            gateway_base_url,
            access_token,
        }
    }

    /// Get the URL where this canvas can be accessed (includes auth token).
    pub fn canvas_url(&self) -> String {
        format!(
            "{}/__zeroclaw__/canvas/{}?token={}",
            self.gateway_base_url.trim_end_matches('/'),
            self.session_id,
            self.access_token
        )
    }

    /// Validate an access token for this canvas session.
    pub fn validate_token(&self, token: &str) -> bool {
        !self.access_token.is_empty() && self.access_token == token
    }

    /// Get the current HTML for a session (used by the gateway HTTP handler).
    pub async fn get_html(&self, session_id: &str) -> Option<String> {
        let sessions = self.sessions.read().await;
        sessions.get(session_id).map(|s| s.current_html.clone())
    }

    /// Push an action from external source (gateway webhook) into the session.
    pub async fn push_action(&self, session_id: &str, action: CanvasAction) -> bool {
        let sessions = self.sessions.read().await;
        if let Some(session) = sessions.get(session_id) {
            session.action_tx.send(action).await.is_ok()
        } else {
            false
        }
    }
}

#[async_trait]
impl Canvas for GatewayCanvas {
    async fn render(&self, content: CanvasContent) -> anyhow::Result<String> {
        let html = HtmlRenderer::render(&content);
        let (tx, rx) = mpsc::channel(32);

        let mut sessions = self.sessions.write().await;
        sessions.insert(
            self.session_id.clone(),
            CanvasSession {
                current_html: html.clone(),
                action_tx: tx,
                action_rx: Some(rx),
            },
        );

        let url = self.canvas_url();
        tracing::info!(
            session = %self.session_id,
            url = %url,
            "Canvas rendered"
        );

        Ok(url)
    }

    async fn update(&self, update: CanvasUpdate) -> anyhow::Result<()> {
        let new_html = HtmlRenderer::render(&update.content);

        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(&self.session_id) {
            session.current_html = new_html;
            Ok(())
        } else {
            anyhow::bail!("Canvas session '{}' not found", self.session_id)
        }
    }

    async fn receive_action(&self) -> anyhow::Result<Option<CanvasAction>> {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(&self.session_id) {
            if let Some(ref mut rx) = session.action_rx {
                match rx.try_recv() {
                    Ok(action) => Ok(Some(action)),
                    Err(mpsc::error::TryRecvError::Empty) => Ok(None),
                    Err(mpsc::error::TryRecvError::Disconnected) => Ok(None),
                }
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    async fn close(&self) -> anyhow::Result<()> {
        let mut sessions = self.sessions.write().await;
        sessions.remove(&self.session_id);
        tracing::info!(session = %self.session_id, "Canvas session closed");
        Ok(())
    }

    fn name(&self) -> &str {
        "gateway"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canvas::traits::{CanvasContent, DashboardWidget};

    #[tokio::test]
    async fn render_returns_url() {
        let canvas = GatewayCanvas::new("test-001".into(), "http://localhost:3000".into());
        let url = canvas
            .render(CanvasContent::Markdown {
                markdown: "# Hello".into(),
            })
            .await
            .unwrap();
        assert!(url.contains("__zeroclaw__/canvas/test-001"));
        assert!(url.contains("?token="));
    }

    #[tokio::test]
    async fn get_html_after_render() {
        let canvas = GatewayCanvas::new("s1".into(), "http://localhost:3000".into());
        canvas
            .render(CanvasContent::Html {
                html: "<p>Test</p>".into(),
            })
            .await
            .unwrap();

        let html = canvas.get_html("s1").await.unwrap();
        assert!(html.contains("Test"));
    }

    #[tokio::test]
    async fn push_and_receive_action() {
        let canvas = GatewayCanvas::new("s1".into(), "http://localhost:3000".into());
        canvas
            .render(CanvasContent::Markdown {
                markdown: "test".into(),
            })
            .await
            .unwrap();

        let action = CanvasAction {
            action_id: "submit".into(),
            data: serde_json::json!({"name": "Alice"}),
        };
        assert!(canvas.push_action("s1", action).await);

        let received = canvas.receive_action().await.unwrap();
        assert!(received.is_some());
        assert_eq!(received.unwrap().action_id, "submit");
    }

    #[tokio::test]
    async fn close_removes_session() {
        let canvas = GatewayCanvas::new("s1".into(), "http://localhost:3000".into());
        canvas
            .render(CanvasContent::Markdown {
                markdown: "test".into(),
            })
            .await
            .unwrap();

        canvas.close().await.unwrap();
        assert!(canvas.get_html("s1").await.is_none());
    }

    #[tokio::test]
    async fn update_replaces_content() {
        let canvas = GatewayCanvas::new("s1".into(), "http://localhost:3000".into());
        canvas
            .render(CanvasContent::Html {
                html: "<p>Old</p>".into(),
            })
            .await
            .unwrap();

        canvas
            .update(CanvasUpdate {
                target: None,
                content: CanvasContent::Html {
                    html: "<p>New</p>".into(),
                },
            })
            .await
            .unwrap();

        let html = canvas.get_html("s1").await.unwrap();
        assert!(html.contains("New"));
        assert!(!html.contains("Old"));
    }
}
