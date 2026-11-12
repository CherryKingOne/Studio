FROM rust:1.94-bookworm AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release --bins

FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates tzdata ffmpeg \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/novelcast-server /usr/local/bin/novelcast-server
COPY --from=builder /app/target/release/novelcast-worker /usr/local/bin/novelcast-worker

COPY assets ./assets

ENV APP_ENV=production
ENV TZ=UTC

EXPOSE 8080

CMD ["novelcast-server"]
