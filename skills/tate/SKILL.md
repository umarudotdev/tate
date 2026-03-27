---
name: tate
description: "Generate spaced repetition cards for code you just wrote. Use this skill after completing any coding task (feature, bug fix, refactoring) in a project with a .tate/ directory. Also trigger when the user says 'add to tate', 'create cards', or 'track this code'. The skill creates review cards with decompression questions so the developer can verify they understand what was shipped."
---

# Tate: Generate review cards for code you wrote

After finishing a coding task, create spaced repetition cards for the code you wrote or significantly modified. Each card has a question (front) and answer (back) that test whether the developer can decompress the code back into the reasoning behind it.

## Prerequisites

Check that `tate` is in PATH. If not, skip silently. If `.tate/` doesn't exist, run `tate init` to set it up, then continue with card generation.

## What to card

You know what you wrote because you just wrote it. Don't diff or scan. Card the symbols where the reasoning matters:

- Functions with non-obvious error handling or edge cases
- Types where the field choices encode domain constraints
- Algorithms where the approach was chosen over alternatives
- Integration points where assumptions about external behavior are baked in

Skip trivial getters, simple constructors, one-line wrappers, and boilerplate. If the developer could reconstruct the function from its signature alone, it doesn't need a card.

Aim for 3-5 cards per coding session. Fewer is better than more. The developer reviews these daily. Respect their time.

## How to write questions

The question tests decompression: can the developer expand the compressed code back into the full set of requirements and decisions it encodes?

Bad questions test recognition (glancing at code and confirming what it does):
- "What does this function return?"
- "What parameters does this take?"
- "Does this function handle errors?"

Good questions test understanding (reconstructing why the code is the way it is):
- "What happens when [unusual input] reaches this function?"
- "Why does this check [condition] before [operation] instead of after?"
- "What breaks if [assumption] stops being true?"
- "What would you change if [requirement] changed?"

The answer should be specific, referencing actual variable names, conditions, and code paths. Not a generic description.

## Adding cards

For each symbol:

```bash
tate add <file>::<symbol> -q "<question>" -a "<answer>"
```

If `tate add` fails (symbol not found, unsupported language), skip silently.

After adding all cards, print:

```
Added N cards to your Tate deck.
```

## Example

After implementing an authentication module:

```bash
tate add src/auth/login.rs::authenticate \
  -q "What happens when bcrypt comparison succeeds but the user account is locked?" \
  -a "The function checks account.locked after password verification. If locked, it returns AuthError::AccountLocked with the lock reason, not a generic 401."

tate add src/auth/login.rs::refresh_token \
  -q "What is the maximum lifetime of a refresh token chain?" \
  -a "Each refresh rotates the token but increments chain_length. When chain_length exceeds MAX_CHAIN (default 30), the entire chain is invalidated and the user must re-authenticate."

tate add src/auth/login.rs::LoginPayload \
  -q "Why does LoginPayload use a borrowed str for password instead of String?" \
  -a "Avoids allocating the password on the heap where it could linger after free. The borrowed reference is only valid for the request lifetime, reducing the window for memory exposure."
```
