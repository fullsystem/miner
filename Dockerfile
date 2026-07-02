# ── Stage 1: build cpuminer (pooler) from source ────────────────
# Building from source (instead of committing a binary) keeps the image
# auditable and enables multi-arch: buildx compiles natively on amd64/arm64.
FROM debian:bookworm-slim AS cpuminer-build
RUN apt-get update && apt-get install -y --no-install-recommends \
    git build-essential autoconf automake libtool pkg-config libcurl4-openssl-dev ca-certificates \
    && rm -rf /var/lib/apt/lists/*
RUN git clone --depth 1 --branch v2.5.1 https://github.com/pooler/cpuminer.git /src
WORKDIR /src
RUN ./autogen.sh && ./configure CFLAGS="-O3" && make -j"$(nproc)"

# ── Stage 2: build the Rust app ─────────────────────────────────
FROM rust:1.96-slim-bookworm AS app-build
WORKDIR /app
# Dependency cache layer: build an empty main first
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo 'fn main() {}' > src/main.rs \
    && cargo build --release \
    && rm -rf src
COPY src ./src
COPY assets ./assets
RUN touch src/main.rs && cargo build --release

# ── Stage 3: final image ────────────────────────────────────────
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends \
    libcurl4 ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --no-create-home --shell /usr/sbin/nologin miner

COPY --from=cpuminer-build /src/minerd /usr/local/bin/minerd
COPY --from=app-build /app/target/release/miner /usr/local/bin/miner

USER miner
ENV PORT=3500
EXPOSE 3500

CMD ["miner"]
