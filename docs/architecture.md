# Architecture

`api-compat-core` owns document loading and bounded reference resolution, the structural compatibility analyzer, scenario and recording models, HTTP execution, assertions, response normalization, findings, release policy, and HTML rendering. `api-compat` owns command parsing, configuration precedence and validation, input/output files, and process exit codes. `examples/ecommerce-api` is a deterministic standard-library HTTP server with baseline, candidate, and compatible modes.

Convert it into Mermaid syntax:

```text
OpenAPI YAML/JSON ──> reference resolution ──> static compatibility ─┐
                                                                    ├─> findings + gate ─> terminal / JSON / HTML
Scenarios ──> baseline capture ──> versioned recordings ──> replay ┘
```

Static contract findings are distinct from observed scenario failures. Replay preserves the request intent recorded from the baseline, applies environment placeholders at execution time, and re-evaluates extracted variables separately in each run. Responses are checked against supported portions of old and new contracts and explicit consumer assertions. Drift normalization is informational only; it does not rewrite assertions or contract validation.

The report records its schema and tool versions, stable contract checksums, summary, consumer outcomes, findings, and scenario coverage. The standalone HTML renderer escapes dynamic text and uses embedded styles without scripts or external assets.
