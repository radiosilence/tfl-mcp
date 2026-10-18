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
