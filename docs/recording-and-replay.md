# Recording and replay

The tool runs only declared scenarios; it does not proxy, crawl, or infer
traffic. A scenario identifies a consumer, request steps, expected statuses,
optional content types and JSON or header assertions. Multi-step scenarios can
capture a JSON Pointer value and interpolate it into a later step with a
`${steps.step-id.variable}` placeholder. Environment placeholders such as
`${API_TOKEN}` are resolved when sending a request.

Capture a baseline recording:

```sh
api-compat capture \
  --base-url http://127.0.0.1:3001 \
  --openapi openapi-old.yaml \
  --cases cases/ \
  --out .api-compat/baseline.json
```

Then replay it against the candidate and combine runtime evidence with static
contract analysis:

```sh
api-compat check \
  --baseline .api-compat/baseline.json \
  --candidate-url http://127.0.0.1:3002 \
  --old-openapi openapi-old.yaml \
  --new-openapi openapi-new.yaml \
  --out reports/candidate
```

Recordings include request and response data, assertions, scenario identity,
and a versioned baseline contract fingerprint. Sensitive headers are redacted
by default. Configure JSON Pointer redaction for sensitive body fields. If
redaction removes request-body data needed to replay a scenario, that scenario
is skipped as inconclusive. Use environment placeholders for replayable
secrets; resolved values are not saved in recordings.

Only GET, HEAD, and OPTIONS are allowed by default. Writes require
`--allow-state-changing`; non-loopback targets require `--allow-remote`. Review
the [safety guide](safety.md) before using live services or production data.
Scenarios are representative checks and do not prove behavior for untested
requests or response values.
