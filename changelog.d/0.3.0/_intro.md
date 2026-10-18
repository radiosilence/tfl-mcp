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
