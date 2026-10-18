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
