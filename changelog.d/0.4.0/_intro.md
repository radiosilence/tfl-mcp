### Added

- `Line.route(direction:)` — the stops on a line **in travel order**, with
  branches, stop counts and per-station zones. This is what answers "how many
  stops to Oxford Circus" and "am I going the right way"; `stopPoints` returns
  the same stations unordered and cannot answer either.
- `StopPoint.disruptions` — a closed entrance or a broken lift, as distinct
  from the disruptions affecting the lines that call there.

### Changed

- Reference data is now cached whatever the configuration says, and live data
  still only when asked for. Deciding by what an endpoint *is* rather than by
  its TTL removes the footgun entirely: no setting can make an arrival stale,
  and the vocabulary a model reads before writing its first query stops costing
  a request every time.
- README no longer implies endpoint-level coverage. 84 endpoints exist, many
  are variants of one another, and the graph reaches what answers questions
  rather than one field per endpoint.
