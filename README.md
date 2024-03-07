# Hivemind

A fully-featured implementation of the game Hive, as well as:
- [a Universal Hive Protocol (UHP) server](https://github.com/jonthysell/Mzinga)
- a strong AI player (coming soon...)

# Usage
```bash
$ just run
```

or:
```bash
$ touch .env # Mandatory, even if empty
$ cargo run --release
```

# Development

Just clone and hack away!

`hivemind`'s `justfile` uses some non-standard utilities.
1. `cargo pretty-test`
2. `cargo flamegraph`