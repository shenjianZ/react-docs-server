FROM rust:1.82-bookworm AS builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/web-rust-template /usr/local/bin/web-rust-template
COPY config ./config
RUN mkdir -p /app/data

EXPOSE 3000
CMD ["web-rust-template", "--host", "0.0.0.0"]
