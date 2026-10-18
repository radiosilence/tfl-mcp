Released 2026-07-26.

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
