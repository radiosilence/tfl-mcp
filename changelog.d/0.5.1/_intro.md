### Fixed

- `StopDisruption.isBlocked` could only ever return `false` — both branches of
  its expression produced the same value. "Is this station closed" answered
  confidently and wrongly, with no error. Replaced by `isClosed` and
  `closureText`, derived from the fields that actually carry closure state,
  and documented so a partial closure is not read as a shut station.
- The MCP tool description and instructions still listed only the domains that
  existed two releases ago, so nothing signalled that roads, air quality,
  taxis, charge points, car parks or collision history were reachable at all.
- `vehicleArrivals` claimed to batch at 20 and did not chunk. Now sent 25 per
  request, TfL's documented maximum.
- `accidents` claimed "nearest first" but ranked every record equally when no
  coordinate was given, making `first` an arbitrary slice. It now orders by
  date in that case and says so.
- `journey`'s `accessibility` argument listed three of TfL's six accepted
  values, so "avoid escalators" had no visible way to be asked for.
- `severities` did not say it is the vocabulary for *lines* only; road
  disruptions grade in words. Added `roadSeverities`.
- `carParks` now says TfL carries no coordinates on that feed, so "the nearest
  car park" is knowingly unanswerable rather than quietly wrong.
- Complexity multipliers added to `chargeConnectors` and `carParks`, the two
  largest unfiltered lists, which carried none.
- A failed fetch no longer reads as a confident empty answer. `load_batch`'s
  retry path discarded every error and returned `Ok` regardless, so a revoked
  key produced twenty nulls rather than a failure; and a per-key failure was
  dropped from the loader's map, which every resolver turned into an empty
  list — so one stop's arrivals timing out beside working siblings reported
  "no trains due", and a failed disruption fetch reported good service.
- The journey rescue search never ran when TfL returned no candidates, which
  is the case it exists for.
