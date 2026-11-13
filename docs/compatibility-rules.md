# Implemented compatibility rules

| Change                                                            | Classification                      |
| ----------------------------------------------------------------- | ----------------------------------- |
| Existing path/HTTP method removed                                 | `static_break`                      |
| Path template parameter names differ but route shape is unchanged | Matched by normalized path position |
| New required query/header/path parameter                          | `static_break`                      |
| Existing query/header/cookie parameter removed                    | `potential_break`                   |
| Parameter serialization changes                                   | `static_break`                      |
| Request type or allowed-type set narrows                          | `static_break`                      |
| Request enum narrows or an enum restriction is introduced         | `static_break`                      |
| New required request body/property                                | `static_break`                      |
| Request constraints become stricter                               | `static_break`                      |
| Request accepts more types/enum values or looser bounds           | No break                            |
| Response property type or required guarantee narrows              | `static_break`                      |
| Optional response property removed                                | `potential_break`                   |
| Response enum expands or is removed                               | `static_break`                      |
| Candidate response violates old supported schema/assertion        | `confirmed_break`                   |
| Candidate response violates new contract                          | `potential_break`                   |
| Security adds requirements/removes an old alternative             | `static_break`                      |
| Status/media type/documented response header removed              | `static_break`                      |
| Added endpoint or optional response property                      | No finding by default               |
| Unsupported reference/composition or failed request setup         | `inconclusive`                      |

The rules are directional: candidate requests must continue accepting old client inputs, while candidate responses must preserve the old guarantees. This is not a complete formal compatibility relation. See [limitations](limitations.md) for unsupported constructs and OpenAPI dialect differences.
