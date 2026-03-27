use std::path::Path;

use crate::common;

pub fn run(
    repo_root: &Path,
    key: Option<&str>,
    value: Option<&str>,
    json: bool,
) -> Result<(), String> {
    let tate_dir = common::ensure_initialized(repo_root)?;
    let mut config = common::load_config(&tate_dir)?;

    match (key, value) {
        (None, None) => {
            if json {
                let pairs: std::collections::HashMap<&str, String> =
                    config.all_keys().into_iter().collect();
                println!(
                    "{}",
                    serde_json::to_string_pretty(&pairs).unwrap_or_default()
                );
            } else {
                for (k, v) in config.all_keys() {
                    println!("{k} = {v}");
                }
            }
        }
        (Some(k), None) => {
            let v = config
                .get_value(k)
                .ok_or_else(|| format!("unknown key: {k}"))?;
            println!("{v}");
        }
        (Some(k), Some(v)) => {
            config.set_value(k, v)?;
            let config_path = tate_dir.join("config");
            std::fs::write(&config_path, config.to_toml_string())
                .map_err(|e| format!("failed to write config: {e}"))?;
            if json {
                println!("{}", serde_json::json!({"key": k, "value": v}));
            } else {
                println!("{k} = {v}");
            }
        }
        (None, Some(_)) => return Err("provide a key to set".to_string()),
    }

    Ok(())
}
