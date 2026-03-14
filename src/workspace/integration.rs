use std::path::{Path, PathBuf};

use super::manager::WorkspaceManager;

/// Resolve the effective workspace directory for a user.
///
/// When multi-user isolation is enabled (users_root exists), the workspace
/// is scoped to `~/.zeroclaw/users/<user-id>/`. Otherwise, the global
/// workspace is used.
///
/// This function is the bridge between the workspace isolation system and
/// the agent runtime. Call it during message processing to get the correct
/// workspace_dir before creating tools.
pub async fn resolve_user_workspace(
    global_workspace: &Path,
    sender_identity: Option<&str>,
) -> std::io::Result<PathBuf> {
    // If no sender identity, use global workspace (CLI mode, etc.)
    let sender = match sender_identity {
        Some(s) if !s.trim().is_empty() => s,
        _ => return Ok(global_workspace.to_path_buf()),
    };

    let mgr = WorkspaceManager::from_workspace(global_workspace);
    let user_ws = mgr.get_or_create(sender).await?;

    tracing::debug!(
        user = %user_ws.user_id,
        workspace = %user_ws.root.display(),
        "Resolved user workspace"
    );

    Ok(user_ws.root)
}

/// Check if a path is within the given workspace root.
///
/// Use this for additional path validation in tools beyond what
/// SecurityPolicy provides. Resolves symlinks and canonicalizes
/// both paths before comparison.
pub fn is_within_workspace(path: &Path, workspace_root: &Path) -> bool {
    // Try to canonicalize both paths for symlink-safe comparison.
    let canonical_root = workspace_root
        .canonicalize()
        .unwrap_or_else(|_| workspace_root.to_path_buf());

    let canonical_path = path
        .canonicalize()
        .unwrap_or_else(|_| path.to_path_buf());

    canonical_path.starts_with(&canonical_root)
}

/// Resolve a relative path against the workspace root.
/// Absolute paths are returned as-is (they'll be validated by SecurityPolicy).
pub fn resolve_path(path: &str, workspace_root: &Path) -> PathBuf {
    let p = Path::new(path);
    if p.is_absolute() {
        p.to_path_buf()
    } else {
        workspace_root.join(p)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn resolve_with_sender_creates_user_workspace() {
        let tmp = TempDir::new().unwrap();
        let result = resolve_user_workspace(tmp.path(), Some("@alice")).await.unwrap();
        assert!(result.exists());
        assert!(result.to_string_lossy().contains("alice"));
    }

    #[tokio::test]
    async fn resolve_without_sender_uses_global() {
        let tmp = TempDir::new().unwrap();
        let result = resolve_user_workspace(tmp.path(), None).await.unwrap();
        assert_eq!(result, tmp.path());
    }

    #[tokio::test]
    async fn resolve_with_empty_sender_uses_global() {
        let tmp = TempDir::new().unwrap();
        let result = resolve_user_workspace(tmp.path(), Some("")).await.unwrap();
        assert_eq!(result, tmp.path());
    }

    #[test]
    fn is_within_workspace_checks_correctly() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        // File inside workspace
        let inside = root.join("project/file.rs");
        std::fs::create_dir_all(root.join("project")).unwrap();
        std::fs::write(&inside, "test").unwrap();
        assert!(is_within_workspace(&inside, root));

        // File outside workspace
        assert!(!is_within_workspace(Path::new("/etc/passwd"), root));
    }

    #[test]
    fn resolve_path_handles_relative_and_absolute() {
        let root = Path::new("/home/user/workspace");
        assert_eq!(
            resolve_path("src/main.rs", root),
            PathBuf::from("/home/user/workspace/src/main.rs")
        );
        assert_eq!(
            resolve_path("/etc/passwd", root),
            PathBuf::from("/etc/passwd")
        );
    }
}
