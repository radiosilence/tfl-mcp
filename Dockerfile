# tfl as a container. The default command runs the MCP server over HTTP — this
# is the image mcp-gateway proxies to as a backend, with the TfL app key
# arriving per request via the X-Tfl-App-Key header. No key is also fine: TfL
# serves anonymous callers at 50 requests/minute.
#
# No build stage, no package manager: CI builds the static musl binaries and
# this image only copies them in. The build context is dist/, holding
# <arch>/tfl for each of amd64 and arm64 (the release tarballs, unpacked), so
# a build by hand needs `docker build -f Dockerfile dist` with those in place.
# That is intended: docker builds only ever happen in CI.

FROM scratch

ARG TARGETARCH

# VALIDATED, DO NOT "SIMPLIFY" THIS AWAY:
# rustls-platform-verifier requires a system trust store on Linux. It does NOT
# fall back to the webpki roots compiled into the binary. With no CA bundle on
# disk, reqwest panics before making a single request:
#   Client::new(): reqwest::Error { kind: Builder,
#     source: General("No CA certificates were loaded from the system") }
# Verified by running the static binary on bare scratch against a real HTTPS
# host: without this line it panics; with it, TLS completes and the server's
# own auth response comes back. Sourced from distroless/static so we need no
# package manager and no build stage — it is a plain copy from a published,
# CVE-maintained image.
COPY --from=gcr.io/distroless/static:latest \
     /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt

COPY ${TARGETARCH}/tfl /tfl

EXPOSE 8080

# scratch has no /etc/passwd, so this must be a raw numeric uid rather than a
# name — matches the uid the previous debian-slim image's `app` user had.
USER 10001:10001

LABEL org.opencontainers.image.title="tfl-mcp"
LABEL org.opencontainers.image.vendor="James Cleveland"
LABEL org.opencontainers.image.licenses="MIT"
LABEL org.opencontainers.image.source="https://github.com/radiosilence/tfl-mcp"

ENTRYPOINT ["/tfl"]
CMD ["--http", "0.0.0.0:8080", "--graphql"]
