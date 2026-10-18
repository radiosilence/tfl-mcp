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
