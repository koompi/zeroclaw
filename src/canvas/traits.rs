use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Content types that can be rendered in the canvas.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CanvasContent {
    /// Raw HTML content.
    Html { html: String },
    /// Markdown content (rendered to HTML).
    Markdown { markdown: String },
    /// Structured data rendered as a table.
    Table {
        headers: Vec<String>,
        rows: Vec<Vec<String>>,
        title: Option<String>,
    },
    /// Interactive form for user input.
    Form {
        fields: Vec<FormField>,
        submit_label: String,
        action_id: String,
    },
    /// Dashboard with multiple widgets.
    Dashboard { widgets: Vec<DashboardWidget> },
}

/// A form field definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormField {
    pub name: String,
    pub label: String,
    pub field_type: String, // "text", "select", "textarea", "checkbox"
    pub required: bool,
    pub options: Option<Vec<String>>, // for select fields
    pub default_value: Option<String>,
}

/// A dashboard widget.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DashboardWidget {
    /// Key-value metric display.
    Metric {
        label: String,
        value: String,
        trend: Option<String>,
    },
    /// Status indicator.
    Status { label: String, ok: bool },
    /// Markdown text block.
    Text { content: String },
    /// Progress bar.
    Progress {
        label: String,
        current: f64,
        total: f64,
    },
}

/// An action sent from the canvas back to the agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasAction {
    /// Action identifier (matches form action_id or widget action).
    pub action_id: String,
    /// Payload from the user interaction.
    pub data: serde_json::Value,
}

/// An update to push to the canvas (for live reload).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasUpdate {
    /// Target element ID to update.
    pub target: Option<String>,
    /// New content to render.
    pub content: CanvasContent,
}

/// Core canvas trait for interactive UI rendering from agent context.
///
/// Inspired by OpenClaw's canvas-host system. The canvas provides a
/// way for agents to render interactive UIs (dashboards, forms, reports)
/// that users can interact with, sending actions back to the agent.
#[async_trait]
pub trait Canvas: Send + Sync {
    /// Render content to the canvas.
    async fn render(&self, content: CanvasContent) -> anyhow::Result<String>;

    /// Push a live update to the canvas.
    async fn update(&self, update: CanvasUpdate) -> anyhow::Result<()>;

    /// Receive an action from the canvas (user interaction).
    async fn receive_action(&self) -> anyhow::Result<Option<CanvasAction>>;

    /// Close the canvas session.
    async fn close(&self) -> anyhow::Result<()>;

    /// Canvas name for diagnostics.
    fn name(&self) -> &str;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canvas_content_serialization() {
        let content = CanvasContent::Table {
            headers: vec!["Name".into(), "Price".into()],
            rows: vec![vec!["Product A".into(), "$10".into()]],
            title: Some("Pricing".into()),
        };

        let json = serde_json::to_string(&content).unwrap();
        assert!(json.contains("table"));
        assert!(json.contains("Pricing"));
    }

    #[test]
    fn dashboard_widget_variants() {
        let widgets = vec![
            DashboardWidget::Metric {
                label: "Revenue".into(),
                value: "$10K".into(),
                trend: Some("+15%".into()),
            },
            DashboardWidget::Status {
                label: "API".into(),
                ok: true,
            },
            DashboardWidget::Progress {
                label: "Tasks".into(),
                current: 7.0,
                total: 10.0,
            },
        ];

        let json = serde_json::to_string(&widgets).unwrap();
        assert!(json.contains("metric"));
        assert!(json.contains("status"));
        assert!(json.contains("progress"));
    }

    #[test]
    fn form_content_roundtrip() {
        let content = CanvasContent::Form {
            fields: vec![FormField {
                name: "email".into(),
                label: "Email Address".into(),
                field_type: "text".into(),
                required: true,
                options: None,
                default_value: None,
            }],
            submit_label: "Submit".into(),
            action_id: "signup".into(),
        };

        let json = serde_json::to_string(&content).unwrap();
        let parsed: CanvasContent = serde_json::from_str(&json).unwrap();
        match parsed {
            CanvasContent::Form { fields, .. } => {
                assert_eq!(fields[0].name, "email");
            }
            _ => panic!("wrong variant"),
        }
    }
}
