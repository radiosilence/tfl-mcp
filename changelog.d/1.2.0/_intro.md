### Added

- `Line.timetable(from:, direction:)` — scheduled departures, so **"when is the
  last train"** finally has an answer. `route` gives the stops in order but
  carries no times at all.

  The endpoint was documented; the way to call it was not. `direction` is a
  query parameter absent from the Swagger document entirely, and the docs imply
  a path segment, which 404s. Without it TfL replies asking which way you meant
  — at HTTP 200, so it cannot be detected by status. That surfaces as
  `isAmbiguous` rather than an error.

  Times are wrapped into real clock times. TfL sends hour 27 for a 03:13 Night
  Tube train, and reporting that raw would describe a train at twenty-seven
  o'clock; `isNextDay` says which day it leaves, and
  `minutesAfterMidnight` keeps the unwrapped figure for ordering across the
  boundary.

### Fixed

- **A stale `Cargo.lock` no longer reaches the image build.** `check` now tests
  with `--locked` as well as building with it, so a lockfile that has drifted
  from `Cargo.toml` fails in the pull request rather than in the Docker build.
- **A version tag is never cut without an image behind it.** `publish` now waits
  for the image builds as well as the tarballs. Previously they ran in parallel,
  so a failed image build still produced a GitHub release with nothing to pull.
