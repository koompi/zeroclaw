use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

use super::user_dir::UserWorkspace;

/// Manages multi-user workspace isolation.
///
/// The `WorkspaceManager` is the central registry that maps user identities
/// (from any channel: Telegram, Discord, Slack, etc.) to isolated workspace
/// directories. Each user gets:
///
/// - Their own root directory under `~/.zeroclaw/users/<user-id>/`
/// - Isolated chat history in `chats/`
/// - Per-project workspaces in `dir-app-00/`, `dir-app-01/`, etc.
///
/// Thread-safe and designed for concurrent multi-user access.
#[derive(Clone)]
pub struct WorkspaceManager {
    /// Root directory for all user workspaces (e.g. `~/.zeroclaw/users/`).
    users_root: PathBuf,
    /// Cache of active user workspaces.
    cache: Arc<RwLock<HashMap<String, UserWorkspace>>>,
}

impl WorkspaceManager {
    /// Create a new workspace manager rooted at the given directory.
    pub fn new(users_root: PathBuf) -> Self {
        Self {
            users_root,
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create from the global ZeroClaw workspace directory.
    /// Users will be stored under `<workspace>/users/`.
    pub fn from_workspace(workspace_dir: &Path) -> Self {
        Self::new(workspace_dir.join("users"))
    }

    /// Get or create a user workspace for the given identity.
    ///
    /// The `user_identity` can be any channel-specific identifier:
    /// - Telegram: `@username` or numeric user ID
    /// - Discord: `username#1234` or numeric user ID
    /// - Slack: `U01234ABCDE`
    /// - Email: `user@example.com`
    ///
    /// The identity is normalized into a safe directory name.
    pub async fn get_or_create(&self, user_identity: &str) -> std::io::Result<UserWorkspace> {
        // Check cache first.
        {
            let cache = self.cache.read().await;
            if let Some(ws) = cache.get(user_identity) {
                return Ok(ws.clone());
            }
        }

        // Create new workspace.
        let ws = UserWorkspace::new(&self.users_root, user_identity);
        ws.ensure_dirs()?;

        // Cache it.
        {
            let mut cache = self.cache.write().await;
            cache.insert(user_identity.to_string(), ws.clone());
        }

        tracing::info!(
            user = %ws.user_id,
            root = %ws.root.display(),
            "Created user workspace"
        );

        Ok(ws)
    }

    /// List all known users (from disk, not just cache).
    pub fn list_users(&self) -> Vec<String> {
        let mut users = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&self.users_root) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    if let Some(name) = entry.file_name().to_str() {
                        users.push(name.to_string());
                    }
                }
            }
        }
        users.sort();
        users
    }

    /// Get the workspace for a known user (from cache or disk).
    pub async fn get(&self, user_identity: &str) -> Option<UserWorkspace> {
        // Check cache.
        {
            let cache = self.cache.read().await;
            if let Some(ws) = cache.get(user_identity) {
                return Some(ws.clone());
            }
        }

        // Try from disk.
        let ws = UserWorkspace::new(&self.users_root, user_identity);
        if ws.root.exists() {
            let mut cache = self.cache.write().await;
            cache.insert(user_identity.to_string(), ws.clone());
            Some(ws)
        } else {
            None
        }
    }

    /// Create a new project workspace for the given user and return its path.
    pub async fn create_project_for_user(
        &self,
        user_identity: &str,
    ) -> std::io::Result<PathBuf> {
        let ws = self.get_or_create(user_identity).await?;
        ws.create_project()
    }

    /// Get the chat file path for a user's session.
    pub async fn chat_file(
        &self,
        user_identity: &str,
        session_id: &str,
    ) -> std::io::Result<PathBuf> {
        let ws = self.get_or_create(user_identity).await?;
        Ok(ws.chat_file(session_id))
    }

    /// Get the users root directory.
    pub fn users_root(&self) -> &Path {
        &self.users_root
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn get_or_create_makes_dirs() {
        let tmp = TempDir::new().unwrap();
        let mgr = WorkspaceManager::new(tmp.path().to_path_buf());

        let ws = mgr.get_or_create("@alice").await.unwrap();
        assert!(ws.root.exists());
        assert!(ws.chats_dir().exists());
        assert_eq!(ws.user_id, "alice");
    }

    #[tokio::test]
    async fn get_or_create_is_idempotent() {
        let tmp = TempDir::new().unwrap();
        let mgr = WorkspaceManager::new(tmp.path().to_path_buf());

        let ws1 = mgr.get_or_create("bob").await.unwrap();
        let ws2 = mgr.get_or_create("bob").await.unwrap();
        assert_eq!(ws1.root, ws2.root);
    }

    #[tokio::test]
    async fn create_project_auto_increments() {
        let tmp = TempDir::new().unwrap();
        let mgr = WorkspaceManager::new(tmp.path().to_path_buf());

        let p0 = mgr.create_project_for_user("charlie").await.unwrap();
        assert!(p0.ends_with("dir-app-00"));
        assert!(p0.exists());

        let p1 = mgr.create_project_for_user("charlie").await.unwrap();
        assert!(p1.ends_with("dir-app-01"));
        assert!(p1.exists());
    }

    #[tokio::test]
    async fn list_users_from_disk() {
        let tmp = TempDir::new().unwrap();
        let mgr = WorkspaceManager::new(tmp.path().to_path_buf());

        mgr.get_or_create("alice").await.unwrap();
        mgr.get_or_create("bob").await.unwrap();
        mgr.get_or_create("charlie").await.unwrap();

        let users = mgr.list_users();
        assert_eq!(users, vec!["alice", "bob", "charlie"]);
    }

    #[tokio::test]
    async fn chat_file_path_correct() {
        let tmp = TempDir::new().unwrap();
        let mgr = WorkspaceManager::new(tmp.path().to_path_buf());

        let path = mgr.chat_file("alice", "session-001").await.unwrap();
        assert!(path.to_string_lossy().contains("alice/chats/session-001.jsonl"));
    }

    #[tokio::test]
    async fn different_users_isolated() {
        let tmp = TempDir::new().unwrap();
        let mgr = WorkspaceManager::new(tmp.path().to_path_buf());

        let ws_a = mgr.get_or_create("user-a").await.unwrap();
        let ws_b = mgr.get_or_create("user-b").await.unwrap();

        assert_ne!(ws_a.root, ws_b.root);

        // Each user's projects are independent.
        let pa = mgr.create_project_for_user("user-a").await.unwrap();
        let pb = mgr.create_project_for_user("user-b").await.unwrap();

        assert!(pa.to_string_lossy().contains("user-a"));
        assert!(pb.to_string_lossy().contains("user-b"));
    }

    #[tokio::test]
    async fn get_nonexistent_returns_none() {
        let tmp = TempDir::new().unwrap();
        let mgr = WorkspaceManager::new(tmp.path().to_path_buf());

        assert!(mgr.get("nobody").await.is_none());
    }
}
