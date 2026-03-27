# Tate Specification

Version: 0.1.0

Spaced repetition CLI for code you don't own yet. This document is the canonical build spec. An AI agent should be able to implement Tate from this document alone.

## 1. Overview

Tate applies the SM-2 spaced repetition algorithm to code comprehension. It is source-agnostic: the code may be AI-generated, inherited from a teammate, vendored from a library, copied from a tutorial, or written by you six months ago. The mechanism is the same.

The core loop:

1. Code you don't fully understand gets added to your deck (manually, via git hook, or by an AI agent)
2. Questions are attached at add-time ("what should this handle that it might not?")
3. Developer reviews on schedule, grades understanding (Blank / Hard / Good / Easy)
4. SM-2 algorithm schedules the next review
5. When code changes, the card resets and the developer re-earns understanding

**v1 scope:** Single-user, local CLI. No GUI, no web UI, no team sync, no plugin system.

**Non-goals for v1:** Custom question templates, multiple decks, Anki export, IDE extensions.

## 2. Technical Stack

Language: Rust (2021 edition). Single binary, no runtime dependencies.

### Workspace layout (vertical slices)

```
tate/
  Cargo.toml              # workspace root
  crates/
    tate-core/             # pure domain types and algorithms, zero I/O
    tate-store/            # persistence: SQLite, deck file, config, sync
    tate-symbols/          # tree-sitter: symbol resolution, hashing, grammars
    tate-review/           # review session: TEA shell, terminal, change detection
    tate-hooks/            # git post-commit hook, auto-population
    tate-cli/              # thin binary: clap, subcommand dispatch, tracing subscriber
```

Crates are organized by feature, not by technical layer. Each feature slice owns its full stack (domain logic, persistence, I/O). `tate-core` is the shared pure kernel.

`tate-core` has no dependency on any I/O crate. If it compiles without `rusqlite`, `crossterm`, `tree-sitter`, or `thiserror`, the "pure core" claim is enforced by the build system, not discipline.

**Dependency graph:**

```
tate-cli -> tate-review -> tate-store -> tate-core
                        -> tate-symbols -> tate-core
         -> tate-hooks  -> tate-store
                        -> tate-symbols
         -> tate-store
```

### Dependencies by crate

**tate-core** (pure, no I/O):

| Purpose   | Crate   |
| --------- | ------- |
| Date/time | `chrono`|

**tate-store** (persistence):

| Purpose        | Crate                          |
| -------------- | ------------------------------ |
| SQLite         | `rusqlite` (bundled feature)   |
| TOML parsing   | `toml`                         |
| Hashing        | `blake3`                       |
| Diagnostics    | `tracing`                      |
| Error handling | `thiserror`                    |

**tate-symbols** (symbol resolution):

| Purpose        | Crate                                       |
| -------------- | ------------------------------------------- |
| Tree-sitter    | `tree-sitter` + per-language grammar crates |
| Hashing        | `blake3`                                    |
| Diagnostics    | `tracing`                                   |
| Error handling | `thiserror`                                 |

**tate-review** (review session):

| Purpose        | Crate                          |
| -------------- | ------------------------------ |
| Terminal input | `crossterm`                    |
| Diagnostics    | `tracing`                      |
| Error handling | `thiserror`                    |

**tate-hooks** (git integration):

| Purpose        | Crate                          |
| -------------- | ------------------------------ |
| Diagnostics    | `tracing`                      |
| Error handling | `thiserror`                    |

**tate-cli** (binary):

| Purpose        | Crate                          |
| -------------- | ------------------------------ |
| CLI framework  | `clap` (derive API)            |
| Diagnostics    | `tracing-subscriber`           |

Tree-sitter grammars are compiled in statically (not loaded at runtime). v1 ships a fixed set of languages in `tate-symbols`. Most development never triggers grammar recompilation.

### Type design

Make invalid states unrepresentable. The SQLite schema stores flat rows. The Rust layer enforces invariants through types. Validation happens at the boundary (hydration from DB, parsing from deck file). Once a value is in a typed form, invalid states are impossible.

**Entry** (parsed at deck file read and CLI input, never stored as raw string internally):

```rust
enum Entry {
    File(PathBuf),
    Symbol { path: PathBuf, name: String },
    Range { path: PathBuf, start: u32, end: u32 },
}
```

Three entry formats: `path/to/file` (whole file), `path/to/file::Symbol` (named symbol), `path/to/file:5-16` (line range). Construction fails if the path is empty, the symbol name is empty, the range is invalid (start > end, zero), or the format is unrecognized. All downstream code works with `Entry`, never with `String`.

**Grade** (parsed from keypress, never stored as raw integer internally):

```rust
enum Grade {
    Again,  // SM-2 quality 1
    Hard,   // SM-2 quality 3
    Good,   // SM-2 quality 4
    Easy,   // SM-2 quality 5
}
```

`Grade` implements `Into<u8>` for DB storage and `TryFrom<u8>` for DB reads. No other integer values are representable.

**Ease** (clamped at construction):

```rust
struct Ease(f64);  // invariant: self.0 >= 1.3
```

`Ease::new(v)` clamps to `max(v, 1.3)`. No method on `Ease` can produce a value below 1.3. The SM-2 update formula returns an `Ease`, not a raw `f64`.

**Card states** (typestate pattern, enforces valid transitions at compile time):

```rust
struct Card<S> {
    entry: Entry,
    ease: Ease,
    added: NaiveDate,
    body_hash: Option<String>,
    state: S,
}

struct New {
    due: NaiveDate,  // today for fresh and failed cards
    lapses: u32,     // 0 for truly new cards, > 0 for lapsed cards
}

struct Learning {
    reps: u32,       // > 0
    interval: u32,   // > 0, < 21
    due: NaiveDate,
    lapses: u32,
}

struct Mature {
    reps: u32,       // > 0
    interval: u32,   // >= 21
    due: NaiveDate,
    lapses: u32,
}

struct Retired;
```

**Transitions** (only valid state changes compile):

```
Card<New>      --[grade >= 2]--> Card<Learning>
Card<New>      --[grade == 1]--> Card<New>         (stays new, due today)
Card<Learning> --[grade >= 2]--> Card<Learning>    (or Card<Mature> if interval >= 21)
Card<Learning> --[grade == 1]--> Card<New>         (lapse)
Card<Mature>   --[grade >= 2]--> Card<Mature>
Card<Mature>   --[grade == 1]--> Card<New>          (lapse, reps resets to 0)
Any            --[tate own]---> Card<Retired>
Card<Retired>  --[re-added]---> Card<New>
```

A `Card<Retired>` has no `review()` method. The type system prevents scheduling a retired card. `Card<New>` carries `lapses` to distinguish truly new cards (lapses = 0) from lapsed cards that reset (lapses > 0), preserving failure history across the New -> Learning -> lapse -> New cycle.

**Hydration from SQLite:**

A row is read into a `CardRow` struct (flat, mirrors the schema). Then `CardRow::into_typed()` returns the appropriate `Card<S>` based on column values:

- `retired = 1` produces `Card<Retired>`
- `reps = 0 AND retired = 0` produces `Card<New>` (with `due` and `lapses` from the row)
- `reps > 0 AND interval < 21 AND retired = 0` produces `Card<Learning>`
- `reps > 0 AND interval >= 21 AND retired = 0` produces `Card<Mature>`

If columns are inconsistent (e.g., `reps = 5` but `interval = 0`), treat as corruption: log warning, reset to `Card<New>`.

### Architecture: Functional Core / Imperative Shell

The core contains all business logic. It is pure: no I/O, no side effects, no error types. Given inputs, it returns outputs deterministically. The SM-2 algorithm, card typestate transitions, deck validation, and entry parsing all live here.

The shell orchestrates I/O. It reads files, queries SQLite, renders to the terminal, writes to disk. All infrastructure errors live here. The shell calls the core, never the reverse.

The boundary follows The Elm Architecture (TEA):

```
Shell: execute(command) -> Result<Message, AdapterError>
Core:  update(state, message) -> (state, Vec<Command>)
Shell: execute next command...
```

`update` is pure and infallible. It takes the current state and a message (something that happened), returns the new state and a list of commands (side effects to perform). Commands are data describing what to do, not function calls.

**Hexagonal layers:**

```
Core (pure functions on data, no traits, no I/O):
  Functions: sm2_update, parse_entry, sync_deck, review_update
  Types: Card<S>, Entry, Grade, Ease, SkipReason, Message, Command, CardRow

Shell (orchestrates I/O, owns ports and adapters):
  Ports: CardStore, DeckFile, SymbolResolver, Terminal
  Adapters: SqliteStore, FsDeck, TreeSitterResolver, CrosstermTerminal
```

The core takes values and returns values. It never defines traits, never calls I/O, never sees `rusqlite::Error` or `std::io::Error`. The shell calls adapters, converts results to Messages, and feeds them to the core's `update` function.

**Domain outcomes are values, not errors:**

```rust
enum SkipReason {
    FileNotFound,
    SymbolNotFound { found: Vec<String> },
    ParseFailed,
}
```

A skipped card is a valid review outcome. The review session always succeeds (produces events). Individual cards may be skipped.

**Errors exist only in the shell, per adapter:**

```rust
#[derive(thiserror::Error, Debug)]
enum StorageError {
    #[error("database corrupted, recreating")]
    Corrupted,
    #[error("failed to write: {0}")]
    Write(#[source] rusqlite::Error),
    #[error("failed to read: {0}")]
    Read(#[source] rusqlite::Error),
}

#[derive(thiserror::Error, Debug)]
enum DeckFileError {
    #[error("deck file not found: {0}")]
    NotFound(PathBuf),
    #[error("failed to read deck: {0}")]
    Io(#[source] std::io::Error),
}

#[derive(thiserror::Error, Debug)]
enum ConfigError {
    #[error("invalid config: {0}")]
    Parse(#[source] toml::de::Error),
    #[error("invalid regex pattern: {pattern}")]
    InvalidPattern { pattern: String },
}
```

**Top-level CLI error (maps to exit codes):**

```rust
#[derive(thiserror::Error, Debug)]
enum CliError {
    #[error("not a tate project (run `tate init`)")]
    NotInitialized,
    #[error("not a git repository")]
    NotGitRepo,
    #[error("{0}")]
    Storage(#[from] StorageError),
    #[error("{0}")]
    DeckFile(#[from] DeckFileError),
    #[error("{0}")]
    Config(#[from] ConfigError),
}
```

**No `anyhow`, no `Box<dyn Error>`.** The core has zero error types. Adapters own infrastructure errors. `?` composes the shell.

## 3. Directory Structure

```
.tate/
  deck                 # plain text, personal (gitignored)
  config               # TOML, personal (gitignored)
  .gitignore           # contains: *
  state/
    tate.db            # SQLite database (gitignored)
```

### Deck file format

One entry per line. Blank lines and lines starting with `#` are ignored.

Three entry formats:

- File entry: `path/to/file.ext`
- Symbol entry: `path/to/file.ext::SymbolName`
- Range entry: `path/to/file.ext:5-16` (lines 5 through 16, 1-indexed, inclusive)

Paths are relative to repo root. No leading `./`. The deck file is the source of truth for what is tracked. Duplicate entries are deduplicated silently during sync.

### Config file

TOML format. Full schema in [Section 8](#8-configuration).

### State database

SQLite. Schema in [Section 4](#4-database-schema). Always gitignored. Disposable: can be rebuilt from the deck file (cards start as new, review history is lost).

## 4. Database Schema

```sql
CREATE TABLE meta (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
-- Seeded with: ('schema_version', '3')

CREATE TABLE cards (
    entry      TEXT PRIMARY KEY,
    ease       REAL NOT NULL DEFAULT 2.5,
    interval   INTEGER NOT NULL DEFAULT 0,
    due        TEXT NOT NULL,
    reps       INTEGER NOT NULL DEFAULT 0,
    lapses     INTEGER NOT NULL DEFAULT 0,
    added      TEXT NOT NULL,
    retired    INTEGER NOT NULL DEFAULT 0,
    body_hash  TEXT
);
-- All dates are UTC, ISO 8601 format (YYYY-MM-DD for dates, YYYY-MM-DDTHH:MM:SSZ for datetimes)
-- reviews.grade stores Tate grades (1-4), not SM-2 quality values
-- body_hash is BLAKE3 hex digest, NULL until first computed

CREATE TABLE reviews (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    entry       TEXT NOT NULL,
    reviewed_at TEXT NOT NULL,
    grade       INTEGER NOT NULL,
    FOREIGN KEY (entry) REFERENCES cards(entry)
);

CREATE TABLE questions (
    entry       TEXT PRIMARY KEY,
    body_hash   TEXT NOT NULL,
    question    TEXT NOT NULL,
    answer      TEXT,
    source_text TEXT,
    created_at  TEXT NOT NULL,
    FOREIGN KEY (entry) REFERENCES cards(entry)
);

CREATE INDEX idx_reviews_entry ON reviews(entry);
CREATE INDEX idx_cards_due ON cards(due) WHERE retired = 0;
```

### Schema migration

Check `meta.schema_version` on open. If it does not match the expected version, drop all tables and recreate. Log a warning.

### Deck sync algorithm

Runs at the start of every command that touches the database. Single transaction.

1. Read deck file lines into a set (skip blanks and comments)
2. For each line not in `cards`: INSERT with defaults, `due` = today
3. For each card where `entry` is not in deck and `retired = 0`: set `retired = 1`
4. For each card where `entry` IS in deck and `retired = 1`: set `retired = 0`, reset `ease = 2.5`, `interval = 0`, `reps = 0`, `lapses = 0`, `due = today`, `body_hash = NULL` (re-added entry starts fully fresh; question is kept if present)

## 5. SM-2 Algorithm

The SM-2 implementation lives in the functional core. It is pure and infallible: `(Card, Grade) -> Card`. The typestate pattern (Section 2) and the Ease newtype guarantee all inputs and outputs are valid. No error types, no Result returns.

### Grade mapping

Tate uses 4 grades. Mapped to SM-2 quality values:

| Tate Grade | Label | SM-2 quality (q) | Meaning                           |
| ---------- | ----- | ---------------- | --------------------------------- |
| 1          | Again | 1                | No recall. Complete failure.      |
| 2          | Hard  | 3                | Recalled with significant effort. |
| 3          | Good  | 4                | Recalled with some effort.        |
| 4          | Easy  | 5                | Instant recall.                   |

Grade 1 is a lapse (failure). Grades 2-4 are passing.

### After each review

Apply in order:

**1. Update ease factor:**

```
new_ease = ease + (0.1 - (5 - q) * (0.08 + (5 - q) * 0.02))
new_ease = max(new_ease, 1.3)
```

**2. Calculate next interval:**

If grade = 1 (lapse):

- `interval = 0`
- `reps = 0`
- `lapses += 1`
- `due = today` (card comes back in same session)

If grade >= 2 (pass):

- If `reps == 0`: `interval = 1`
- If `reps == 1`: `interval = 6`
- If `reps >= 2`: `interval = round(interval * new_ease)`
- `reps += 1`

**3. Clamp interval:**

```
interval = min(interval, max_interval)
```

`max_interval` is from config (default 365).

**4. Set due date:**

```
due = today + interval days
```

**5. Persist:** Update the `cards` row. Insert into `reviews`.

**6. State transition:** The SM-2 result determines the new card type (see type design in Section 2). A `Card<New>` with a passing grade becomes `Card<Learning>`. A `Card<Learning>` whose interval reaches 21 becomes `Card<Mature>`. A lapse on any card produces `Card<New>` (or `Card<Learning>` from `Card<Mature>`). These transitions are enforced by the type system, not runtime checks.

### On code change (body_hash mismatch)

When change detection (Section 10) finds a hash mismatch:

- Set `interval = 0`, `reps = 0`, `due = today`
- Do NOT reset ease (baseline familiarity with the area persists)
- Do NOT reset or increment lapses (content change is not a failure)
- Mark stored question as stale (body_hash will no longer match)

## 6. Tree-sitter Integration

### Purpose

1. Validate that a symbol entry in the deck exists in the source file
2. Compute a BLAKE3 hash of the symbol body for change detection

### Supported languages (v1)

26 languages with symbol-level tracking. Languages without a grammar fall back to whole-file or line-range tracking.

| Extension(s)           | Grammar crate              |
| ---------------------- | -------------------------- |
| `.rs`                  | `tree-sitter-rust`         |
| `.py`                  | `tree-sitter-python`       |
| `.ts`                  | `tree-sitter-typescript`   |
| `.tsx`                 | `tree-sitter-typescript`   |
| `.js`, `.jsx`          | `tree-sitter-javascript`   |
| `.go`                  | `tree-sitter-go`           |
| `.java`                | `tree-sitter-java`         |
| `.c`, `.h`             | `tree-sitter-c`            |
| `.cpp`, `.hpp`, `.cc`, `.cxx`, `.hh` | `tree-sitter-cpp` |
| `.cs`                  | `tree-sitter-c-sharp`      |
| `.rb`                  | `tree-sitter-ruby`         |
| `.odin`                | `tree-sitter-odin`         |
| `.dart`                | `tree-sitter-dart`         |
| `.ex`, `.exs`          | `tree-sitter-elixir`       |
| `.gleam`               | `tree-sitter-gleam`        |
| `.scala`, `.sc`        | `tree-sitter-scala`        |
| `.zig`                 | `tree-sitter-zig`          |
| `.ml`                  | `tree-sitter-ocaml`        |
| `.mli`                 | `tree-sitter-ocaml`        |
| `.swift`               | `tree-sitter-swift`        |
| `.hs`                  | `tree-sitter-haskell`      |
| `.lua`                 | `tree-sitter-lua`          |
| `.sh`, `.bash`         | `tree-sitter-bash`         |
| `.php`                 | `tree-sitter-php`          |
| `.r`, `.R`             | `tree-sitter-r`            |
| `.jl`                  | `tree-sitter-julia`        |
| `.clj`, `.cljs`, etc.  | Whole-file only            |
| `.sql`                 | Whole-file only            |

Unsupported extensions: whole-file or line-range tracking only.

### Symbol resolution algorithm

1. Determine language from file extension
2. Parse file with tree-sitter
3. Walk the syntax tree, collect nodes whose `kind()` matches the language's symbol node types
4. For each, extract the identifier child node's text as the symbol name
5. Match against the requested symbol name (exact match, first occurrence in file)
6. If found, extract the full byte range of the symbol node
7. Return the bytes for hashing or display

**v1 limitation:** Symbol matching uses the first match by name in the file. Ambiguous cases (overloaded Java methods, multiple Rust `impl` blocks for the same type, nested Python classes) may match the wrong symbol. Future versions may support qualified paths (e.g., `file.rs::MyStruct::impl::method`).

### File entry hashing

For whole-file entries, hash the entire file contents with BLAKE3. No tree-sitter needed.

### Symbol not found

- During `tate add`: error, refuse to add. Print symbols found in the file as suggestions.
- During review: produces `SkipReason::SymbolNotFound` or `SkipReason::ParseFailed`. No silent fallback to whole-file hashing. The card is skipped, not degraded. Card is not removed from deck automatically. Developer decides.
- During hook auto-add only: if tree-sitter parsing fails, add whole-file entry instead (because symbols are unknown). This is the only place whole-file fallback applies.

## 7. Questions

Questions and answers are provided externally, not generated by Tate. The AI agent that wrote the code provides the question and answer at add-time via the `-q` and `-a` flags. Tate has no LLM dependency.

### Adding questions

```
tate add src/auth/login.ts::authenticate -q "What happens when the token is expired?" -a "Checks expiry, attempts refresh, returns 401 if both fail."
```

If `-q` or `-a` is used on an entry that already exists in the deck, the question/answer is updated (entry is not re-added).

Questions and answers are stored in the `questions` table keyed by entry. The current `body_hash` is stored alongside the question. For range entries, the `source_text` column stores the full content of the range at add-time for content-anchored change detection.

### During review

1. Look up the question for this entry in the `questions` table
2. If found and `body_hash` matches the current hash, display the question
3. If found but `body_hash` does not match (code changed since question was written), display the question with a note: "(code has changed since this question was written)"
4. If no question exists, display: "Review this code. Can you explain the key decisions and potential edge cases?"

### Integration with tools

Anyone (or anything) that adds code can also add a question. AI coding agents (Claude Code, Cursor, Copilot) can call `tate add -q "..."` as part of their workflow. A team lead onboarding someone can add questions for key modules. A developer can add questions for their own code before they forget the reasoning.

Example Claude Code hook (post-tool):

```
tate add $FILE::$SYMBOL -q "$(generate_question)"
```

The git post-commit hook (Section 11) adds entries without questions. Questions can be added later by re-running `tate add -q` on existing entries.

## 8. Configuration

Default `.tate/config` generated by `tate init`:

```toml
[scheduling]
max_interval = 365              # max days between reviews
new_card_limit = 20             # max new cards introduced per day

[display]
show_code = true                # show source code during review
color = true                    # terminal colors

[hooks]
auto_add = true                 # enable post-commit auto-population
track_patterns = [              # commit message patterns that trigger auto-add
    "Co-authored-by:.*Claude",
    "Co-authored-by:.*Copilot",
    "Co-authored-by:.*Cursor",
    "Generated by",
    "🤖",
]
# Set to [".*"] to track all commits. Remove patterns to narrow scope.
```

All fields have defaults. A missing field uses its default. An empty config file is valid.

## 9. CLI Commands

Binary name: `tate`. All commands require `.tate/` to exist (except `init`). If `.tate/` is not found, print "Not a tate project. Run `tate init`." and exit 1.

**Global flag:** `--json` outputs structured JSON to stdout instead of human-readable text. Available on all commands.

### 9.1 `tate init`

**Usage:** `tate init`

**Behavior:**

1. Check if `.tate/` exists. If yes, print "Already initialized." and exit 0.
2. Create `.tate/` directory
3. Create empty `.tate/deck` file
4. Create `.tate/config` with defaults from Section 8
5. Create `.tate/state/` directory
6. Create `.tate/.gitignore` containing `state/`
7. Initialize SQLite database with schema from Section 4
8. Install git post-commit hook (see Section 11)
9. Detect languages by scanning file extensions in the repo
10. Print summary:
    ```
    Initialized tate.
    Detected languages: TypeScript, Python, Rust
    Post-commit hook installed.
    ```

**Exit codes:** 0 success, 1 if not in a git repo.

### 9.2 `tate add <entry>`

**Usage:** `tate add <path>`, `tate add <path>::<Symbol>`, or `tate add <path>:<start>-<end>`

**Flags:**
- `-q <question>` (optional) provides a decompression question (front of card)
- `-a <answer>` (optional) provides the answer (back of card)
- `--json` outputs result as JSON

**Behavior:**

1. Validate the file exists (relative to repo root)
2. If symbol entry, parse with tree-sitter, validate symbol exists
3. Compute body hash
4. If entry already in deck file:
   - If `-q` provided, update the question in the `questions` table (with current body_hash). Print: `Updated question: <entry>`
   - If no `-q`, print "Already tracked." and exit 0
5. If entry is new:
   - Append entry to `.tate/deck`
   - Run deck sync
   - Store body hash
   - If `-q` provided, insert into `questions` table
   - Print: `Added: <entry>`

**Errors:**

- File not found: exit 1
- Symbol not found: exit 1, print found symbols as suggestions
- Unsupported language for symbol entry: exit 1

### 9.3 `tate review`

**Usage:** `tate review`, `tate review --export`, `tate review --grade <entry> <1-4>`

**Non-interactive modes:**
- `--export`: dumps due cards as JSON array with source, question, answer. No grading, no TUI.
- `--grade <entry> <1-4>`: grades a single card non-interactively. Outputs result as JSON with `--json`.

**Interactive mode:**

The review session is structured as a TEA loop (see Section 2, Architecture). The shell handles I/O. The core decides what happens next. The core is pure and infallible.

**Messages** (things that happened):

```rust
enum Message {
    Next,                                              // advance to next card
    SourceResolved(Entry, Result<String, SkipReason>), // source read or skip reason
    QuestionLoaded(Entry, Option<String>, Option<String>), // question, answer
    Graded(Grade),                                     // user graded a card
    Quit,                                              // user pressed q
    Persisted(Entry),                                  // card saved to DB
}
```

**Commands** (side effects for the shell to execute):

```rust
enum Command {
    ResolveSource(Entry),                    // read and validate source from disk
    LoadQuestion(Entry),                     // read question from DB
    PresentCard { entry: Entry, source: String, question: Option<String> },
    RevealAnswer { answer: String },         // show answer, wait for space to flip
    PromptGrade,                             // wait for keypress
    PersistReview(Entry, CardRow),           // write flat DTO to DB
    ShowSkipped(Entry, SkipReason),          // display skip notice with reason
    ShowSummary { reviewed: u32, skipped: u32 },
}
```

**Shell behavior:**

Pre-session I/O (shell, can fail with CliError and abort before session starts):

1. Run deck sync
2. Run change detection for all due cards (Section 10)
3. Query cards: `due <= today AND retired = 0`, ordered by due ASC
4. Apply `new_card_limit`: of those, at most N cards with `reps = 0`
5. If none due, print "No cards due. Next review: <date>" and exit 0

TEA loop (per-card I/O failures become Messages, never errors):

6. Initialize review state with due cards
7. Loop:
   - Call `update(state, message)` (pure, infallible)
   - Execute each returned command via adapters (I/O)
   - Feed resulting messages back into `update`
   - Break when commands are empty

The `update` function handles all domain logic: which card to present next, whether to skip (source missing), how to apply SM-2, when the session is complete. The shell only does I/O.

**User interaction (ratatui TUI):**

Three-pane layout: code (scrollable, syntax-highlighted, line numbers), question/answer (separate bordered panes), grade bar. Nord-inspired color palette.

- **Before flip:** Code pane + question pane + `[space] flip` prompt. No grade bar.
- **After flip:** Answer pane appears. Grade bar appears with `[1] Again [2] Hard [3] Good [4] Easy [q] Quit`.
- **After grade:** Grade bar replaced with `Next review: <date>`.
- Grade input: single keypress (1-4 or space for Good), no enter required. `q` quits.
- Scrolling: arrow keys, j/k, Ctrl-d/Ctrl-u, g/G.
- Progress gauge at top: `3/8` with visual fill.

**Output after all cards:**

```
Session complete. Reviewed 5 cards. Skipped 1.
```

### 9.4 `tate status`

**Usage:** `tate status`

**Behavior:**

1. Run deck sync
2. Query and display:
   ```
   Deck:     42 entries
   Due:       5 today, 12 this week
   Streak:    7 days
   Progress:  8 new / 20 learning / 14 mature
   ```

Definitions:

- Streak: consecutive days with at least one review (from reviews table)
- New: `reps = 0`
- Learning: `reps > 0 AND interval < 21`
- Mature: `interval >= 21`

**Exit codes:** 0 always.

### 9.5 `tate list [filter]`

**Usage:** `tate list`, `tate list <prefix>`, `tate list --due`, `tate list --owned`

**Behavior:**

1. Run deck sync
2. Display entries as a table:
   ```
   ENTRY                                STATUS    INTERVAL  DUE
   src/auth/login.ts::authenticate      due       -         today
   src/auth/login.ts::LoginPayload      learning  6d        Mar 23
   src/db/migrations/001.sql            new       -         today
   src/api/routes.ts                    mature    45d       Apr 30
   ```

**Filters:**

- No argument: all non-retired entries
- Prefix: entries starting with that string
- `--due`: only cards due today
- `--owned`: only retired cards
- Filters can combine: `tate list src/auth/ --due`

**Exit codes:** 0 always.

### 9.6 `tate own <entry>`

**Usage:** `tate own <entry>`

**Behavior:**

1. Validate entry exists in deck file
2. If not found, print "Entry not in deck." and exit 1
3. Remove entry from deck file (write to temp file, atomic rename)
4. Set `retired = 1` in cards table
5. Print: `Owned: <entry>`

Deck file is written first. If it fails, DB is unchanged. If DB update fails after file rewrite, next sync reconciles. The card remains in SQLite for history.

## 10. Change Detection

Runs at the start of `tate review`, after deck sync.

For each card that is due and not retired:

1. Read the file at the entry path. If missing, set `body_hash = NULL`, flag for warning during review.
2. If symbol entry, resolve symbol via tree-sitter. If missing, set `body_hash = NULL`, flag for warning.
3. Compute BLAKE3 hash of the symbol body (or full file for file entries).
4. Compare with stored `body_hash`.
5. If different or stored hash is NULL: apply change reset (Section 5), update `body_hash`. Stored question is kept but marked stale by the hash mismatch.
6. If same: no action.

Tate never automatically removes entries from the deck. The developer always decides.

## 11. Auto-population

### Git post-commit hook

`tate init` installs a post-commit hook at `.git/hooks/post-commit` (or appends to an existing one).

The hook runs `tate hook post-commit` (a hidden subcommand).

**`tate hook post-commit` behavior:**

1. Check `hooks.auto_add` in config. If false, exit.
2. Get the commit message of HEAD.
3. Test against each regex pattern in `hooks.track_patterns`. If none match, exit.
4. Get the list of changed files: `git diff --name-only HEAD~1 HEAD`
5. For each changed file:
   a. If file extension is unsupported or file is in `.tate/`, skip
   b. Parse with tree-sitter
   c. Get all symbols in the file
   d. Add all symbols to the deck (duplicates are deduplicated by sync)
   e. If tree-sitter parsing fails, add the whole file entry instead (hook-only fallback)
6. Run deck sync
7. Print: `tate: added N entries from AI-assisted commit`

For the initial commit (no HEAD~1), diff against the empty tree.

### Existing hook handling

If `.git/hooks/post-commit` already exists:

- Append `tate hook post-commit` to the end of the file
- Do not overwrite existing content
- Print a note during `tate init` that the hook was appended

## 12. Diagnostics

Use `tracing` for all diagnostics. No `println!` for warnings, no `eprintln!` for errors. User-facing output (command results, review UI) goes to stdout. Everything else is a tracing event.

### Subscriber

Default subscriber: `tracing_subscriber::fmt` writing to stderr. Verbosity controlled by `TATE_LOG` env var (maps to `tracing` filter directives). Default level: `warn`.

```
TATE_LOG=debug tate review    # verbose output
TATE_LOG=info tate status     # moderate output
tate review                   # only warnings and errors (default)
```

### Spans

Each CLI command is a top-level span with structured fields:

```rust
#[tracing::instrument(fields(command = "review", deck_size, cards_due))]
fn cmd_review() { ... }
```

Key operations get sub-spans: `deck_sync`, `change_detection`, `tree_sitter_parse`, `sm2_update`. This makes it possible to trace exactly what happened during any command.

### Events by architectural layer

**Shell (CliError propagates, top-level):**

| Situation                     | Level | Behavior                          |
| ----------------------------- | ----- | --------------------------------- |
| Not initialized (no `.tate/`) | ERROR | Print message to stderr. Exit 1.  |
| Not a git repository          | ERROR | Print message to stderr. Exit 1.  |

**Adapters (internal recovery or propagation):**

| Situation                     | Level | Behavior                              |
| ----------------------------- | ----- | ------------------------------------- |
| SQLite corruption             | WARN  | Adapter recreates DB. Continues.      |
| Schema version mismatch       | WARN  | Adapter recreates tables. Continues.  |
| Config TOML parse error       | ERROR | Propagates as ConfigError.            |
| Config invalid regex          | ERROR | Propagates as ConfigError.            |
| Deck file I/O failure         | ERROR | Propagates as DeckFileError.          |
| Tree-sitter parse failure     | WARN  | During review: SkipReason. During hook: whole-file fallback.|
| Deck file has duplicates      | DEBUG | Adapter deduplicates during sync.     |
| Deck file has invalid line    | DEBUG | Adapter skips line during sync.       |
| Existing git hook file        | INFO  | Adapter appends, does not overwrite.  |

**Core (domain outcomes, not errors):**

| Situation                     | Level | Behavior                                |
| ----------------------------- | ----- | --------------------------------------- |
| File not found for entry      | INFO  | Core produces SkipReason::FileNotFound. |
| Symbol not found for entry    | INFO  | Core produces SkipReason::SymbolNotFound.|
| No question for entry         | DEBUG | Core uses generic fallback prompt.      |
| Stale question (hash mismatch)| DEBUG | Core displays question with note.       |

Principle: Tate never panics and never blocks a review session. The core is infallible. Adapters self-heal or propagate. The shell catches propagated errors and exits cleanly.

## 13. Build and Distribution

- Cargo workspace with six crates: `tate-core`, `tate-store`, `tate-symbols`, `tate-review`, `tate-hooks`, `tate-cli`
- `cargo build --release -p tate-cli` produces single binary named `tate`
- SQLite bundled via rusqlite in `tate-store` (no system dependency)
- Tree-sitter grammars compiled in via `tate-symbols` (no runtime grammar loading)
- Target platforms: Linux x86_64, macOS aarch64, macOS x86_64
- Distribution: GitHub releases with prebuilt binaries, `cargo install tate-cli`

## 14. Future Considerations

Out of scope for v1. Listed so the data model does not preclude them.

- `tate watch`: continuous file watcher for auto-adding
- FSRS algorithm as SM-2 alternative
- Whitespace-normalized hashing (reduce false resets from formatting changes)
- Content-anchored range drift detection (substring search when line numbers shift)
- Schema migration without data loss (ALTER TABLE instead of drop/recreate)
- Custom question templates
- Multiple decks
- Anki export
- VS Code / IDE extension
- Per-card tags and filtering
- Built-in LLM question generation (provider-configurable, for users without AI agent integration)
