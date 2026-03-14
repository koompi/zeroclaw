use serde::{Deserialize, Serialize};
use std::fmt;

/// Enumeration of all supported persona roles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PersonaRole {
    SystemArchitect,
    UiUxExpert,
    FullstackEngineer,
    BizDev,
    Marketing,
    Sales,
    Researcher,
    KhmerExpert,
}

impl fmt::Display for PersonaRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SystemArchitect => write!(f, "system_architect"),
            Self::UiUxExpert => write!(f, "ui_ux_expert"),
            Self::FullstackEngineer => write!(f, "fullstack_engineer"),
            Self::BizDev => write!(f, "bizdev"),
            Self::Marketing => write!(f, "marketing"),
            Self::Sales => write!(f, "sales"),
            Self::Researcher => write!(f, "researcher"),
            Self::KhmerExpert => write!(f, "khmer_expert"),
        }
    }
}

/// Cluster grouping for personas.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cluster {
    /// Technical/Builder cluster: architect, UI/UX, engineer.
    Builder,
    /// Business cluster: bizdev, marketing, sales.
    Business,
    /// Research cluster: researcher.
    Research,
}

/// Core persona trait. Each persona defines its identity, system prompt,
/// tool access, and constraints.
pub trait Persona: Send + Sync {
    /// Persona role identifier.
    fn role(&self) -> PersonaRole;

    /// Human-readable display name.
    fn display_name(&self) -> &str;

    /// Which cluster this persona belongs to.
    fn cluster(&self) -> Cluster;

    /// System prompt injected into the agent's context.
    /// This shapes the agent's behavior, expertise, and communication style.
    fn system_prompt(&self) -> String;

    /// List of tool names this persona should have access to.
    /// Empty means "inherit all tools from parent".
    fn allowed_tools(&self) -> Vec<String> {
        Vec::new()
    }

    /// List of tool names this persona should NOT have access to.
    fn denied_tools(&self) -> Vec<String> {
        Vec::new()
    }

    /// Suggested model for this persona (e.g. fast model for simple tasks).
    /// None means "use the default model".
    fn suggested_model(&self) -> Option<&str> {
        None
    }

    /// Suggested temperature for this persona.
    fn suggested_temperature(&self) -> Option<f64> {
        None
    }

    /// Maximum tool iterations for this persona's agent loop.
    fn max_iterations(&self) -> usize {
        10
    }

    /// Model to use when this persona spawns Claude Code sessions.
    /// Cost-aware: lighter tasks use cheaper models, heavy implementation uses capable ones.
    /// None means "use the persona's suggested_model or default".
    fn claude_code_model(&self) -> Option<&str> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestPersona;
    impl Persona for TestPersona {
        fn role(&self) -> PersonaRole {
            PersonaRole::Researcher
        }
        fn display_name(&self) -> &str {
            "Test Researcher"
        }
        fn cluster(&self) -> Cluster {
            Cluster::Research
        }
        fn system_prompt(&self) -> String {
            "You are a test researcher.".into()
        }
    }

    #[test]
    fn default_trait_methods() {
        let p = TestPersona;
        assert!(p.allowed_tools().is_empty());
        assert!(p.denied_tools().is_empty());
        assert!(p.suggested_model().is_none());
        assert!(p.suggested_temperature().is_none());
        assert_eq!(p.max_iterations(), 10);
    }

    #[test]
    fn role_display() {
        assert_eq!(PersonaRole::SystemArchitect.to_string(), "system_architect");
        assert_eq!(PersonaRole::FullstackEngineer.to_string(), "fullstack_engineer");
    }
}
