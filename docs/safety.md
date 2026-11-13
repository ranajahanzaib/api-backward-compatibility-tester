# Replay safety

Capture and replay are real HTTP requests. GET, HEAD, and OPTIONS are permitted by default; POST, PUT, PATCH, and DELETE need `--allow-state-changing`. Do not treat a read-like method as harmless for every API. Use an isolated environment and disposable data.

Targets must use HTTP or HTTPS. Loopback hosts are allowed by default; remote hosts require `--allow-remote`. Redirects are disabled, TLS verification is enabled, the default per-request timeout is 10 seconds, and responses are capped at 2 MiB. Requests are not automatically retried. The current request count is sequential and total execution time is bounded by the per-request timeout multiplied by scenario count.

Environment placeholders such as `${API_TOKEN}` are resolved only for the outbound request; recordings retain the placeholder. Authorization, Cookie, and Set-Cookie headers are redacted by default. Configured JSON Pointers are redacted before request/response bodies are persisted. If redaction removes a request-body value needed for replay, the scenario is marked skipped/inconclusive instead of sending a fabricated value. Redaction can reduce comparison coverage.

Never commit real credentials, sensitive recordings, or reports containing production data. Explicitly review any custom redaction configuration; the built-in sensitive-header list is always retained.
