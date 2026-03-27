mod add;
mod common;
mod config_cmd;
mod init;
mod list;
mod own;
mod review_cmd;
mod status;
mod themes;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "tate", about = "Spaced repetition for code you don't own yet")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    #[arg(long, global = true, help = "Output as JSON")]
    json: bool,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Initialize tate in the current repository")]
    Init,
    #[command(about = "Add a file, symbol, or line range to the deck")]
    Add {
        #[arg(help = "Entry: path, path::symbol, or path:start-end")]
        entry: String,
        #[arg(short, help = "Question (front of card)")]
        q: Option<String>,
        #[arg(short, help = "Answer (back of card)")]
        a: Option<String>,
    },
    #[command(about = "Start a review session")]
    Review {
        #[arg(long, help = "Export due cards as JSON (no grading)")]
        export: bool,
        #[arg(long, num_args = 2, value_names = &["ENTRY", "GRADE"], help = "Grade a card non-interactively (1-4)")]
        grade: Option<Vec<String>>,
    },
    #[command(about = "Show deck size, due cards, and streak")]
    Status,
    #[command(about = "List deck entries")]
    List {
        #[arg(help = "Filter entries by prefix")]
        prefix: Option<String>,
        #[arg(long, help = "Show only cards due today")]
        due: bool,
        #[arg(long, help = "Show only owned (retired) cards")]
        owned: bool,
    },
    #[command(about = "Mark an entry as owned (remove from rotation)")]
    Own {
        #[arg(help = "Entry to mark as owned")]
        entry: String,
    },
    #[command(about = "Get or set config values")]
    Config {
        #[arg(help = "Config key (e.g., display.theme)")]
        key: Option<String>,
        #[arg(help = "Value to set")]
        value: Option<String>,
    },
    #[command(about = "List available syntax highlighting themes")]
    Themes,
    #[command(hide = true)]
    Hook {
        #[command(subcommand)]
        action: HookAction,
    },
}

#[derive(Subcommand)]
enum HookAction {
    PostCommit,
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_env("TATE_LOG")
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .with_writer(std::io::stderr)
        .init();

    let cli = Cli::parse();
    let cwd = std::env::current_dir().expect("failed to get current directory");
    let json = cli.json;

    let result = match cli.command {
        Commands::Init => init::run(&cwd),
        Commands::Add { entry, q, a } => add::run(&cwd, &entry, q.as_deref(), a.as_deref(), json),
        Commands::Review { export, grade } => review_cmd::run(&cwd, json, export, grade),
        Commands::Status => status::run(&cwd, json),
        Commands::List { prefix, due, owned } => {
            list::run(&cwd, prefix.as_deref(), due, owned, json)
        }
        Commands::Own { entry } => own::run(&cwd, &entry, json),
        Commands::Config { key, value } => {
            config_cmd::run(&cwd, key.as_deref(), value.as_deref(), json)
        }
        Commands::Themes => themes::run(&cwd, json),
        Commands::Hook { action } => match action {
            HookAction::PostCommit => tate_hooks::post_commit::run(&cwd),
        },
    };

    if let Err(e) = result {
        if json {
            println!("{}", serde_json::json!({"error": e.to_string()}));
        } else {
            eprintln!("error: {e}");
        }
        std::process::exit(1);
    }
}
