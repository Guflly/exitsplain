# Contributing

Focused rules, fixtures, and platform fixes are welcome.

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
```

For a rule:

1. Add it to the closest file in `rules/`.
2. Match a stable error signature, not a vague word.
3. Suggest inspection before mutation. Never add `sudo`, force deletion, or a
   command that hides certificate or permission failures.
4. Add a synthetic test. Do not paste real logs, paths, tokens, or customer
   data.
5. Link official documentation when it exists.

Open an issue before adding a new rule-pack format or changing CLI output.
