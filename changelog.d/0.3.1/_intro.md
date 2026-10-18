### Fixed

- **A query could fan out to hundreds of concurrent requests.** Depth was
  capped but width was not, so `linesByMode(modes: ["bus"]) { stopPoints }` —
  two levels deep, 676 lines wide — would have fired 676 simultaneous requests
  and burned a minute of TfL's rate limit in one call. Fan-out fields now
  declare their width and the schema refuses the product; the client also caps
  requests in flight, which guards paths nobody anticipated.
- Complexity costs overflowed a few levels of nesting in. Debug panicked;
  release wrapped to a small number, so the limit silently stopped limiting.
  Each field's cost is now clamped.
- `hasBikes`/`hasDocks` ignored whether a station was locked or uninstalled.
  TfL leaves stale counts on stations it has pulled, so `bikePointsNear` could
  send someone to a locked dock reporting four bikes.
- An ambiguous journey with no usable candidates read as a confident "no route
  exists" rather than as ambiguous. Ambiguity is now recorded when TfL says so
  rather than inferred from the candidate list being non-empty.
- The response cache and the startup credential check were built and never
  wired to anything. `TFL_CACHE=1` enables the cache; a bad app key now fails
  when the server starts rather than as unexplained 429s on the first query.
- Poisoned-mutex tolerance: one panicking request no longer bricks every
  subsequent one.
