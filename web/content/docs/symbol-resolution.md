+++
title = "Symbol Resolution"
weight = 5
description = "Tree-sitter integration, 26 supported languages, and BLAKE3 hashing."

[extra]
group = "reference"
+++

Tate uses [tree-sitter](https://tree-sitter.github.io/) to parse source files and extract named symbols (functions, classes, types, structs). This powers two features:

1. **Validation** - confirming a symbol entry in the deck actually exists in the source file
2. **Change detection** - hashing just the symbol body with BLAKE3, so file-level changes that don't affect your symbol don't reset the card

## Supported languages

26 languages with symbol-level tracking:

| Extension(s) | Language |
|---|---|
| `.rs` | Rust |
| `.py` | Python |
| `.ts` | TypeScript |
| `.tsx` | TypeScript (JSX) |
| `.js`, `.jsx` | JavaScript |
| `.go` | Go |
| `.java` | Java |
| `.c`, `.h` | C |
| `.cpp`, `.hpp`, `.cc`, `.cxx`, `.hh` | C++ |
| `.cs` | C# |
| `.rb` | Ruby |
| `.odin` | Odin |
| `.dart` | Dart |
| `.ex`, `.exs` | Elixir |
| `.gleam` | Gleam |
| `.scala`, `.sc` | Scala |
| `.zig` | Zig |
| `.ml`, `.mli` | OCaml |
| `.swift` | Swift |
| `.hs` | Haskell |
| `.lua` | Lua |
| `.sh`, `.bash` | Bash |
| `.php` | PHP |
| `.r`, `.R` | R |
| `.jl` | Julia |

Languages without a grammar (SQL, Clojure, YAML, etc.) fall back to whole-file or line-range tracking.

## How resolution works

1. Determine language from file extension
2. Parse file with tree-sitter
3. Walk the syntax tree, collecting nodes whose `kind()` matches the language's symbol node types (function declarations, class definitions, struct declarations, etc.)
4. Extract the identifier child node's text as the symbol name
5. Match against the requested symbol name (exact match, first occurrence)
6. If found, extract the full byte range of the symbol node
7. Return the bytes for BLAKE3 hashing or display

## Change detection

BLAKE3 hashes are stored per-card in the database. At the start of each review session:

- For **file entries**: hash the entire file contents
- For **symbol entries**: resolve the symbol, hash just the symbol body
- For **range entries**: hash the specified line range

If the hash differs from the stored value, the card resets: interval goes to 0, reps go to 0, due date becomes today. Ease is preserved (your baseline familiarity with the area persists). Lapses are not incremented (a code change is not a failure).

## Symbol not found

Different commands handle missing symbols differently:

- **`tate add`**: error, refuses to add. Prints found symbols as suggestions.
- **`tate review`**: skips the card with a notice. The card stays in the deck for the developer to investigate.
- **Git hook auto-add**: falls back to whole-file entry (because symbols are unknown at commit time). This is the only place whole-file fallback applies.

## Limitations

Symbol matching uses the first match by name in the file. Ambiguous cases (overloaded Java methods, multiple Rust `impl` blocks for the same type, nested Python classes) may match the wrong symbol.

Tree-sitter grammars are compiled in statically. No runtime grammar loading.
