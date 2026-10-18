### Added

- GraphQL schema over the TfL Unified API, joining TfL's foreign keys into a
  graph: `Prediction.line`/`.destination`/`.stopPoint`, `StopPoint.lines`/
  `.arrivals`, `Line.stopPoints`/`.disruptions`. Every edge resolves through a
  DataLoader, so following one across a list is a single batched request.
- MCP server over stdio and streamable-HTTP, exposing `tfl_schema` and `tfl`.
  GraphQL and GraphiQL are available on the same listener.
- `tfl-api-client`, generated from TfL's Swagger document by `cargo xtask regen`.
- CLI: `arrivals`, `status`, `search`, `query`, `schema`, `mcp`, `completions`.
- Journey planning: `journey(from:, to:)` with legs, changes, fares, obstacles
  and accessibility preferences. Handles TfL's `300 Multiple Choices` by
  returning candidate locations rather than failing.
- Santander Cycles: `bikePoint`, `bikePointsNear`, `searchBikePoints`, with
  TfL's property bag parsed into typed counts and a batched occupancy edge.

### Notes

- Loaders cache within a request as well as batching. Batching alone only
  collapses keys that arrive in the same window, so two branches of one query
  asking for the same stop would each fetch it.

- Caching is off by default; transit data goes stale within seconds. When
  enabled it honours only TfL's own `Cache-Control`.
- A blank `app_key` is never sent: TfL answers an invalid key with 429 where an
  anonymous caller gets 200, so sending one is strictly worse than sending none.
