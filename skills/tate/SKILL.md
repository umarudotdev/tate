---
name: tate
description: "Generate spaced repetition cards for code you just wrote. Use this skill after completing any coding task (feature, bug fix, refactoring) where reasoning was compressed into implementation. Also trigger when the user says 'add to tate', 'create cards', or 'track this code'. Initializes tate automatically if needed. Creates review cards with decompression questions so the developer can verify they understand what was shipped."
---

# Tate: Generate review cards for code you wrote

After finishing a coding task, create spaced repetition cards for the code you wrote or significantly modified. Each card has a question (front) and answer (back) that test whether the developer can decompress the code back into the reasoning behind it.

## Prerequisites

Check that `tate` is in PATH. If not, skip silently. If `.tate/` doesn't exist, run `tate init` to set it up, then continue with card generation.

## When NOT to trigger automatically

When triggering automatically after a task, skip:
- Typo fixes, formatting, import reordering
- Dependency version bumps
- Renaming without behavior change
- Deleting dead code

When the user explicitly asks (says "add to tate", "/tate", "create cards"), always generate cards for the meaningful work in the session, even if the most recent changes are trivial. Look at the full session, not just the last commit.

## What to card

You know what you wrote because you just wrote it. Don't diff or scan. Card the symbols where the reasoning matters:

- Functions with non-obvious error handling or edge cases
- Types where the field choices encode domain constraints
- Algorithms where the approach was chosen over alternatives
- Integration points where assumptions about external behavior are baked in

Skip trivial getters, simple constructors, one-line wrappers, and boilerplate. If the developer could reconstruct the function from its signature alone, it doesn't need a card. Only card code the developer will maintain or depend on. A utility function the developer will never change or debug doesn't need a card, even if it's complex.

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

One question, one reasoning chain. If you're joining two questions with "and," that's two cards. The developer can't grade a compound question honestly - they might nail one half and blank on the other. Each question can be as long as the reasoning demands. The problem is compound questions, not long questions.

No enumerations. Don't ask "What are the three conditions this function checks?" Order-dependent recall is fragile - the developer will memorize the list order, not the reasoning. Ask about each condition's purpose individually.

Cards must connect to the developer's work. Don't card isolated implementation details the developer will never touch again. Every card should relate to code on the developer's critical path. Orphan cards erode the review habit.

## How to write answers

The answer should give the reviewer something they can't get from re-reading the code. The code shows *what*. The answer explains *why*.

Shallow answer (restates the code):
> "The function checks if the account is locked and returns an error."

Deep answer (explains the reasoning):
> "The function checks account.locked after password verification, not before. This ordering prevents timing attacks: an attacker can't distinguish 'wrong password' from 'locked account' by response time. The lock reason is included in the error so the support team can tell the user why without a DB lookup."

The deep answer contains three things the code doesn't say: why the ordering matters, what attack it prevents, and who uses the lock reason downstream. That's what a card should teach.

## Adding cards

For named symbols:

```bash
tate add <file>::<symbol> -q "<question>" -a "<answer>"
```

For code without named symbols (CSS rules, YAML blocks, SQL migrations, config sections, anonymous callbacks), use line ranges:

```bash
tate add <file>:<start>-<end> -q "<question>" -a "<answer>"
```

If `tate add` fails (symbol not found, unsupported language, range out of bounds), skip silently.

After adding all cards, print:

```
Added N cards to your Tate deck.
```

## Example

After implementing an authentication module:

```bash
tate add src/auth/login.rs::authenticate \
  -q "What happens when bcrypt comparison succeeds but the user account is locked?" \
  -a "The function checks account.locked after password verification, not before. This ordering prevents timing attacks: an attacker can't distinguish 'wrong password' from 'locked account' by response time. The lock reason is included so support can tell the user why without a DB lookup."

tate add src/auth/login.rs::refresh_token \
  -q "What is the maximum lifetime of a refresh token chain?" \
  -a "Each refresh rotates the token but increments chain_length. When chain_length exceeds MAX_CHAIN (default 30), the entire chain is invalidated and the user must re-authenticate. This bounds the window of exposure if a token is stolen mid-chain."

tate add src/styles/reset.css:1-12 \
  -q "Why does the reset zero margin on body but not on html?" \
  -a "html margin is used by some browsers for scroll anchoring. Zeroing it breaks scroll-to-top on navigation in Safari. body margin is safe to zero because no browser behavior depends on it."
```
