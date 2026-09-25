# Contributing

Thanks for helping improve `loopkeel`.

## Before opening a change

Please search existing issues first and keep changes focused. For behavior changes, add or update tests and document user-facing CLI changes in `README.md`.

## Development

```sh
cargo fmt -- --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

The project targets stable Rust and should remain portable across supported Unix environments. Do not add provider-specific behavior to the core engine; put integration behavior in adapters or skills.

## Pull requests

Explain the user-visible outcome, testing performed, and any compatibility or checkpoint-format impact. Keep generated runtime files such as `.loop/` out of changes.

By contributing, you agree that your contributions are provided under the MIT License in this repository.
