# ─── Build stage ─────────────────────────────────────────────────
FROM rust:1.83-slim-bookworm AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Cache dependencies first
COPY Cargo.toml ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release && rm -rf src target/release/deps/thepond*

# Build the actual sources
COPY src ./src
COPY migrations ./migrations
ENV SQLX_OFFLINE=true
RUN cargo build --release

# ─── Runtime stage ───────────────────────────────────────────────
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

RUN useradd --system --create-home --uid 1000 thepond

WORKDIR /app
COPY --from=builder /app/target/release/thepond /usr/local/bin/thepond
COPY --from=builder /app/migrations /app/migrations

USER thepond
EXPOSE 3000

ENV BIND_ADDRESS=0.0.0.0:3000 \
    DB_PATH=/data/thepond.db \
    UPLOADS_PATH=/data/uploads \
    THEME_PATH=/themes/duck-pond/theme.ron

CMD ["thepond"]
