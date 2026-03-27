+++
title = "Usage"
weight = 2
description = "Review sessions, adding entries, listing cards, and CLI commands."

[extra]
group = "start"
+++

## Review session

Start your daily review with:

```
tate review
```

Tate presents each due card with the source code (syntax-highlighted, scrollable) and a decompression question. You grade your understanding:

- **[1] Again** - no recall, complete failure
- **[2] Hard** - recalled with significant effort
- **[3] Good** - recalled with some effort
- **[4] Easy** - instant recall

Grade 1 is a lapse. The card resets and comes back in the same session. Grades 2-4 are passing, and SM-2 calculates the next review date.

```
$ tate review

  src/auth/login.ts::authenticate
  Review #3 · Last reviewed 8 days ago

  What happens when the token is expired but the
  refresh token is still valid?

  [1] Again  [2] Hard  [3] Good  [4] Easy

> 3

  Next review: April 5
  0 cards remaining today.
```

### Keyboard controls

| Key | Action |
|-----|--------|
| `space` | Flip card (show answer), or grade Good |
| `1-4` | Grade the card |
| `j/k` | Scroll code down/up |
| `Ctrl-d/Ctrl-u` | Half-page scroll |
| `g/G` | Jump to top/bottom |
| `q` | Quit the session |

### Non-interactive mode

Export due cards as JSON for use in scripts or external tools:

```
tate review --export
```

Grade a single card without the TUI:

```
tate review --grade src/auth/login.ts::authenticate 3
```

## Status

Check your deck at a glance:

```
$ tate status

Deck:     42 entries
Due:       5 today, 12 this week
Streak:    7 days
Progress:  8 new / 20 learning / 14 mature
```

- **New**: never reviewed (`reps = 0`)
- **Learning**: reviewed but interval < 21 days
- **Mature**: interval >= 21 days (you really know this)
- **Streak**: consecutive days with at least one review

## Adding entries

Add files, symbols, or line ranges to your deck:

```
tate add src/api/routes.ts                   # whole file
tate add src/auth/login.ts::authenticate     # named symbol
tate add src/styles/reset.css:1-12           # line range
```

### With questions

Attach a decompression question (and optional answer) at add-time:

```
tate add src/auth/login.ts::authenticate \
  -q "What happens when the token is expired but the refresh token is still valid?" \
  -a "Checks expiry, attempts refresh, returns 401 if both fail."
```

Questions can be added or updated on existing entries by re-running `tate add -q` on the same entry.

## Listing entries

```
tate list                        # all non-retired entries
tate list src/auth/              # filter by path prefix
tate list --due                  # only cards due today
tate list --owned                # only retired cards
```

Output:

```
ENTRY                                STATUS    INTERVAL  DUE
src/auth/login.ts::authenticate      due       -         today
src/auth/login.ts::LoginPayload      learning  6d        Mar 23
src/db/migrations/001.sql            new       -         today
src/api/routes.ts                    mature    45d       Apr 30
```

## Marking as owned

When you fully understand a piece of code, retire it from your deck:

```
tate own src/auth/login.ts::LoginPayload
```

The card leaves active rotation but stays in the database for history. If the entry is re-added to the deck later, it starts fresh as a new card.

## JSON output

All commands support `--json` for structured output:

```
tate status --json
tate list --json
tate review --grade src/auth/login.ts::authenticate 3 --json
```
