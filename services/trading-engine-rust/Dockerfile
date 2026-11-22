# ── Builder ──────────────────────────────────────────────────────────────
# Pin to bookworm so the binary's glibc matches the debian:bookworm-slim runtime
# (the rolling rust:1-slim tag moved to a newer Debian → GLIBC_2.38 not found).
FROM rust:1-slim-bookworm AS builder
WORKDIR /app

# System deps for building (pkg-config rarely needed with rustls, but cheap).
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config && rm -rf /var/lib/apt/lists/*

# Cache dependencies: copy manifests first.
COPY Cargo.toml ./
COPY crates/core/Cargo.toml crates/core/
COPY crates/market-data/Cargo.toml crates/market-data/
COPY crates/signals/Cargo.toml crates/signals/
COPY crates/risk/Cargo.toml crates/risk/
COPY crates/execution/Cargo.toml crates/execution/
COPY crates/engine/Cargo.toml crates/engine/

# Now the sources and the real build.
COPY crates crates
RUN cargo build --release --workspace \
 && strip target/release/engine target/release/market-data \
          target/release/signals target/release/risk target/release/execution

# ── Runtime ──────────────────────────────────────────────────────────────
FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates && rm -rf /var/lib/apt/lists/* \
 && useradd -r -u 10001 trader
WORKDIR /app

COPY --from=builder /app/target/release/engine        /usr/local/bin/engine
COPY --from=builder /app/target/release/market-data   /usr/local/bin/market-data
COPY --from=builder /app/target/release/signals       /usr/local/bin/signals
COPY --from=builder /app/target/release/risk          /usr/local/bin/risk
COPY --from=builder /app/target/release/execution     /usr/local/bin/execution

USER trader
EXPOSE 9100
# Default to the all-in-one supervisor; override `command:` to run a single
# service (e.g. ["market-data"]) for a split regional deployment.
ENTRYPOINT []
CMD ["engine"]
