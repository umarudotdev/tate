+++
title = "Deck Format"
weight = 4
description = "File, symbol, and range entry formats in .tate/deck."

[extra]
group = "reference"
+++

Tate tracks entries in `.tate/deck`, a plain text file. One entry per line. Blank lines and lines starting with `#` are ignored.

## Entry formats

Three levels of granularity:

### File entries

Track an entire file:

```
src/db/migrations/001.sql
src/api/routes.ts
```

The whole file content is hashed for change detection.

### Symbol entries

Track a specific function, class, type, or other named symbol:

```
src/auth/login.ts::authenticate
src/auth/login.ts::LoginPayload
src/core/sm2.rs::sm2_update
```

Symbols are resolved via tree-sitter. The symbol body is extracted and hashed independently. If the file changes but your symbol doesn't, the card stays on schedule.

### Range entries

Track specific line ranges (1-indexed, inclusive):

```
src/styles/reset.css:1-12
src/config/nginx.conf:45-67
```

Useful for code without named symbols: CSS rules, YAML blocks, SQL migrations, config sections, anonymous callbacks.

## Example deck file

```
# Authentication module
src/auth/login.ts::authenticate
src/auth/login.ts::LoginPayload
src/auth/login.ts::refresh_token

# Database
src/db/migrations/001.sql

# API layer
src/api/routes.ts::handleRequest

# Styles
src/styles/reset.css:1-12
```

## Sync algorithm

Tate syncs the deck file with the SQLite database at the start of every command:

1. Read deck file lines into a set (skip blanks and comments)
2. New entries in deck but not in DB: create as new cards, due today
3. Entries in DB but removed from deck: mark as retired
4. Entries in DB that are retired but re-appear in deck: fully reset (new card, due today, ease reset)

The deck file is the source of truth. The database is derived state. You can safely edit the deck file by hand, and the next command will reconcile.

## Paths

All paths are relative to the repository root. No leading `./`.

```
# correct
src/auth/login.ts::authenticate

# incorrect
./src/auth/login.ts::authenticate
/home/user/project/src/auth/login.ts::authenticate
```

Duplicate entries are deduplicated silently during sync.
