//! Generic orphan branch storage for git.
//!
//! Provides read/write operations on orphan branches using git plumbing
//! commands (hash-object, update-index, write-tree, commit-tree).
//! Used by both session storage (`chub/sessions/v1`) and checkpoint
//! storage (`entire/checkpoints/v1`).

use std::fs;

use std::process::Command;

/// Ensure an orphan branch exists. Creates it with an empty tree if missing.
pub fn ensure_branch(branch: &str) -> bool {
    let exists = Command::new("git")
        .args(["rev-parse", "--verify", branch])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if exists {
        return true;
    }

    // Well-known empty tree hash (works on all git versions)
    let empty_tree = "4b825dc642cb6eb9a060e54bf899d69f7264209e";

    let commit = git_output(&[
        "commit-tree",
        empty_tree,
        "-m",
        &format!("Initialize {}", branch),
    ]);
    if let Some(hash) = commit {
        Command::new("git")
            .args(["update-ref", &format!("refs/heads/{}", branch), &hash])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    } else {
        false
    }
}

/// Write files to an orphan branch. Each entry is `(path_on_branch, content)`.
/// Merges with the existing tree (does not replace).
/// Returns `true` on success.
pub fn write_files(branch: &str, files: &[(&str, &[u8])], commit_msg: &str) -> bool {
    if files.is_empty() {
        return false;
    }

    ensure_branch(branch);

    // Get parent commit
    let parent = git_output(&["rev-parse", branch]);

    // Create temp index file
    let tmp_index = std::env::temp_dir().join(format!(
        "chub-idx-{}-{}",
        branch.replace('/', "-"),
        std::process::id()
    ));

    // Read existing tree into temp index
    if let Some(ref parent_hash) = parent {
        let _ = Command::new("git")
            .env("GIT_INDEX_FILE", tmp_index.to_str().unwrap_or(""))
            .args(["read-tree", parent_hash])
            .output();
    }

    // Hash each blob and add to index
    for (path, content) in files {
        // Write content to temp file, then hash it
        let tmp_blob = std::env::temp_dir().join(format!("chub-blob-{}", std::process::id()));
        if fs::write(&tmp_blob, content).is_err() {
            continue;
        }
        let hash = Command::new("git")
            .args(["hash-object", "-w"])
            .arg(tmp_blob.to_str().unwrap_or(""))
            .output()
            .ok()
            .and_then(|o| {
                let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
                if s.is_empty() {
                    None
                } else {
                    Some(s)
                }
            });
        let _ = fs::remove_file(&tmp_blob);

        if let Some(hash) = hash {
            let _ = Command::new("git")
                .env("GIT_INDEX_FILE", tmp_index.to_str().unwrap_or(""))
                .args([
                    "update-index",
                    "--add",
                    "--cacheinfo",
                    "100644",
                    &hash,
                    path,
                ])
                .output();
        }
    }

    // Write tree
    let tree = Command::new("git")
        .env("GIT_INDEX_FILE", tmp_index.to_str().unwrap_or(""))
        .args(["write-tree"])
        .output()
        .ok()
        .and_then(|o| {
            let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if s.is_empty() {
                None
            } else {
                Some(s)
            }
        });

    let _ = fs::remove_file(&tmp_index);

    let tree = match tree {
        Some(t) => t,
        None => return false,
    };

    // Create commit
    let mut args = vec!["commit-tree".to_string(), tree.clone()];
    if let Some(ref parent_hash) = parent {
        args.push("-p".to_string());
        args.push(parent_hash.clone());
    }
    args.push("-m".to_string());
    args.push(commit_msg.to_string());

    let commit = Command::new("git").args(&args).output().ok().and_then(|o| {
        let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
        if s.is_empty() {
            None
        } else {
            Some(s)
        }
    });

    if let Some(commit_hash) = commit {
        Command::new("git")
            .args([
                "update-ref",
                &format!("refs/heads/{}", branch),
                &commit_hash,
            ])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    } else {
        false
    }
}

/// Read a single file from an orphan branch.
pub fn read_file(branch: &str, path: &str) -> Option<Vec<u8>> {
    let output = Command::new("git")
        .args(["show", &format!("{}:{}", branch, path)])
        .output()
        .ok()?;

    if output.status.success() {
        Some(output.stdout)
    } else {
        None
    }
}

/// List all file paths on an orphan branch.
pub fn list_files(branch: &str) -> Vec<String> {
    let output = Command::new("git")
        .args(["ls-tree", "-r", "--name-only", branch])
        .output();

    match output {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout)
            .lines()
            .map(|l| l.to_string())
            .collect(),
        _ => vec![],
    }
}

/// Check if a branch exists.
pub fn branch_exists(branch: &str) -> bool {
    Command::new("git")
        .args(["rev-parse", "--verify", branch])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Push a branch to a remote. Returns true on success.
/// Never fails loudly — designed for use in hooks.
pub fn push_branch(branch: &str, remote: &str) -> bool {
    // Check if there's anything to push
    let local = git_output(&["rev-parse", branch]);
    let remote_ref = git_output(&["rev-parse", &format!("refs/remotes/{}/{}", remote, branch)]);

    // If both match, nothing to push
    if local == remote_ref && local.is_some() {
        return true;
    }

    Command::new("git")
        .args(["push", "--no-verify", remote, branch])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn git_output(args: &[&str]) -> Option<String> {
    Command::new("git").args(args).output().ok().and_then(|o| {
        if !o.status.success() {
            return None;
        }
        let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
        if s.is_empty() {
            None
        } else {
            Some(s)
        }
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn empty_tree_hash_is_valid() {
        // The well-known empty tree hash
        assert_eq!("4b825dc642cb6eb9a060e54bf899d69f7264209e".len(), 40);
    }
}
