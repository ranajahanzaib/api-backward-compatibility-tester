# Contributing

Bug reports, documentation improvements, compatibility rules, and code changes
are welcome. For larger changes, open an issue in the
[GitHub repository](https://github.com/ranajahanzaib/api-backward-compatibility-tester)
to discuss the scope before submitting a pull request.

## Development setup

Install the stable Rust toolchain with `rustfmt` and `clippy`, then clone the
repository:

```sh
git clone https://github.com/ranajahanzaib/api-backward-compatibility-tester.git
cd api-backward-compatibility-tester
cargo build --workspace
```

Before opening a pull request, run:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Keep compatibility findings evidence-based. Add regression tests for behavior
changes, document unsupported cases, and avoid including credentials or
production recordings in issues and pull requests. Use concise commit messages
that describe the change.

Pull requests should explain the user-visible behavior, include test results,
and update relevant documentation. The project follows the
[Code of Conduct](CODE_OF_CONDUCT.md). Report security issues using the process
in [SECURITY.md](.github/SECURITY.md), rather than in a public issue.
