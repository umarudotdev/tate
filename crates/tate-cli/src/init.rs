use std::collections::HashSet;
use std::fs;
use std::path::Path;

use tate_store::config::Config;
use tate_store::db::SqliteStore;

pub fn run(repo_root: &Path) -> Result<(), String> {
    let tate_dir = repo_root.join(".tate");

    if tate_dir.exists() {
        println!("Already initialized.");
        return Ok(());
    }

    fs::create_dir_all(tate_dir.join("state"))
        .map_err(|e| format!("failed to create .tate/state: {e}"))?;

    fs::write(tate_dir.join("deck"), "").map_err(|e| format!("failed to create deck file: {e}"))?;

    let config = Config::default();
    fs::write(tate_dir.join("config"), config.to_toml_string())
        .map_err(|e| format!("failed to create config: {e}"))?;

    fs::write(tate_dir.join(".gitignore"), "*\n")
        .map_err(|e| format!("failed to create .gitignore: {e}"))?;

    let db_path = tate_dir.join("state").join("tate.db");
    SqliteStore::open(&db_path).map_err(|e| format!("failed to initialize database: {e}"))?;

    match tate_hooks::post_commit::install_hook(repo_root) {
        Ok(()) => println!("Post-commit hook installed."),
        Err(e) => eprintln!("warning: could not install hook: {e}"),
    }

    let languages = detect_languages(repo_root);

    println!("Initialized tate.");
    if !languages.is_empty() {
        let langs: Vec<&str> = languages.iter().map(|s| s.as_str()).collect();
        println!("Detected languages: {}", langs.join(", "));
    }

    Ok(())
}

fn detect_languages(root: &Path) -> Vec<String> {
    let mut extensions = HashSet::new();
    walk_extensions(root, &mut extensions, 0);

    let mut languages: Vec<String> = extensions
        .iter()
        .filter_map(|ext| ext_to_language(ext))
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    languages.sort();
    languages
}

fn walk_extensions(dir: &Path, extensions: &mut HashSet<String>, depth: u32) {
    if depth > 10 {
        return;
    }

    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        if name_str.starts_with('.')
            || name_str == "target"
            || name_str == "node_modules"
            || name_str == "vendor"
        {
            continue;
        }

        if path.is_dir() {
            walk_extensions(&path, extensions, depth + 1);
        } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            extensions.insert(ext.to_string());
        }
    }
}

fn ext_to_language(ext: &str) -> Option<String> {
    match ext {
        "rs" => Some("Rust".to_string()),
        "py" => Some("Python".to_string()),
        "js" | "jsx" => Some("JavaScript".to_string()),
        "ts" | "tsx" => Some("TypeScript".to_string()),
        "go" => Some("Go".to_string()),
        "java" => Some("Java".to_string()),
        "c" | "h" => Some("C".to_string()),
        "cpp" | "hpp" | "cc" | "cxx" => Some("C++".to_string()),
        "rb" => Some("Ruby".to_string()),
        "scala" | "sc" => Some("Scala".to_string()),
        "sql" => Some("SQL".to_string()),
        _ => None,
    }
}
