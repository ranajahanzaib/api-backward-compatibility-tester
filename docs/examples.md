# Examples

The [`ecommerce-api`](../examples/ecommerce-api/) fixture contains baseline,
breaking-candidate, and compatible-candidate modes. Its OpenAPI files and
consumer cases are intentionally deterministic and use fictional data. Run
them together with:

```sh
./scripts/demo.sh
```

The sample [web-client scenarios](../examples/ecommerce-api/cases/web-client.yaml)
show a customer-profile assertion, a two-step pagination flow using an
extracted cursor, and an invalid-customer response expectation. The fixture
does not represent a production API or establish behavior beyond those cases.

For a new project, `api-compat init` generates a minimal one-request case and a
starter configuration. Replace the example path and expected response with
behavior that the actual consumer depends on. See
[recording-and-replay.md](recording-and-replay.md) for capture and replay.
