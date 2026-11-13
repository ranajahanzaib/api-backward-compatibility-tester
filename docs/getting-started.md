# Getting started

## Requirements

- Rust stable with `rustfmt` and `clippy`
- `curl` to run the shell-based demonstration

Build the workspace and inspect the CLI:

```sh
cargo build --workspace
cargo run -p api-compat -- --help
```

## Run the demonstration

```sh
cargo run -p api-compat -- demo
```

The demo starts three local fixture APIs, captures baseline scenarios, blocks a
breaking candidate, and checks a compatible candidate. It writes the recording
to `.api-compat/` and reports to `reports/`. The demo uses loopback ports 3001,
3002, and 3003; stop any services already using those ports before starting it.

## Check contracts

To compare specifications without making HTTP requests:

```sh
cargo run -p api-compat -- spec diff \
  --old examples/ecommerce-api/openapi/baseline.yaml \
  --new examples/ecommerce-api/openapi/candidate.yaml
```

Exit status 2 indicates a release-blocking finding. The supported rules and
their limits are described in [compatibility-rules.md](compatibility-rules.md)
and [limitations.md](limitations.md).

For a consuming API, capture and check scenarios as described in
[recording-and-replay.md](recording-and-replay.md). Configuration fields are
listed in [configuration.md](configuration.md).
