pub mod traits;

mod architect;
mod bizdev;
mod engineer;
mod khmer_expert;
mod marketing;
mod researcher;
mod sales;
mod ui_ux;

pub use architect::SystemArchitectPersona;
pub use bizdev::BizDevPersona;
pub use engineer::FullstackEngineerPersona;
pub use khmer_expert::KhmerExpertPersona;
pub use marketing::MarketingPersona;
pub use researcher::ResearcherPersona;
pub use sales::SalesPersona;
pub use traits::{Persona, PersonaRole};
pub use ui_ux::UiUxExpertPersona;

/// Create a persona by role name.
pub fn create_persona(role: &str) -> Option<Box<dyn Persona>> {
    match role {
        "architect" | "system_architect" | "archon" => Some(Box::new(SystemArchitectPersona)),
        "ui_ux" | "designer" | "prism" => Some(Box::new(UiUxExpertPersona)),
        "engineer" | "fullstack" | "developer" | "forge" => {
            Some(Box::new(FullstackEngineerPersona))
        }
        "bizdev" | "business" | "nexus" => Some(Box::new(BizDevPersona)),
        "marketing" | "echo" => Some(Box::new(MarketingPersona)),
        "sales" | "closer" => Some(Box::new(SalesPersona)),
        "researcher" | "research" | "oracle" => Some(Box::new(ResearcherPersona)),
        "khmer" | "khmer_expert" | "translator" | "veasna" => Some(Box::new(KhmerExpertPersona)),
        _ => None,
    }
}

/// List all available persona roles.
pub fn available_roles() -> Vec<PersonaRole> {
    vec![
        PersonaRole::SystemArchitect,
        PersonaRole::UiUxExpert,
        PersonaRole::FullstackEngineer,
        PersonaRole::BizDev,
        PersonaRole::Marketing,
        PersonaRole::Sales,
        PersonaRole::Researcher,
        PersonaRole::KhmerExpert,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_all_personas() {
        let roles = [
            "architect",
            "ui_ux",
            "engineer",
            "bizdev",
            "marketing",
            "sales",
            "researcher",
            "khmer",
        ];
        for role in &roles {
            let persona = create_persona(role);
            assert!(persona.is_some(), "Failed to create persona for '{role}'");
        }
    }

    #[test]
    fn unknown_role_returns_none() {
        assert!(create_persona("wizard").is_none());
    }

    #[test]
    fn aliases_work() {
        assert!(create_persona("system_architect").is_some());
        assert!(create_persona("designer").is_some());
        assert!(create_persona("fullstack").is_some());
        assert!(create_persona("developer").is_some());
        assert!(create_persona("business").is_some());
        assert!(create_persona("research").is_some());
        assert!(create_persona("khmer_expert").is_some());
        assert!(create_persona("translator").is_some());
    }

    #[test]
    fn available_roles_has_eight() {
        assert_eq!(available_roles().len(), 8);
    }
}
