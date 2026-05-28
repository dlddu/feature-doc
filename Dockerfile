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
RUN mkdir src && echo "fn main() {}" > src/main.rs && \
    cargo build --release && rm -rf src target/release/featuredoc*
COPY backend/src ./src
RUN touch src/main.rs && cargo build --release

# ---- stage 3: runtime ----
FROM debian:bookworm-slim AS runtime
RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates && \
    rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=backend /app/target/release/featuredoc /usr/local/bin/featuredoc
COPY --from=frontend /app/dist ./dist
ENV STATIC_DIR=/app/dist
EXPOSE 8080
CMD ["/usr/local/bin/featuredoc"]
