+++
title = "Installation"
weight = 1
description = "Install Tate via cargo or build from source."

[extra]
group = "start"
+++

## From crates.io

```
cargo install tate-cli
```

## From source

```
git clone https://github.com/rarescosma/tate.git
cd tate
cargo install --path crates/tate-cli
```

## Initialize in a project

```
cd your-project
tate init
```

This creates the `.tate/` directory, initializes the SQLite database, and installs a git post-commit hook.
