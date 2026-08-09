FROM rust:1-bookworm AS builder
WORKDIR /build
COPY server/Cargo.toml ./Cargo.toml
COPY server/src ./src
COPY server/assets ./assets
COPY server/migrations ./migrations
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /build/target/release/pistation-server /usr/local/bin/pistation-server
RUN mkdir -p /data
EXPOSE 8080
CMD ["pistation-server"]
