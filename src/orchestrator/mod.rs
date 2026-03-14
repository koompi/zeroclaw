pub mod announce;
pub mod depth;
pub mod policy;
pub mod registry;
pub mod session;
pub mod spawn;
pub mod traits;

#[cfg(test)]
mod tests;

pub use registry::SubagentRegistry;
pub use session::SessionKey;
pub use spawn::{SpawnMode, SpawnParams, SpawnResult};
pub use traits::Orchestrator;
