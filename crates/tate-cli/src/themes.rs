use std::path::Path;

use crate::common;

pub fn run(repo_root: &Path, json: bool) -> Result<(), String> {
    let tate_dir = common::ensure_initialized(repo_root)?;
    let config = common::load_config(&tate_dir)?;
    let theme_set = tate_review::highlight::load_theme_set();
    let mut names: Vec<&str> = theme_set.themes.keys().map(|s| s.as_str()).collect();
    names.sort();

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&names).unwrap_or_default()
        );
    } else {
        for name in &names {
            if *name == config.display.theme {
                println!("* {name}");
            } else {
                println!("  {name}");
            }
        }
    }

    Ok(())
}
