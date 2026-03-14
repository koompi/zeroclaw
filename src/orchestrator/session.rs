use serde::{Deserialize, Serialize};
use std::fmt;

/// Session type discriminator within the orchestrator.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionType {
    /// Primary agent session.
    Main,
    /// Spawned subagent session.
    Subagent,
    /// Scheduled/cron-triggered session.
    Cron,
}

impl fmt::Display for SessionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Main => write!(f, "main"),
            Self::Subagent => write!(f, "subagent"),
            Self::Cron => write!(f, "cron"),
        }
    }
}

impl SessionType {
    pub fn from_str_loose(s: &str) -> Option<Self> {
        match s {
            "main" => Some(Self::Main),
            "subagent" => Some(Self::Subagent),
            "cron" => Some(Self::Cron),
            _ => None,
        }
    }
}

/// Deterministic session key following the pattern `agent:<id>:<type>:<uuid>`.
///
/// Inspired by OpenClaw's session key system — allows stateless key generation,
/// hierarchical routing, and parent-child relationship tracking.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionKey {
    /// Agent identifier (e.g. "main", "researcher", "coder").
    pub agent_id: String,
    /// Session type.
    pub session_type: SessionType,
    /// Unique instance identifier (UUID v4).
    pub instance_id: String,
}

impl SessionKey {
    /// Create a new session key with a random UUID.
    pub fn new(agent_id: &str, session_type: SessionType) -> Self {
        Self {
            agent_id: normalize_agent_id(agent_id),
            session_type,
            instance_id: generate_id(),
        }
    }

    /// Create a main session key for the given agent.
    pub fn main(agent_id: &str) -> Self {
        Self::new(agent_id, SessionType::Main)
    }

    /// Create a subagent session key.
    pub fn subagent(agent_id: &str) -> Self {
        Self::new(agent_id, SessionType::Subagent)
    }

    /// Create a cron session key.
    pub fn cron(agent_id: &str) -> Self {
        Self::new(agent_id, SessionType::Cron)
    }

    /// Parse a session key from its canonical string representation.
    pub fn parse(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.splitn(4, ':').collect();
        if parts.len() != 4 || parts[0] != "agent" {
            return None;
        }
        let agent_id = parts[1].to_string();
        let session_type = SessionType::from_str_loose(parts[2])?;
        let instance_id = parts[3].to_string();

        if !is_valid_agent_id(&agent_id) || instance_id.is_empty() {
            return None;
        }

        Some(Self {
            agent_id,
            session_type,
            instance_id,
        })
    }

    /// Canonical string representation.
    pub fn to_key(&self) -> String {
        format!(
            "agent:{}:{}:{}",
            self.agent_id, self.session_type, self.instance_id
        )
    }

    /// Check if this session is a child of the given parent agent.
    pub fn is_child_of(&self, parent_agent_id: &str) -> bool {
        self.agent_id == normalize_agent_id(parent_agent_id)
            && self.session_type == SessionType::Subagent
    }
}

impl fmt::Display for SessionKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_key())
    }
}

/// Normalize an agent ID to lowercase, alphanumeric + hyphens/underscores.
fn normalize_agent_id(id: &str) -> String {
    let normalized: String = id
        .trim()
        .to_lowercase()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .collect();
    if normalized.is_empty() {
        "main".to_string()
    } else {
        // Truncate to 64 chars max.
        normalized.chars().take(64).collect()
    }
}

/// Validate an agent ID (already normalized).
fn is_valid_agent_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 64
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        && id
            .chars()
            .next()
            .map_or(false, |c| c.is_ascii_alphanumeric())
}

/// Generate a short unique ID for internal tracking.
/// NOTE: Not cryptographically random — uses timestamp + PID + counter.
/// Sufficient for session tracking; do NOT use as a security token.
fn generate_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    let pid = std::process::id();
    // Mix time + pid + a counter for uniqueness within same nanosecond.
    static COUNTER: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    let count = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    format!("{:08x}{:04x}{:04x}", nanos, pid & 0xFFFF, count & 0xFFFF)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_key_roundtrip() {
        let key = SessionKey::new("researcher", SessionType::Subagent);
        let serialized = key.to_key();
        assert!(serialized.starts_with("agent:researcher:subagent:"));
        let parsed = SessionKey::parse(&serialized).unwrap();
        assert_eq!(parsed.agent_id, "researcher");
        assert_eq!(parsed.session_type, SessionType::Subagent);
        assert_eq!(parsed.instance_id, key.instance_id);
    }

    #[test]
    fn session_key_parse_invalid() {
        assert!(SessionKey::parse("invalid").is_none());
        assert!(SessionKey::parse("agent:foo:bar").is_none());
        assert!(SessionKey::parse("notagent:foo:main:123").is_none());
        assert!(SessionKey::parse("agent::main:123").is_none());
    }

    #[test]
    fn normalize_agent_id_handles_edge_cases() {
        assert_eq!(normalize_agent_id("  MyAgent  "), "myagent");
        assert_eq!(normalize_agent_id("agent@123!"), "agent123");
        assert_eq!(normalize_agent_id(""), "main");
        assert_eq!(normalize_agent_id("valid-id_01"), "valid-id_01");
    }

    #[test]
    fn session_key_main_convenience() {
        let key = SessionKey::main("primary");
        assert_eq!(key.agent_id, "primary");
        assert_eq!(key.session_type, SessionType::Main);
    }

    #[test]
    fn is_child_of_checks_agent_and_type() {
        let child = SessionKey::subagent("worker");
        assert!(child.is_child_of("worker"));
        assert!(!child.is_child_of("other"));

        let main = SessionKey::main("worker");
        assert!(!main.is_child_of("worker"));
    }

    #[test]
    fn generate_id_is_unique() {
        let a = generate_id();
        let b = generate_id();
        assert_ne!(a, b);
    }
}
