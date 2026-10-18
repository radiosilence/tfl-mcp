Released 2026-07-26.

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
