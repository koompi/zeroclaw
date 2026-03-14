use super::traits::{CanvasContent, DashboardWidget, FormField};

/// Escape HTML special characters to prevent XSS.
fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

/// Renders canvas content to HTML.
///
/// Used by the gateway to serve interactive UI pages.
/// Generates self-contained HTML with Tailwind CSS for styling.
pub struct HtmlRenderer;

impl HtmlRenderer {
    /// Render canvas content to an HTML string.
    pub fn render(content: &CanvasContent) -> String {
        let body = match content {
            CanvasContent::Html { html } => html.clone(),
            CanvasContent::Markdown { markdown } => render_markdown(markdown),
            CanvasContent::Table {
                headers,
                rows,
                title,
            } => render_table(title.as_deref(), headers, rows),
            CanvasContent::Form {
                fields,
                submit_label,
                action_id,
            } => render_form(fields, submit_label, action_id),
            CanvasContent::Dashboard { widgets } => render_dashboard(widgets),
        };

        wrap_html(&body)
    }
}

fn wrap_html(body: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en" class="dark">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>ZeroClaw Canvas</title>
<script src="https://cdn.tailwindcss.com"></script>
<style>
  body {{ font-family: ui-sans-serif, system-ui, sans-serif; }}
</style>
</head>
<body class="bg-zinc-900 text-zinc-100 min-h-screen p-6">
<div class="max-w-4xl mx-auto">
{body}
</div>
<script>
  // Action bridge: sends user actions back to the agent via WebSocket.
  window.ZeroClaw = {{
    sendAction: function(actionId, data) {{
      if (window._ws && window._ws.readyState === 1) {{
        window._ws.send(JSON.stringify({{ action_id: actionId, data: data }}));
      }}
    }}
  }};
</script>
</body>
</html>"#
    )
}

fn render_markdown(markdown: &str) -> String {
    // Basic markdown-to-HTML conversion (headers, bold, links, code).
    // For production, use pulldown-cmark or similar.
    let mut html = String::new();
    for line in markdown.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("### ") {
            html.push_str(&format!(
                "<h3 class=\"text-lg font-semibold mt-4 mb-2\">{}</h3>\n",
                &trimmed[4..]
            ));
        } else if trimmed.starts_with("## ") {
            html.push_str(&format!(
                "<h2 class=\"text-xl font-bold mt-6 mb-3\">{}</h2>\n",
                &trimmed[3..]
            ));
        } else if trimmed.starts_with("# ") {
            html.push_str(&format!(
                "<h1 class=\"text-2xl font-bold mt-8 mb-4\">{}</h1>\n",
                &trimmed[2..]
            ));
        } else if trimmed.starts_with("- ") {
            html.push_str(&format!(
                "<li class=\"ml-4\">{}</li>\n",
                &trimmed[2..]
            ));
        } else if trimmed.is_empty() {
            html.push_str("<br>\n");
        } else {
            html.push_str(&format!("<p class=\"mb-2\">{trimmed}</p>\n"));
        }
    }
    html
}

fn render_table(title: Option<&str>, headers: &[String], rows: &[Vec<String>]) -> String {
    let mut html = String::new();

    if let Some(title) = title {
        html.push_str(&format!(
            "<h2 class=\"text-xl font-bold mb-4\">{title}</h2>\n"
        ));
    }

    html.push_str("<div class=\"overflow-x-auto\">\n");
    html.push_str("<table class=\"w-full border-collapse\">\n<thead>\n<tr>\n");
    for header in headers {
        html.push_str(&format!(
            "  <th class=\"text-left p-3 bg-zinc-800 border-b border-zinc-700 font-semibold\">{}</th>\n",
            escape_html(header)
        ));
    }
    html.push_str("</tr>\n</thead>\n<tbody>\n");

    for row in rows {
        html.push_str("<tr class=\"hover:bg-zinc-800/50\">\n");
        for cell in row {
            html.push_str(&format!(
                "  <td class=\"p-3 border-b border-zinc-800\">{}</td>\n",
                escape_html(cell)
            ));
        }
        html.push_str("</tr>\n");
    }

    html.push_str("</tbody>\n</table>\n</div>\n");
    html
}

fn render_form(fields: &[FormField], submit_label: &str, action_id: &str) -> String {
    let mut html = String::new();
    html.push_str(&format!(
        "<form id=\"form-{action_id}\" class=\"space-y-4\" onsubmit=\"return handleSubmit(event, '{action_id}')\">\n"
    ));

    for field in fields {
        html.push_str("<div class=\"space-y-1\">\n");
        html.push_str(&format!(
            "  <label class=\"block text-sm font-medium text-zinc-300\">{}{}</label>\n",
            escape_html(&field.label),
            if field.required { " *" } else { "" }
        ));

        match field.field_type.as_str() {
            "textarea" => {
                html.push_str(&format!(
                    "  <textarea name=\"{}\" class=\"w-full p-2 bg-zinc-800 border border-zinc-700 rounded-md text-zinc-100\" rows=\"4\"{}>{}</textarea>\n",
                    field.name,
                    if field.required { " required" } else { "" },
                    field.default_value.as_deref().unwrap_or("")
                ));
            }
            "select" => {
                html.push_str(&format!(
                    "  <select name=\"{}\" class=\"w-full p-2 bg-zinc-800 border border-zinc-700 rounded-md text-zinc-100\"{}>\n",
                    field.name,
                    if field.required { " required" } else { "" }
                ));
                if let Some(options) = &field.options {
                    for opt in options {
                        html.push_str(&format!("    <option value=\"{opt}\">{opt}</option>\n"));
                    }
                }
                html.push_str("  </select>\n");
            }
            "checkbox" => {
                html.push_str(&format!(
                    "  <input type=\"checkbox\" name=\"{}\" class=\"rounded bg-zinc-800 border-zinc-700\">\n",
                    field.name
                ));
            }
            _ => {
                // Default: text input.
                html.push_str(&format!(
                    "  <input type=\"text\" name=\"{}\" value=\"{}\" class=\"w-full p-2 bg-zinc-800 border border-zinc-700 rounded-md text-zinc-100\" placeholder=\"{}\"{}>\n",
                    field.name,
                    field.default_value.as_deref().unwrap_or(""),
                    field.label,
                    if field.required { " required" } else { "" }
                ));
            }
        }

        html.push_str("</div>\n");
    }

    html.push_str(&format!(
        "<button type=\"submit\" class=\"px-4 py-2 bg-blue-600 hover:bg-blue-500 text-white rounded-md font-medium\">{submit_label}</button>\n"
    ));
    html.push_str("</form>\n");
    html.push_str(&format!(
        r#"<script>
function handleSubmit(e, actionId) {{
  e.preventDefault();
  var fd = new FormData(e.target);
  var data = {{}};
  fd.forEach(function(v, k) {{ data[k] = v; }});
  window.ZeroClaw.sendAction(actionId, data);
  return false;
}}
</script>"#
    ));
    html
}

fn render_dashboard(widgets: &[DashboardWidget]) -> String {
    let mut html = String::new();
    html.push_str("<div class=\"grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4\">\n");

    for widget in widgets {
        match widget {
            DashboardWidget::Metric {
                label,
                value,
                trend,
            } => {
                let trend_html = trend
                    .as_ref()
                    .map(|t| {
                        let color = if t.starts_with('+') {
                            "text-green-400"
                        } else if t.starts_with('-') {
                            "text-red-400"
                        } else {
                            "text-zinc-400"
                        };
                        format!("<span class=\"text-sm {color}\">{t}</span>")
                    })
                    .unwrap_or_default();

                html.push_str(&format!(
                    "<div class=\"p-4 bg-zinc-800 rounded-lg\">\n  <div class=\"text-sm text-zinc-400\">{}</div>\n  <div class=\"text-2xl font-bold mt-1\">{}</div>\n  {trend_html}\n</div>\n",
                    escape_html(label),
                    escape_html(value)
                ));
            }
            DashboardWidget::Status { label, ok } => {
                let (color, icon) = if *ok {
                    ("text-green-400", "●")
                } else {
                    ("text-red-400", "●")
                };
                html.push_str(&format!(
                    "<div class=\"p-4 bg-zinc-800 rounded-lg flex items-center gap-2\">\n  <span class=\"{color}\">{icon}</span>\n  <span>{label}</span>\n</div>\n"
                ));
            }
            DashboardWidget::Text { content } => {
                html.push_str(&format!(
                    "<div class=\"p-4 bg-zinc-800 rounded-lg col-span-full\">\n{}</div>\n",
                    render_markdown(content)
                ));
            }
            DashboardWidget::Progress {
                label,
                current,
                total,
            } => {
                let pct = if *total > 0.0 {
                    (current / total * 100.0).min(100.0)
                } else {
                    0.0
                };
                html.push_str(&format!(
                    "<div class=\"p-4 bg-zinc-800 rounded-lg\">\n  <div class=\"text-sm text-zinc-400 mb-2\">{label}</div>\n  <div class=\"w-full bg-zinc-700 rounded-full h-2\">\n    <div class=\"bg-blue-500 h-2 rounded-full\" style=\"width: {pct:.0}%\"></div>\n  </div>\n  <div class=\"text-sm mt-1 text-zinc-400\">{current:.0} / {total:.0}</div>\n</div>\n"
                ));
            }
        }
    }

    html.push_str("</div>\n");
    html
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_html_passthrough() {
        let content = CanvasContent::Html {
            html: "<p>Hello</p>".into(),
        };
        let result = HtmlRenderer::render(&content);
        assert!(result.contains("<p>Hello</p>"));
        assert!(result.contains("tailwindcss"));
    }

    #[test]
    fn render_table_output() {
        let content = CanvasContent::Table {
            headers: vec!["Name".into(), "Value".into()],
            rows: vec![vec!["A".into(), "1".into()]],
            title: Some("Test Table".into()),
        };
        let result = HtmlRenderer::render(&content);
        assert!(result.contains("Test Table"));
        assert!(result.contains("<table"));
        assert!(result.contains("Name"));
    }

    #[test]
    fn render_dashboard_widgets() {
        let content = CanvasContent::Dashboard {
            widgets: vec![
                DashboardWidget::Metric {
                    label: "Users".into(),
                    value: "1,234".into(),
                    trend: Some("+12%".into()),
                },
                DashboardWidget::Status {
                    label: "API".into(),
                    ok: true,
                },
            ],
        };
        let result = HtmlRenderer::render(&content);
        assert!(result.contains("Users"));
        assert!(result.contains("1,234"));
        assert!(result.contains("+12%"));
        assert!(result.contains("green"));
    }

    #[test]
    fn render_form_fields() {
        let content = CanvasContent::Form {
            fields: vec![FormField {
                name: "email".into(),
                label: "Email".into(),
                field_type: "text".into(),
                required: true,
                options: None,
                default_value: None,
            }],
            submit_label: "Go".into(),
            action_id: "signup".into(),
        };
        let result = HtmlRenderer::render(&content);
        assert!(result.contains("email"));
        assert!(result.contains("required"));
        assert!(result.contains("Go"));
        assert!(result.contains("handleSubmit"));
    }

    #[test]
    fn render_markdown_headings() {
        let content = CanvasContent::Markdown {
            markdown: "# Title\n## Subtitle\n- Item 1\n- Item 2".into(),
        };
        let result = HtmlRenderer::render(&content);
        assert!(result.contains("<h1"));
        assert!(result.contains("Title"));
        assert!(result.contains("<h2"));
        assert!(result.contains("<li"));
    }
}
