# CI integration

`.github/workflows/ci.yml` checks formatting, Clippy, workspace tests, and static breaking/compatible fixtures. `.github/workflows/compatibility-demo.yml` builds and starts the deterministic example servers, performs real capture/replay, verifies the breaking check exits 2 and the compatible check passes, then uploads JSON and HTML reports.

For a consuming project, prepare a baseline recording against a controlled baseline API, store credentials in CI environment variables, and run:

```sh
api-compat check \
  --baseline recordings/staging-baseline.json \
  --candidate-url http://127.0.0.1:3002 \
  --old-openapi openapi-old.yaml \
  --new-openapi openapi-new.yaml \
  --out reports/
```

Use `${API_TOKEN}` placeholders in scenario headers. The tool never persists resolved placeholder values. Reports are created even when findings block the release. Exit 3 indicates a required inconclusive or execution check; do not treat it as a pass.
