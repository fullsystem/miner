# ── Stage 1: compila o cpuminer (pooler) do fonte ──────────────
# Compilar do fonte (em vez de commitar um binário) garante auditabilidade
# e suporte multi-arch: buildx compila nativo em amd64 e arm64.
FROM debian:bookworm-slim AS cpuminer-build
RUN apt-get update && apt-get install -y --no-install-recommends \
    git build-essential autoconf automake libtool pkg-config libcurl4-openssl-dev ca-certificates \
    && rm -rf /var/lib/apt/lists/*
RUN git clone --depth 1 --branch v2.5.1 https://github.com/pooler/cpuminer.git /src
WORKDIR /src
RUN ./autogen.sh && ./configure CFLAGS="-O3" && make -j"$(nproc)"

# ── Stage 2: compila o app Rust ─────────────────────────────────
FROM rust:1.96-slim-bookworm AS app-build
WORKDIR /app
# Camada de cache das dependências: builda um main vazio primeiro
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo 'fn main() {}' > src/main.rs \
    && cargo build --release \
    && rm -rf src
COPY src ./src
RUN touch src/main.rs && cargo build --release

# ── Stage 3: imagem final ───────────────────────────────────────
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
