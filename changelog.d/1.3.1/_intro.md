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
