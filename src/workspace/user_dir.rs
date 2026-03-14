use std::path::{Path, PathBuf};

/// Represents a single user's workspace within ZeroClaw.
///
/// Each user gets their own directory tree under the global workspace:
/// ```text
/// ~/.zeroclaw/users/
///   user-a/
///     chats/          # Chat history
///     dir-app-00/     # First project workspace
///     dir-app-01/     # Second project workspace
///   user-b/
///     chats/
///     dir-app-00/
/// ```
#[derive(Debug, Clone)]
pub struct UserWorkspace {
    /// Canonical user identifier (normalized from channel identity).
    pub user_id: String,
    /// Root directory for this user's data.
    pub root: PathBuf,
}

impl UserWorkspace {
    /// Create a new user workspace descriptor.
    pub fn new(users_root: &Path, user_id: &str) -> Self {
        let normalized = normalize_user_id(user_id);
        Self {
            user_id: normalized.clone(),
            root: users_root.join(&normalized),
        }
    }

    /// Path to this user's chat history directory.
    pub fn chats_dir(&self) -> PathBuf {
        self.root.join("chats")
    }

    /// Path to a specific project workspace for this user.
    /// Project index is auto-incremented (dir-app-00, dir-app-01, ...).
    pub fn project_dir(&self, index: u32) -> PathBuf {
        self.root.join(format!("dir-app-{index:02}"))
    }

    /// List existing project directories for this user (sorted).
    pub fn list_projects(&self) -> Vec<PathBuf> {
        let mut projects = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&self.root) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with("dir-app-") && entry.path().is_dir() {
                    projects.push(entry.path());
                }
            }
        }
        projects.sort();
        projects
    }

    /// Get the next available project directory index.
    pub fn next_project_index(&self) -> u32 {
        let projects = self.list_projects();
        if projects.is_empty() {
            return 0;
        }
        projects
            .last()
            .and_then(|p| {
                p.file_name()?
                    .to_string_lossy()
                    .strip_prefix("dir-app-")?
                    .parse::<u32>()
                    .ok()
            })
            .map_or(0, |n| n + 1)
    }

    /// Create a new project workspace and return its path.
    pub fn create_project(&self) -> std::io::Result<PathBuf> {
        let index = self.next_project_index();
        let dir = self.project_dir(index);
        std::fs::create_dir_all(&dir)?;
        Ok(dir)
    }

    /// Ensure the user's root and chats directories exist.
    pub fn ensure_dirs(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.root)?;
        std::fs::create_dir_all(self.chats_dir())?;
        Ok(())
    }

    /// Path to a specific chat session file.
    pub fn chat_file(&self, session_id: &str) -> PathBuf {
        self.chats_dir()
            .join(format!("{}.jsonl", sanitize_filename(session_id)))
    }
}

/// Normalize a user identity from any channel into a safe directory name.
/// Handles: Telegram usernames, Discord IDs, email addresses, etc.
fn normalize_user_id(raw: &str) -> String {
    let trimmed = raw.trim().to_lowercase();
    // Replace @ and other special chars with hyphens.
    let normalized: String = trimmed
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect();
    // Collapse multiple hyphens and trim.
    let mut result = String::new();
    let mut prev_hyphen = false;
    for c in normalized.chars() {
        if c == '-' {
            if !prev_hyphen && !result.is_empty() {
                result.push(c);
            }
            prev_hyphen = true;
        } else {
            result.push(c);
            prev_hyphen = false;
        }
    }
    // Trim trailing hyphen and limit length.
    let result = result.trim_end_matches('-').to_string();
    if result.is_empty() {
        "anonymous".to_string()
    } else {
        result.chars().take(64).collect()
    }
}

/// Sanitize a string for use as a filename.
fn sanitize_filename(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn normalize_user_id_handles_various_formats() {
        assert_eq!(normalize_user_id("@telegram_user"), "telegram_user");
        assert_eq!(normalize_user_id("user@email.com"), "user-email-com");
        assert_eq!(normalize_user_id("Discord#1234"), "discord-1234");
        assert_eq!(normalize_user_id("  SpAcEs  "), "spaces");
        assert_eq!(normalize_user_id(""), "anonymous");
        assert_eq!(normalize_user_id("@@@"), "anonymous");
    }

    #[test]
    fn user_workspace_paths() {
        let ws = UserWorkspace::new(Path::new("/data/users"), "alice");
        assert_eq!(ws.root, PathBuf::from("/data/users/alice"));
        assert_eq!(ws.chats_dir(), PathBuf::from("/data/users/alice/chats"));
        assert_eq!(
            ws.project_dir(0),
            PathBuf::from("/data/users/alice/dir-app-00")
        );
        assert_eq!(
            ws.project_dir(5),
            PathBuf::from("/data/users/alice/dir-app-05")
        );
    }

    #[test]
    fn chat_file_path() {
        let ws = UserWorkspace::new(Path::new("/data/users"), "bob");
        let path = ws.chat_file("session-123");
        assert_eq!(
            path,
            PathBuf::from("/data/users/bob/chats/session-123.jsonl")
        );
    }

    #[test]
    fn sanitize_filename_handles_special_chars() {
        assert_eq!(sanitize_filename("hello world!"), "hello_world_");
        assert_eq!(sanitize_filename("file/name"), "file_name");
        assert_eq!(sanitize_filename("ok.jsonl"), "ok.jsonl");
    }
}
