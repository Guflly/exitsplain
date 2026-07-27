# exitsplain

Your command failed. Get a useful explanation without sending the log anywhere.

```sh
exitsplain run -- npm install
```

`exitsplain` streams the command normally, preserves its exit code, and adds a
short diagnosis only when it fails.

```text
exitsplain: 1 likely explanation

1. npm found incompatible peer dependencies [97%]
   Two packages require peer versions that cannot be selected together.
   Try:
   - Read both conflicting version ranges in the ERESOLVE report.
   - Choose package versions whose peer ranges overlap.
```

It is deterministic, offline, and never runs a suggested fix.

## Install

```sh
cargo install --git https://github.com/Guflly/exitsplain
```

Prebuilt binaries will be attached to tagged releases.

## Other commands

Analyze a saved log or stdin:

```sh
exitsplain analyze build.log
cat build.log | exitsplain analyze --format json
```

Browse the built-in catalog:

```sh
exitsplain rules
exitsplain rules --tag docker
```

The first release ships 28 rules for common shell, npm, Python, Git, Docker,
Rust, Go, Java, and Gradle failures.

## Contributing

New rules need a synthetic failing fixture, a safe action, and an official
reference when one exists. See [CONTRIBUTING.md](CONTRIBUTING.md).

MIT
