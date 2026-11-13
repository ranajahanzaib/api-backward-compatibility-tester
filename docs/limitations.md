# Compatibility scope and limitations

The parser accepts OpenAPI 3.0, 3.1, and 3.2 YAML/JSON, resolves local references and relative files beneath the contract directory, and stops cyclic references at a bounded depth. It does not fully validate every OpenAPI object or implement all JSON Schema dialect rules. References it cannot resolve and compositions it cannot compare produce inconclusive findings.

Static comparison covers operation removal, path/method identity, parameters, basic request/response schemas, enums, nullability, selected numeric/string/array bounds, response statuses/media types/headers, and security tightening. It does not fully model `oneOf`, `anyOf`, `not`, discriminators, all format semantics, every exclusive-bound dialect, pattern properties, response wildcard equivalence, or all serialization rules. Complex changes need manual review.

Runtime checks execute only supplied scenarios. The engine checks the observed response against supported portions of both contracts, runs explicit status/header/content-type/JSON assertions, and reports normalized response drift as informational. Normalization does not suppress explicit assertion failures. Scenario coverage is not exhaustive and unmodeled workflows remain untested.

The versioned recording bundle stores an FNV-1a contract checksum for stale-recording detection; this is a reproducibility checksum, not a cryptographic hash. Capture time is stored as Unix seconds. Legacy array recordings are accepted with an inconclusive warning.

The current CLI executes scenarios sequentially. It does not yet provide automatic retries, authentication-scheme configuration beyond header placeholders, multi-host allowlists, API discovery, or complete CI annotations. State-changing replay can mutate data and should only run in a controlled environment.

No tool can infer undocumented business rules, pricing calculations, authorization decisions, rate limits, side effects, eventual consistency, sorting behavior, or unrecorded client expectations. A clean result means only that the checks actually run passed under the configured policy.
