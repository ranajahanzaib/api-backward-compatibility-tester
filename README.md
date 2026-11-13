# API Backward Compatibility Tester

**Catch API changes that break existing clients before deploying them.** `api-compat` is a local Rust tool that compares OpenAPI contracts and replays declared consumer scenarios against a candidate API.

## What works

- Parse YAML/JSON OpenAPI 3.0, 3.1, and 3.2 documents and resolve bounded local and relative-file `$ref`s.
- Find removed operations, stricter request parameters and schemas, changed response guarantees, security tightening, media type/status changes, and unsupported comparisons.
- Capture versioned baseline recordings and replay multi-step scenarios with environment-variable substitution, JSON Pointer assertions, and cursor extraction.
- Check candidate responses against the old and new response contracts and report consumer impact.
- Write a versioned JSON report and standalone HTML report, with configurable release gates.
- Run a deterministic local ecommerce demonstration with baseline, breaking, and compatible API modes.

The analyzer covers a useful subset of OpenAPI and JSON Schema; see [limitations](docs/limitations.md) and [implemented rules](docs/compatibility-rules.md). A passing result is evidence for the checks and scenarios supplied, not a proof of complete semantic compatibility.

## Quick start

Requires stable Rust and `curl` for the demo. Build and run the full local demonstration:

```sh
cargo build --workspace
cargo run -p api-compat -- demo
```

The script starts all three local API modes, records the baseline scenarios, confirms the breaking candidate is blocked with exit code 2, confirms the compatible release passes, and writes reports under `reports/`.

See [getting started](docs/getting-started.md) for requirements and a first contract comparison.

## Use the CLI

```sh
cargo run -p api-compat -- --help
cargo run -p api-compat -- spec diff \
  --old examples/ecommerce-api/openapi/baseline.yaml \
  --new examples/ecommerce-api/openapi/candidate.yaml

cargo run -p api-compat -- capture \
  --base-url http://127.0.0.1:3001 \
  --openapi examples/ecommerce-api/openapi/baseline.yaml \
  --cases examples/ecommerce-api/cases \
  --out .api-compat/baseline.json

cargo run -p api-compat -- check \
  --baseline .api-compat/baseline.json \
  --candidate-url http://127.0.0.1:3002 \
  --old-openapi examples/ecommerce-api/openapi/baseline.yaml \
  --new-openapi examples/ecommerce-api/openapi/candidate.yaml \
  --out reports/candidate

cargo run -p api-compat -- report \
  --input reports/candidate/report.json \
  --format html --output reports/candidate/index.html
```

An individual scenario can use a simple one-request form:

```yaml
name: customer lookup
consumer: web-client
method: GET
path: /customers/cust_123
expect_status: 200
assertions:
  - pointer: /customer_id
    type: string
    required: true
```

For multi-step cases, use `id`, `consumer`, `steps`, `request`, `expect`, and optional `capture` fields as shown in [web-client.yaml](examples/ecommerce-api/cases/web-client.yaml). Step variables are scoped to a scenario and re-extracted independently during baseline capture and candidate replay.

## Configuration and gate behavior

`api-compat init` creates `api-compat.yaml` and a sample case without overwriting existing files. CLI arguments override environment variables, config, then defaults for supported settings. Unknown configuration keys are rejected. `--timeout`, `--strict`, `--quiet`, `--fail-on`, and `--allow-remote` are supported where applicable. Output is plain text without color codes. Replay is sequential (`max_concurrency: 1`) in this release.

Exit codes: `0` passed, `1` invalid arguments/configuration, `2` release blocked by compatibility findings, `3` required checks were inconclusive or could not execute. Defaults block on confirmed/static breaks and inconclusive checks. Set `policy.fail_on` to adjust classifications; inconclusive checks still block by default.

## Safety

Only GET, HEAD, and OPTIONS run by default. State-changing methods need `--allow-state-changing`; remote hosts need `--allow-remote`. Redirects are disabled, TLS verification remains enabled, the default timeout is 10 seconds, and response bodies are capped at 2 MiB. Credentials can use `${ENV_VAR}` placeholders and are not persisted after substitution. Authorization, Cookie, and Set-Cookie headers are redacted by default. Configure JSON Pointer redaction for sensitive bodies and responses; scenarios whose request bodies were redacted are marked skipped during replay. See [safety details](docs/safety.md).

## Reports and CI

JSON reports include the schema/tool versions, decision summary, finding classifications, scenarios, consumer summaries, contract checksums, and operation coverage. The checksum currently uses stable FNV-1a for mismatch detection; it is not a cryptographic signature. HTML reports are standalone and escape response-derived content.

The workflows in `.github/workflows/` run formatting, Clippy, tests, contract fixtures, and the full local compatibility demo. CI publishes the demo reports as an artifact. For a consuming project, provide its baseline recording and run `api-compat check` against the candidate after deployment to an isolated test environment.

## Development

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

See [configuration](docs/configuration.md), [recording and replay](docs/recording-and-replay.md), [report format](docs/report-format.md), [examples](docs/examples.md), [architecture](docs/architecture.md), [CI setup](docs/ci-integration.md), and [contributing](CONTRIBUTING.md). Licensed under MIT.
