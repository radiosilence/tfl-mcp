# tfl-mcp — MCP and GraphQL server for the TfL Unified API

# What CI checks: format, lint with warnings as errors, tests, and that the
# generated client matches the committed spec.
check:
    actionlint
    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace --locked
    TFL_SPEC_OFFLINE=1 cargo xtask regen
    git diff --exit-code --stat -- crates/tfl-api-client/src/generated

# Format
fmt:
    cargo fmt --all

# Install the git hooks
hooks:
    lefthook install

# Write CHANGELOG.md from the fragments in changelog.d/. `--stdout` prints it instead.
changelog *args:
    python3 scripts/changelog.py {{args}}

# Open the release PR: `just release patch|minor|major`, or an exact version.
# Merging it publishes the image and the GitHub release; tags are never pushed by hand.
release version *args:
    scripts/release.sh {{version}} {{args}}
