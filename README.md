# Tate

Spaced repetition for code you don't own yet.

## The problem

Tests verify behavior. Code review checks style. Git blame tracks authorship. Nothing tracks whether anyone actually understands the code that shipped.

That gap is invisible until it isn't. A teammate leaves and nobody can touch their module. You inherit a service and spend a week reading before you can change a line. AI generates a function and you ship it because the tests pass. You copy a pattern from Stack Overflow and move on.

The code works. But you can't defend it. Three weeks later, nobody remembers which parts were understood and which were accepted because they looked right.

## What Tate does

Tate is Anki for your codebase. It applies spaced repetition, the most proven learning algorithm we have, to close the gap between "this code exists" and "I can defend this."

The core loop:

1. Code you don't fully understand gets added to your deck (manually, via git hook, or by your AI agent)
2. Decompression questions are attached (not "what does this do?" but "what should this handle that it might not?")
3. You review on schedule, grade your understanding
4. SM-2 algorithm schedules the next review
5. Code changes, card resets, you re-earn it

Understanding decays. Tate brings it back.

## Who it's for

- You shipped AI-generated code without reading every line
- You joined a team and inherited 50k lines you've never seen
- You vendored a library and need to understand the parts you depend on
- You wrote something six months ago and can't explain it anymore
- You copied a pattern and never learned why it works

If there's code in your repo you can't defend, Tate tracks it until you can.

## Usage

```
tate review                     # start today's review session
tate status                     # deck size, cards due, streak
```

### A review session

```
$ tate review

  src/auth/login.ts::authenticate
  Review #3 · Last reviewed 8 days ago

  What happens when the token is expired but the
  refresh token is still valid?

  [1] Blank  [2] Hard  [3] Good  [4] Easy

> 3

  Next review: March 27
  0 cards remaining today.
```

### Managing your deck

```
tate add src/api/routes.ts                   # add an entry
tate add src/auth/login.ts::authenticate     # add a symbol
tate list                                    # show all entries
tate list src/auth/                          # filter by path
tate own src/auth/login.ts::LoginPayload     # mark as owned (leaves rotation)
```

### The deck file

Tate tracks entries in `.tate/deck`. One entry per line, two levels of granularity:

```
# Whole file
src/db/migrations/001.sql

# Symbols (functions, classes, types)
src/auth/login.ts::authenticate
src/auth/login.ts::LoginPayload
```

The deck grows when code enters your repo that you haven't internalized. It shrinks as you learn. But unlike a to-do list, entries come back, because understanding fades.

## Philosophy

> You're the developer. Tate is how you make sure you own what ships.

The industry has tools for writing code faster. Tate is the tool for making sure humans still understand what's been written.

## Agent skill

`skills/tate/SKILL.md` teaches AI coding agents to generate cards automatically after coding tasks. Point your agent at the file or copy it to your agent's skill directory.

## Contributing

Pull requests are welcome.

## License

[MIT](LICENSE)
