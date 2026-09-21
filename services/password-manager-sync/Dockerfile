FROM rust:1.85-bookworm AS builder

WORKDIR /src
COPY . .
RUN cargo build --release -p dragonforge-sync-server --features postgres

FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN useradd --create-home --uid 10001 dragonforge
WORKDIR /app
COPY --from=builder /src/target/release/dragonforge-sync-server /usr/local/bin/dragonforge-sync-server

USER dragonforge
EXPOSE 8787

ENTRYPOINT ["/usr/local/bin/dragonforge-sync-server"]
