use std::process::Command;

fn tate_bin() -> String {
    let mut path = std::env::current_exe().unwrap();
    path.pop();
    path.pop();
    path.push("tate");
    path.to_string_lossy().to_string()
}

fn setup_repo() -> tempfile::TempDir {
    let dir = tempfile::TempDir::new().unwrap();
    Command::new("git")
        .args(["init"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    Command::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    Command::new("git")
        .args(["config", "user.name", "Test"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    dir
}

fn run_tate(dir: &std::path::Path, args: &[&str]) -> std::process::Output {
    Command::new(tate_bin())
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap()
}

fn stdout(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stdout).to_string()
}

fn read_deck(dir: &std::path::Path) -> String {
    std::fs::read_to_string(dir.join(".tate/deck")).unwrap_or_default()
}
#[test]
fn init_creates_structure() {
    let dir = setup_repo();
    let out = run_tate(dir.path(), &["init"]);
    assert!(out.status.success());
    assert!(stdout(&out).contains("Initialized tate"));
    assert!(dir.path().join(".tate/deck").exists());
    assert!(dir.path().join(".tate/config").exists());
    assert!(dir.path().join(".tate/state/tate.db").exists());
    assert!(dir.path().join(".tate/.gitignore").exists());
}

#[test]
fn init_already_initialized() {
    let dir = setup_repo();
    run_tate(dir.path(), &["init"]);
    let out = run_tate(dir.path(), &["init"]);
    assert!(out.status.success());
    assert!(stdout(&out).contains("Already initialized"));
}

#[test]
fn add_file_entry() {
    let dir = setup_repo();
    std::fs::write(dir.path().join("main.rs"), "fn main() {}\n").unwrap();
    run_tate(dir.path(), &["init"]);

    let out = run_tate(dir.path(), &["add", "main.rs"]);
    assert!(out.status.success());
    assert!(stdout(&out).contains("Added: main.rs"));
    assert!(read_deck(dir.path()).contains("main.rs"));
}

#[test]
fn add_symbol_entry_with_question() {
    let dir = setup_repo();
    std::fs::write(dir.path().join("lib.rs"), "fn hello() {}\nfn world() {}\n").unwrap();
    run_tate(dir.path(), &["init"]);

    let out = run_tate(
        dir.path(),
        &[
            "add",
            "lib.rs::hello",
            "-q",
            "What does this do?",
            "-a",
            "It prints hello.",
        ],
    );
    assert!(out.status.success());
    assert!(stdout(&out).contains("Added: lib.rs::hello"));
    assert!(read_deck(dir.path()).contains("lib.rs::hello"));
}

#[test]
fn add_duplicate_is_idempotent() {
    let dir = setup_repo();
    std::fs::write(dir.path().join("main.rs"), "fn main() {}\n").unwrap();
    run_tate(dir.path(), &["init"]);

    run_tate(dir.path(), &["add", "main.rs"]);
    let out = run_tate(dir.path(), &["add", "main.rs"]);
    assert!(out.status.success());
    assert!(stdout(&out).contains("Already tracked"));

    let deck = read_deck(dir.path());
    assert_eq!(deck.matches("main.rs").count(), 1);
}

#[test]
fn add_nonexistent_file_fails() {
    let dir = setup_repo();
    run_tate(dir.path(), &["init"]);

    let out = run_tate(dir.path(), &["add", "nonexistent.rs"]);
    assert!(!out.status.success());
}

#[test]
fn add_nonexistent_symbol_fails_with_suggestions() {
    let dir = setup_repo();
    std::fs::write(dir.path().join("lib.rs"), "fn existing() {}\n").unwrap();
    run_tate(dir.path(), &["init"]);

    let out = run_tate(dir.path(), &["add", "lib.rs::nonexistent"]);
    assert!(!out.status.success());
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("existing"));
}

#[test]
fn status_shows_counts() {
    let dir = setup_repo();
    std::fs::write(dir.path().join("a.rs"), "fn a() {}\n").unwrap();
    std::fs::write(dir.path().join("b.rs"), "fn b() {}\n").unwrap();
    run_tate(dir.path(), &["init"]);
    run_tate(dir.path(), &["add", "a.rs"]);
    run_tate(dir.path(), &["add", "b.rs"]);

    let out = run_tate(dir.path(), &["status"]);
    assert!(out.status.success());
    let s = stdout(&out);
    assert!(s.contains("2 entries"));
    assert!(s.contains("2 new"));
}

#[test]
fn list_shows_entries() {
    let dir = setup_repo();
    std::fs::write(dir.path().join("main.rs"), "fn main() {}\n").unwrap();
    run_tate(dir.path(), &["init"]);
    run_tate(dir.path(), &["add", "main.rs"]);

    let out = run_tate(dir.path(), &["list"]);
    assert!(out.status.success());
    assert!(stdout(&out).contains("main.rs"));
}

#[test]
fn own_removes_from_deck() {
    let dir = setup_repo();
    std::fs::write(dir.path().join("main.rs"), "fn main() {}\n").unwrap();
    run_tate(dir.path(), &["init"]);
    run_tate(dir.path(), &["add", "main.rs"]);

    let out = run_tate(dir.path(), &["own", "main.rs"]);
    assert!(out.status.success());
    assert!(stdout(&out).contains("Owned: main.rs"));
    assert!(!read_deck(dir.path()).contains("main.rs"));
}

#[test]
fn own_nonexistent_entry_fails() {
    let dir = setup_repo();
    run_tate(dir.path(), &["init"]);

    let out = run_tate(dir.path(), &["own", "nonexistent.rs"]);
    assert!(!out.status.success());
}

#[test]
fn list_owned_filter() {
    let dir = setup_repo();
    std::fs::write(dir.path().join("main.rs"), "fn main() {}\n").unwrap();
    run_tate(dir.path(), &["init"]);
    run_tate(dir.path(), &["add", "main.rs"]);
    run_tate(dir.path(), &["own", "main.rs"]);

    let out = run_tate(dir.path(), &["list", "--owned"]);
    assert!(out.status.success());
    assert!(stdout(&out).contains("main.rs"));
    assert!(stdout(&out).contains("owned"));
}

#[test]
fn add_range_entry() {
    let dir = setup_repo();
    std::fs::write(
        dir.path().join("styles.css"),
        "/* reset */\nbody {\n  color: red;\n  margin: 0;\n}\n",
    )
    .unwrap();
    run_tate(dir.path(), &["init"]);

    let out = run_tate(dir.path(), &["add", "styles.css:2-5"]);
    assert!(out.status.success());
    assert!(stdout(&out).contains("Added: styles.css:2-5"));
    assert!(read_deck(dir.path()).contains("styles.css:2-5"));
}

#[test]
fn add_range_out_of_bounds_fails() {
    let dir = setup_repo();
    std::fs::write(dir.path().join("small.css"), "a {}\nb {}\n").unwrap();
    run_tate(dir.path(), &["init"]);

    let out = run_tate(dir.path(), &["add", "small.css:1-100"]);
    assert!(!out.status.success());
}

#[test]
fn not_initialized_error() {
    let dir = setup_repo();
    let out = run_tate(dir.path(), &["status"]);
    assert!(!out.status.success());
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("Not a tate project"));
}

#[test]
fn status_json_output() {
    let dir = setup_repo();
    std::fs::write(dir.path().join("main.rs"), "fn main() {}\n").unwrap();
    run_tate(dir.path(), &["init"]);
    run_tate(dir.path(), &["add", "main.rs"]);

    let out = run_tate(dir.path(), &["status", "--json"]);
    assert!(out.status.success());
    let json: serde_json::Value = serde_json::from_str(&stdout(&out)).expect("valid JSON");
    assert_eq!(json["deck"]["total"], 1);
    assert!(json["streak"].is_number());
}

#[test]
fn list_json_output() {
    let dir = setup_repo();
    std::fs::write(dir.path().join("main.rs"), "fn main() {}\n").unwrap();
    run_tate(dir.path(), &["init"]);
    run_tate(dir.path(), &["add", "main.rs"]);

    let out = run_tate(dir.path(), &["list", "--json"]);
    assert!(out.status.success());
    let json: serde_json::Value = serde_json::from_str(&stdout(&out)).expect("valid JSON");
    assert!(json.is_array());
    assert_eq!(json[0]["entry"], "main.rs");
    assert!(json[0]["ease"].is_f64());
}

#[test]
fn add_json_output() {
    let dir = setup_repo();
    std::fs::write(dir.path().join("main.rs"), "fn main() {}\n").unwrap();
    run_tate(dir.path(), &["init"]);

    let out = run_tate(dir.path(), &["add", "main.rs", "--json"]);
    assert!(out.status.success());
    let json: serde_json::Value = serde_json::from_str(&stdout(&out)).expect("valid JSON");
    assert_eq!(json["action"], "added");
    assert_eq!(json["entry"], "main.rs");
}

#[test]
fn review_export_json() {
    let dir = setup_repo();
    std::fs::write(dir.path().join("main.rs"), "fn main() {}\n").unwrap();
    run_tate(dir.path(), &["init"]);
    run_tate(dir.path(), &["add", "main.rs"]);

    let out = run_tate(dir.path(), &["review", "--export"]);
    assert!(out.status.success());
    let json: serde_json::Value = serde_json::from_str(&stdout(&out)).expect("valid JSON");
    assert!(json.is_array());
    assert_eq!(json[0]["entry"], "main.rs");
    assert!(json[0]["source"].is_string());
}

#[test]
fn review_grade_json() {
    let dir = setup_repo();
    std::fs::write(dir.path().join("main.rs"), "fn main() {}\n").unwrap();
    run_tate(dir.path(), &["init"]);
    run_tate(dir.path(), &["add", "main.rs"]);

    let out = run_tate(dir.path(), &["review", "--grade", "main.rs", "3", "--json"]);
    assert!(out.status.success());
    let json: serde_json::Value = serde_json::from_str(&stdout(&out)).expect("valid JSON");
    assert_eq!(json["entry"], "main.rs");
    assert_eq!(json["grade"], 3);
    assert!(json["next_due"].is_string());
    assert!(json["interval"].is_number());
}

#[test]
fn own_json_output() {
    let dir = setup_repo();
    std::fs::write(dir.path().join("main.rs"), "fn main() {}\n").unwrap();
    run_tate(dir.path(), &["init"]);
    run_tate(dir.path(), &["add", "main.rs"]);

    let out = run_tate(dir.path(), &["own", "main.rs", "--json"]);
    assert!(out.status.success());
    let json: serde_json::Value = serde_json::from_str(&stdout(&out)).expect("valid JSON");
    assert_eq!(json["action"], "retired");
    assert_eq!(json["entry"], "main.rs");
}

#[test]
fn config_list_all() {
    let dir = setup_repo();
    run_tate(dir.path(), &["init"]);

    let out = run_tate(dir.path(), &["config"]);
    assert!(out.status.success());
    let s = stdout(&out);
    assert!(s.contains("scheduling.max_interval"));
    assert!(s.contains("display.theme"));
}

#[test]
fn config_get_and_set() {
    let dir = setup_repo();
    run_tate(dir.path(), &["init"]);

    let out = run_tate(dir.path(), &["config", "display.theme"]);
    assert!(out.status.success());
    assert!(stdout(&out).contains("base16-eighties.dark"));

    let out = run_tate(dir.path(), &["config", "display.theme", "Nord"]);
    assert!(out.status.success());

    let out = run_tate(dir.path(), &["config", "display.theme"]);
    assert_eq!(stdout(&out).trim(), "Nord");
}

#[test]
fn config_json() {
    let dir = setup_repo();
    run_tate(dir.path(), &["init"]);

    let out = run_tate(dir.path(), &["config", "--json"]);
    assert!(out.status.success());
    let json: serde_json::Value = serde_json::from_str(&stdout(&out)).expect("valid JSON");
    assert!(json["display.theme"].is_string());
}

#[test]
fn themes_list() {
    let dir = setup_repo();
    run_tate(dir.path(), &["init"]);

    let out = run_tate(dir.path(), &["themes"]);
    assert!(out.status.success());
    let s = stdout(&out);
    assert!(s.contains("Nord"));
    assert!(s.contains("Dracula"));
    assert!(s.contains("base16-eighties.dark"));
}
