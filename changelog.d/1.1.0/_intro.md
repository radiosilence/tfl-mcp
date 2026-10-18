### Fixed

- `tfl --graphiql` used to hang. The web flags only did anything alongside
  `--http`, so without it you got the stdio MCP server sitting on stdin, which
  looks exactly like a hang. Each flag now brings up what it needs: `--browser`
  implies `--graphiql`, which implies `--graphql`, and any of them starts a
  listener.
- Added `--browser`, which previously did not exist — opening one was welded to
  `--graphiql`.
- `--http` now takes an optional address, defaulting to `127.0.0.1:8080`.

`--http` remains the only flag that puts **MCP** on HTTP. Someone poking at
GraphiQL on a laptop has not asked to expose an MCP endpoint, and the default
address is loopback rather than all interfaces for the same reason.
`--http 0.0.0.0:8080 --graphql`, as deployed, is unchanged in meaning.

### Added

- `StopPoint.crowding` and `StopPoint.crowdingOn(day:)` — how busy a station is
  right now, and across a normal day in quarter-hours. Answers "is Oxford
  Circus hell at the moment" and "when is the quietest time to travel".

  None of this is in TfL's Swagger document. The endpoint the spec *does*
  describe returns a plain stop point with no crowding in it, which is why this
  looked withdrawn rather than merely undocumented.

  Every figure is relative to that station's own normal, not a headcount, and
  the field descriptions say so — `0.17` means a sixth of usual traffic, and
  the numbers are not comparable between stations.
