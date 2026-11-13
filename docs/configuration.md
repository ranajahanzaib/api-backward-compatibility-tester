# Configuration

`api-compat` reads `api-compat.yaml` from the current directory when present.
Pass `--config PATH` to select another file. Relative paths in the file are
resolved from its parent directory. `api-compat init` creates a starter file;
`api-compat init --force` replaces the generated config and sample case.

CLI options take precedence over supported environment variables, which take
precedence over configuration values and defaults. Environment overrides are
`API_COMPAT_BASELINE_URL`, `API_COMPAT_CANDIDATE_URL`, and
`API_COMPAT_TIMEOUT_SECONDS` where the corresponding command setting applies.
Unknown configuration keys are rejected.

```yaml
version: 1
project:
  name: storefront
contracts:
  baseline: openapi-old.yaml
  candidate: openapi-new.yaml
targets:
  baseline:
    base_url: http://127.0.0.1:3001
  candidate:
    base_url: http://127.0.0.1:3002
scenarios:
  directory: cases/
replay:
  timeout_seconds: 10
  max_concurrency: 1
  max_request_bytes: 1048576
  max_response_bytes: 2097152
  allow_state_changing: false
  allow_remote: false
normalization:
  ignore_json_pointers: []
  ignore_headers: [date, server]
  ignore_array_order: []
redaction:
  headers: [authorization, cookie, set-cookie]
  json_pointers: []
policy:
  fail_on: [confirmed_break, static_break]
  fail_on_inconclusive: true
  require_successful_replay: true
  require_nonempty_scenarios: true
  require_valid_baseline: true
reports:
  directory: reports/
  formats: [json, html]
```

Replay is sequential, so `max_concurrency` must be `1`. Request and response
body limits are byte counts. Remote targets and state-changing requests remain
disabled unless enabled in configuration or with the corresponding CLI flag.
`fail_on` accepts evidence classifications or `none`. Inconclusive results
block by default; `--strict` also requires successful checks and nonempty
scenario coverage.

The configuration surface is intentionally narrower than the full OpenAPI
specification. See [limitations.md](limitations.md) for analysis boundaries.
