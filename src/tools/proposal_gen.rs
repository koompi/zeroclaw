use super::traits::{Tool, ToolResult};
use async_trait::async_trait;
use serde_json::json;

/// Tool for generating structured business documents: proposals,
/// pitch decks, competitive analyses, and campaign briefs.
///
/// Used by the Business cluster personas (BizDev, Marketing, Sales).
pub struct ProposalGenTool;

impl ProposalGenTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for ProposalGenTool {
    fn name(&self) -> &str {
        "proposal_gen"
    }

    fn description(&self) -> &str {
        "Generate structured business documents. Types: 'proposal' (partnership/project proposal), \
         'pitch' (pitch deck outline), 'analysis' (competitive/market analysis), \
         'campaign' (marketing campaign brief), 'outreach' (sales outreach sequence). \
         Returns structured markdown output."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "additionalProperties": false,
            "properties": {
                "doc_type": {
                    "type": "string",
                    "enum": ["proposal", "pitch", "analysis", "campaign", "outreach"],
                    "description": "Type of business document to generate"
                },
                "subject": {
                    "type": "string",
                    "minLength": 1,
                    "description": "Subject or topic (e.g. company name, product, market)"
                },
                "context": {
                    "type": "string",
                    "description": "Additional context, requirements, or constraints"
                },
                "audience": {
                    "type": "string",
                    "description": "Target audience for the document"
                }
            },
            "required": ["doc_type", "subject"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        let doc_type = args
            .get("doc_type")
            .and_then(|v| v.as_str())
            .unwrap_or("proposal");
        let subject = args
            .get("subject")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let context = args
            .get("context")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let audience = args
            .get("audience")
            .and_then(|v| v.as_str())
            .unwrap_or("decision-makers");

        if subject.trim().is_empty() {
            return Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some("'subject' is required".into()),
            });
        }

        let template = match doc_type {
            "proposal" => format_proposal(subject, context, audience),
            "pitch" => format_pitch(subject, context, audience),
            "analysis" => format_analysis(subject, context),
            "campaign" => format_campaign(subject, context, audience),
            "outreach" => format_outreach(subject, context, audience),
            _ => {
                return Ok(ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some(format!("Unknown doc_type: {doc_type}")),
                });
            }
        };

        Ok(ToolResult {
            success: true,
            output: template,
            error: None,
        })
    }
}

fn format_proposal(subject: &str, context: &str, audience: &str) -> String {
    let ctx_section = if context.is_empty() {
        String::new()
    } else {
        format!("\n## Background\n{context}\n")
    };

    format!(
        r#"# Proposal: {subject}
**Audience:** {audience}
**Date:** {{{{current_date}}}}
{ctx_section}
## Executive Summary
[Concise overview of the proposal — problem, solution, expected outcome]

## Problem Statement
[What problem does this solve? Why now?]

## Proposed Solution
[Detailed description of what we're proposing]

## Value Proposition
- [Benefit 1]
- [Benefit 2]
- [Benefit 3]

## Implementation Plan
| Phase | Timeline | Deliverable |
|-------|----------|-------------|
| Phase 1 | Week 1-2 | [Deliverable] |
| Phase 2 | Week 3-4 | [Deliverable] |
| Phase 3 | Week 5-6 | [Deliverable] |

## Investment
[Pricing, resource requirements, or partnership terms]

## Next Steps
1. [Action item with owner and deadline]
2. [Action item with owner and deadline]
3. [Action item with owner and deadline]"#
    )
}

fn format_pitch(subject: &str, context: &str, audience: &str) -> String {
    format!(
        r#"# Pitch Deck: {subject}
**Target Audience:** {audience}

## Slide 1: Hook
[One-sentence problem statement that resonates with {audience}]

## Slide 2: Problem
[The pain point — quantified with data if possible]

## Slide 3: Solution
[What we built/propose — clear and concise]

## Slide 4: How It Works
[3-step explanation or demo flow]

## Slide 5: Traction / Proof
[Metrics, testimonials, case studies, or milestones]

## Slide 6: Market Opportunity
[TAM/SAM/SOM with sources]

## Slide 7: Business Model
[How we make money / partnership structure]

## Slide 8: Team
[Key people and relevant expertise]

## Slide 9: Ask
[What we need: funding, partnership, pilot, etc.]

## Slide 10: Contact
[How to follow up]

{context}"#
    )
}

fn format_analysis(subject: &str, context: &str) -> String {
    format!(
        r#"# Competitive Analysis: {subject}

## Overview
[Market context and analysis scope]
{context}

## Competitor Matrix

| Feature | Us | Competitor A | Competitor B | Competitor C |
|---------|-----|-------------|-------------|-------------|
| [Feature 1] | | | | |
| [Feature 2] | | | | |
| Pricing | | | | |
| Target Market | | | | |

## SWOT Analysis

### Strengths
- [Our advantages]

### Weaknesses
- [Our gaps]

### Opportunities
- [Market opportunities]

### Threats
- [Competitive risks]

## Strategic Recommendations
1. [Recommendation with rationale]
2. [Recommendation with rationale]
3. [Recommendation with rationale]

## Sources
- [Source 1]
- [Source 2]"#
    )
}

fn format_campaign(subject: &str, context: &str, audience: &str) -> String {
    format!(
        r#"# Campaign Brief: {subject}
**Target Audience:** {audience}

## Objective
[What does this campaign aim to achieve? Be specific and measurable.]
{context}

## Key Messages
1. [Primary message]
2. [Supporting message]
3. [Supporting message]

## Channels
| Channel | Content Type | Frequency | Owner |
|---------|-------------|-----------|-------|
| Blog | Long-form | 2x/month | |
| Twitter/X | Short-form | Daily | |
| Email | Newsletter | Weekly | |
| LinkedIn | Thought leadership | 2x/week | |

## Timeline
| Week | Activity | Deliverable |
|------|----------|-------------|
| 1 | [Activity] | [Deliverable] |
| 2 | [Activity] | [Deliverable] |
| 3 | [Activity] | [Deliverable] |
| 4 | [Activity] | [Deliverable] |

## KPIs
- [Metric 1]: [Target]
- [Metric 2]: [Target]
- [Metric 3]: [Target]

## Budget
[Budget allocation by channel]"#
    )
}

fn format_outreach(subject: &str, context: &str, audience: &str) -> String {
    format!(
        r#"# Outreach Sequence: {subject}
**Target:** {audience}
{context}

## Email 1: Cold Introduction (Day 0)
**Subject:** [Subject line — under 50 chars, personalized]
**Body:**
Hi [Name],

[1-2 sentences showing you've done research on their company/role]

[1 sentence on the specific problem you solve for people like them]

[1 sentence with a concrete proof point or result]

Would you be open to a 15-minute call this week?

Best,
[Your name]

---

## Email 2: Follow-up (Day 3)
**Subject:** Re: [Previous subject]
**Body:**
Hi [Name],

[Quick follow-up — add new value (case study, insight, relevant news)]

[Restate the ask with lower friction (async options)]

---

## Email 3: Break-up (Day 7)
**Subject:** Should I close your file?
**Body:**
Hi [Name],

I don't want to be a bother. If the timing isn't right, no worries.

If [problem] becomes a priority, here's where to find us: [link]

All the best,
[Your name]"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_metadata() {
        let tool = ProposalGenTool::new();
        assert_eq!(tool.name(), "proposal_gen");
        let schema = tool.parameters_schema();
        assert!(schema["properties"]["doc_type"].is_object());
    }

    #[tokio::test]
    async fn generate_proposal() {
        let tool = ProposalGenTool::new();
        let result = tool
            .execute(json!({
                "doc_type": "proposal",
                "subject": "AI Integration Partnership"
            }))
            .await
            .unwrap();
        assert!(result.success);
        assert!(result.output.contains("AI Integration Partnership"));
        assert!(result.output.contains("Executive Summary"));
    }

    #[tokio::test]
    async fn generate_outreach() {
        let tool = ProposalGenTool::new();
        let result = tool
            .execute(json!({
                "doc_type": "outreach",
                "subject": "Developer Tools",
                "audience": "CTOs at mid-stage startups"
            }))
            .await
            .unwrap();
        assert!(result.success);
        assert!(result.output.contains("CTOs at mid-stage startups"));
    }

    #[tokio::test]
    async fn empty_subject_rejected() {
        let tool = ProposalGenTool::new();
        let result = tool
            .execute(json!({"doc_type": "proposal", "subject": ""}))
            .await
            .unwrap();
        assert!(!result.success);
    }

    #[tokio::test]
    async fn invalid_doc_type_rejected() {
        let tool = ProposalGenTool::new();
        let result = tool
            .execute(json!({"doc_type": "unknown", "subject": "test"}))
            .await
            .unwrap();
        assert!(!result.success);
    }
}
