use std::path::{Path, PathBuf};

use tate_store::config::Config;
use tate_store::db::SqliteStore;
use tate_store::deck::DeckFile;

pub fn ensure_initialized(cwd: &Path) -> Result<PathBuf, String> {
    let tate_dir = cwd.join(".tate");
    if !tate_dir.exists() {
        return Err("Not a tate project. Run `tate init`.".to_string());
    }
    Ok(tate_dir)
}

pub fn open_store(tate_dir: &Path) -> Result<SqliteStore, String> {
    let db_path = tate_dir.join("state").join("tate.db");
    SqliteStore::open(&db_path).map_err(|e| format!("failed to open database: {e}"))
}

pub fn open_deck(tate_dir: &Path) -> DeckFile {
    DeckFile::new(tate_dir.join("deck"))
}

pub fn load_config(tate_dir: &Path) -> Result<Config, String> {
    Config::load(&tate_dir.join("config")).map_err(|e| format!("failed to load config: {e}"))
}

pub fn today_str() -> String {
    chrono::Utc::now().format("%Y-%m-%d").to_string()
}
