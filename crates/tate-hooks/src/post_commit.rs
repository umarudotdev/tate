use std::path::Path;
use std::process::Command;

use tate_store::config::Config;
use tate_store::db::SqliteStore;
use tate_store::deck::{sync_deck, DeckFile};

pub fn run(repo_root: &Path) -> Result<(), String> {
    let tate_dir = repo_root.join(".tate");
    if !tate_dir.exists() {
        return Ok(());
    }

    let config = Config::load(&tate_dir.join("config"))
        .map_err(|e| format!("hook: failed to load config: {e}"))?;

    if !config.hooks.auto_add {
        return Ok(());
    }

    let msg_output = Command::new("git")
        .args(["log", "-1", "--format=%B", "HEAD"])
        .current_dir(repo_root)
        .output()
        .map_err(|e| format!("hook: failed to run git log: {e}"))?;

    let commit_msg = String::from_utf8_lossy(&msg_output.stdout);

    let matches = config.hooks.track_patterns.iter().any(|pattern| {
        regex::Regex::new(pattern)
            .map(|re| re.is_match(&commit_msg))
            .unwrap_or(false)
    });

    if !matches {
        return Ok(());
    }

    let diff_output = Command::new("git")
        .args(["diff", "--name-only", "HEAD~1", "HEAD"])
        .current_dir(repo_root)
        .output()
        .or_else(|_| {
            Command::new("git")
                .args([
                    "diff",
                    "--name-only",
                    "4b825dc642cb6eb9a060e54bf8d69288fbee4904",
                    "HEAD",
                ])
                .current_dir(repo_root)
                .output()
        })
        .map_err(|e| format!("hook: failed to run git diff: {e}"))?;

    let changed_files: Vec<&str> = String::from_utf8_lossy(&diff_output.stdout)
        .lines()
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .into_iter()
        .map(|s| s.to_owned())
        .collect::<Vec<_>>()
        .leak()
        .iter()
        .map(|s| s.as_str())
        .collect();

    let deck = DeckFile::new(tate_dir.join("deck"));
    let existing = deck.read().unwrap_or_default();
    let mut added = 0u32;

    for file in &changed_files {
        let path = Path::new(file);

        if file.starts_with(".tate/") || file.starts_with(".tate\\") {
            continue;
        }

        if !tate_symbols::resolver::is_supported(path) {
            continue;
        }

        let full_path = repo_root.join(path);
        if !full_path.exists() {
            continue;
        }

        if tate_symbols::resolver::supports_symbols(path) {
            if let Ok(symbols) = tate_symbols::resolver::list_symbols(&full_path) {
                for symbol in symbols {
                    let entry = format!("{}::{}", file, symbol);
                    if !existing.contains(&entry) && deck.append(&entry).is_ok() {
                        added += 1;
                    }
                }
                continue;
            }
        }

        let entry = file.to_string();
        if !existing.contains(&entry) && deck.append(&entry).is_ok() {
            added += 1;
        }
    }

    if added > 0 {
        let store = SqliteStore::open(&tate_dir.join("state").join("tate.db"))
            .map_err(|e| format!("hook: failed to open db: {e}"))?;
        let entries = deck.read().unwrap_or_default();
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        sync_deck(&store, &entries, &today).map_err(|e| format!("hook: failed to sync: {e}"))?;

        println!("tate: added {added} entries from tracked commit");
    }

    Ok(())
}

pub fn install_hook(repo_root: &Path) -> Result<(), String> {
    let hooks_dir = repo_root.join(".git").join("hooks");
    if !hooks_dir.exists() {
        return Err("not a git repository (no .git/hooks)".to_string());
    }

    let hook_path = hooks_dir.join("post-commit");
    let hook_line = "tate hook post-commit\n";

    if hook_path.exists() {
        let content = std::fs::read_to_string(&hook_path)
            .map_err(|e| format!("failed to read existing hook: {e}"))?;
        if content.contains("tate hook post-commit") {
            return Ok(());
        }
        let mut new_content = content;
        if !new_content.ends_with('\n') {
            new_content.push('\n');
        }
        new_content.push_str(hook_line);
        std::fs::write(&hook_path, new_content)
            .map_err(|e| format!("failed to append to hook: {e}"))?;
    } else {
        let content = format!("#!/bin/sh\n{hook_line}");
        std::fs::write(&hook_path, content).map_err(|e| format!("failed to create hook: {e}"))?;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o755);
        std::fs::set_permissions(&hook_path, perms)
            .map_err(|e| format!("failed to set hook permissions: {e}"))?;
    }

    Ok(())
}
