# syntax=docker/dockerfile:1.7
#
# Image for the kind e2e's mock GitHub server ONLY. Built and `kind load`ed by
# scripts/e2e.sh; never pushed to a registry. Kept separate from the production
# image (the repo-root Dockerfile) so the test double is never shipped to prod.
#
# The `backend` stage below is byte-identical to the root Dockerfile's `backend`
# stage on purpose: `cargo build --release` already builds every binary target
# (both `featuredoc` and `mock_github`), so when e2e builds the app image first,
# BuildKit reuses this stage from cache and only the runtime copy below runs — no
# second compile.

# ---- build ----
FROM rust:1.94-slim-bookworm AS backend
WORKDIR /app
COPY backend/Cargo.toml backend/Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && \
    cargo build --release && rm -rf src target/release/featuredoc*
COPY backend/src ./src
# migrations are embedded into the crate at compile time by sqlx::migrate!, so the
# library (which the mock binary links) needs them present to compile.
COPY backend/migrations ./migrations
RUN touch src/main.rs && cargo build --release

# ---- runtime: just the mock binary, nothing else ----
FROM debian:bookworm-slim AS runtime
WORKDIR /app
COPY --from=backend /app/target/release/mock_github /usr/local/bin/mock_github
EXPOSE 8090
CMD ["/usr/local/bin/mock_github"]
