Released 2026-07-26.

### Changed

- Fat LTO (`lto = "fat"`) with a single codegen unit was serialising whole-program optimisation across the entire dependency tree. For an I/O-bound service that gains nothing measurable from it, the cost was maximal: CI builds were taking 6+ minutes per target. Switched to thin LTO and raised codegen units to 16.
