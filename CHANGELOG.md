# Changelog

## [Unreleased]

### Changed

- **macOS release binaries are signed with a Developer ID and notarised.** Downloaded through a browser, the ad-hoc signed binaries were quarantined and Gatekeeper refused to run them until the quarantine attribute was cleared.

## [1.3.4] (2026-07-26)

### Fixed

- Linux release binaries now come from `build-image`'s musl compile instead
  of a separate `x86_64-unknown-linux-gnu` build in `build-binaries`, which
  was a second linux amd64 compile every push for no reason. Worse, the musl
  binaries `build-image` already produced were uploaded as raw artifacts that
  `create-release` never collects (it only attaches `**/*.tar.gz`), so they
  were built and then silently discarded. They're now packaged into tarballs
  with the same naming and internal binary name as before, and linux arm64
  gets a release binary for the first time. The `linux-x86_64` asset keeps
  its filename but is now statically linked musl rather than glibc — a
  strict portability improvement, and the same binary the published
  container images already run.

## [1.3.3] (2026-07-26)

### Changed

- Fat LTO (`lto = "fat"`) with a single codegen unit was serialising whole-program optimisation across the entire dependency tree. For an I/O-bound service that gains nothing measurable from it, the cost was maximal: CI builds were taking 6+ minutes per target. Switched to thin LTO and raised codegen units to 16.

## [1.3.2] (2026-07-26)

### Changed

- The container image is now a statically linked musl binary on bare `scratch`
  (~20MB) instead of a `debian-slim` base that recompiled the binary on every
  build. The Dockerfile is a single `COPY`-only stage with no package manager
  and no `cargo build` — CI builds the musl binary once per architecture and
  copies it straight in, so a local `docker build .` requires `dist/` to
  already be populated; that's intentional, docker builds only happen in CI.
  The CA bundle is copied from `gcr.io/distroless/static` (not compiled or
  apt-installed) because `rustls-platform-verifier` needs a system trust
  store for outbound calls to `api.tfl.gov.uk` — confirmed by running the
  static binary on bare `scratch` with no bundle: it panics in
  `Client::new()` with "No CA certificates were loaded from the system"
  rather than falling back to embedded webpki roots.
- CI now builds and runs on every PR — image and binaries included — with
  publishing (registry push, GitHub release) gated to pushes on `main`. A
  broken Dockerfile or build now fails before merge instead of after.
- Docker layer caching was dropped from the image job: nothing compiles
  inside the image anymore, and `mode=max` was filling the repo-wide 10GB
  GitHub Actions cache shared with `Swatinem/rust-cache`.

## 1.3.1

### Fixed

- List **query** parameters are sent comma-joined where TfL's own description
  says they must be, rather than as a repeated key. `journey(accessibility:)`
  returned 400; so did `journey(modes:)`, which nothing had exercised. Eleven
  parameters carry the contradiction — the spec declares `collectionFormat:
  multi` while the description in the same object says "comma separated list",
  and the repeated form really does fail.
- The README's journey example passed a place name and showed routes coming
  back. TfL calls "Kings Cross" ambiguous and returns candidates instead, so
  the documented query returned nothing. It now asks for `isAmbiguous` and the
  options alongside `journeys`, and the server instructions no longer claim
  names resolve directly.

## 1.3.0

### Added

- `places`, `place`, `searchPlaces` and `placeTypes` — car parks, taxi ranks,
  cycle parks, coach bays and charge stations. TfL's `Place` domain was
  unreachable: the type was used everywhere, as bike points and as a stop's
  children, so it looked covered while having no entry point of its own.

### Fixed

- `every_domain_tfl_documents_is_reachable` now derives the domain list from
  the committed spec. It previously listed them from memory, so it passed while
  `Place` was unreachable — a test asserting what its author already believed.
  It also now fails loudly if TfL adds a domain nobody has ruled on.

## 1.2.0

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

## 1.1.0

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

## 1.0.1

Identical to 1.0.0 in behaviour. The 1.0.0 tag was cut from the commit before
the rename, so its image published to the old `tfl-cli` path; this is the first
release under `ghcr.io/radiosilence/tfl-mcp` and the one to pin.

## 1.0.0

Breaking, and the surface is settled enough to say so.

### Changed

- **Renamed from `tfl-cli` to `tfl-mcp`**, including the published image, which
  moves to `ghcr.io/radiosilence/tfl-mcp`. Its siblings earn the `-cli` suffix
  — you do want to send mail from a terminal — but nobody checks the tube from
  a shell when they could ask an assistant. The binary is still `tfl`.

- **Serving is the default.** `tfl --http 0.0.0.0:8080` replaces `tfl mcp
  --http …`, and the `mcp` subcommand is gone rather than deprecated. Anything
  invoking it must adapt; keeping a compatibility alias alive from the first
  day of a 1.0 sets the wrong precedent. Deployments must therefore move their
  image tag and their arguments together.
- **Removed the `arrivals`, `status` and `search` subcommands.** They were
  GraphQL queries assembled by pasting strings together: a second
  implementation of what `tfl query` already does, carrying its own escaping so
  that a station called `King's Cross` could not end the literal early. Nobody
  would type them when they could ask an assistant.
- **Every read now takes the same path** — resolver, loader, request. The
  argument-less feeds (the `Meta` vocabularies, roads, air quality, charge
  connectors, car parks, bike points) went straight to the client, which meant
  two branches of one query each fetched them. They share a loader now, so
  `{ a: modes { name } b: modes { name } }` costs one request rather than two.
  Closes the last case where identical reads in a single query did not
  deduplicate.

## 0.5.1

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

## 0.5.0

### Added

- `StopPoint.directionTo` — inbound or outbound to reach another stop. Every
  field taking a `direction` argument wants exactly this, and a model was
  previously guessing.
- `StopPoint.canReachOnLine` — where you can get to without changing.
- `now` — the current time in London, with offset and whether the tube is
  likely running. TfL's timestamps are London-local with no zone marker, so
  "is this departure soon" was unanswerable without knowing what time it is
  there. Costs no request.

### Fixed

- TfL declares booleans it then sends as strings — `StopPoint.status` comes
  back as `"Unknown"` from `/CanReachOnLine`. serde fails a whole response on
  one bad field, so a single stop with an opinion about its status lost the
  other fourteen. Generated booleans now accept either, and an unrecognised
  word decodes as null rather than guessing `false`.
- List decoding no longer hides the real error. `#[serde(untagged)]` reported
  only "data did not match any variant", throwing away which field and which
  line — every decode failure in the client had become undiagnosable.

## 0.4.0

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

## 0.3.1

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

## 0.3.0

### Added

- Roads: corridors and disruptions, worst-first, with `corridorIds` resolved
  into the roads a disruption blocks.
- Air quality: forecast bands per pollutant, with TfL's escaped HTML decoded.
- Cabwise: licensed taxi and minicab operators near a point.
- Occupancy: EV charge connectors (batched) and car parks.
- AccidentStats: 2019 casualty records, filtered locally by radius, severity
  and borough because TfL offers no filter of its own.

### Fixed

- The generator emitted an empty struct for `System.Object`, which parsed
  successfully and discarded the entire payload. Definitions with no properties
  now decode as raw JSON, which is what made air quality and Cabwise reachable
  at all.

## 0.2.1

### Fixed

- Tool descriptions and server instructions never mentioned journey planning or
  Santander Cycles, which landed after they were written — so a model had no way
  to know either existed. The tool description matters most: it is always
  loaded, and is what decides whether the server gets reached for at all.

## 0.2.0

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
