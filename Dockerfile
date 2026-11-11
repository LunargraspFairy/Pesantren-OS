FROM rust:1.82-bookworm AS builder
WORKDIR /app
COPY Cargo.toml ./
COPY migrations ./migrations
COPY templates ./templates
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates sqlite3 && rm -rf /var/lib/apt/lists/*
WORKDIR /opt/sipesantren
COPY --from=builder /app/target/release/sipesantren /opt/sipesantren/sipesantren
COPY static /opt/sipesantren/static
COPY .env.example /opt/sipesantren/.env
RUN mkdir -p /opt/sipesantren/data/private_photos
EXPOSE 3000
CMD ["/opt/sipesantren/sipesantren"]
