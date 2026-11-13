# Report format

`api-compat check` writes `report.json` and `index.html` by default. The JSON
report is versioned with `schema_version: 1`; it contains the tool version,
release decision, findings, scenario results, consumer summaries, contract
fingerprints, operation coverage, and gate policy. Finding classifications are
`confirmed_break`, `static_break`, `potential_break`, `non_breaking`, and
`inconclusive`.

The contract fingerprints use stable FNV-1a checksums to detect when a baseline
recording no longer matches its OpenAPI contract. They are not cryptographic
signatures and do not establish artifact authenticity.

Generate HTML from a saved JSON report without replaying traffic:

```sh
api-compat report \
  --input reports/candidate/report.json \
  --format html \
  --output reports/candidate/index.html
```

The HTML report is standalone, escapes dynamic text, and uses no external
scripts or assets. It presents summary findings, scenarios, consumer impact,
and available operation coverage. It does not include a full before/after JSON
response diff viewer. See [limitations.md](limitations.md) for interpretation
and evidence limits.
