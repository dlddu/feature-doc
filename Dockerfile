# syntax=docker/dockerfile:1.7

# ---- stage 1: frontend ----
FROM node:22-bookworm-slim AS frontend
WORKDIR /app
COPY frontend/package.json frontend/package-lock.json* ./
RUN if [ -f package-lock.json ]; then npm ci; else npm install; fi
COPY frontend/ ./
RUN npm run build

# ---- stage 2: backend ----
FROM rust:1.94-slim-bookworm AS backend
WORKDIR /app
COPY backend/Cargo.toml backend/Cargo.lock ./
# Prime the dependency cache. Both [[bin]] targets declared in Cargo.toml must
# exist for this stub build to resolve, hence the second placeholder.
RUN mkdir -p src/bin && echo "fn main() {}" > src/main.rs && \
    echo "fn main() {}" > src/bin/worker.rs && \
    cargo build --release && rm -rf src target/release/featuredoc*
COPY backend/src ./src
# migrations are embedded into the binary at compile time by sqlx::migrate!,
# so they must be present for the real build (but not in the runtime image).
COPY backend/migrations ./migrations
RUN touch src/main.rs && cargo build --release

# ---- stage 3: runtime ----
FROM debian:bookworm-slim AS runtime
RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates && \
    rm -rf /var/lib/apt/lists/*
WORKDIR /app
# One image, two workloads (AC4.5): the API is the default command and the worker
# Deployment overrides it. Sharing the image keeps the two in lockstep — they
# speak the same /internal contract, so they must never be separately versioned.
COPY --from=backend /app/target/release/featuredoc /usr/local/bin/featuredoc
COPY --from=backend /app/target/release/featuredoc-worker /usr/local/bin/featuredoc-worker
COPY --from=frontend /app/dist ./dist
ENV STATIC_DIR=/app/dist
EXPOSE 8080
CMD ["/usr/local/bin/featuredoc"]
