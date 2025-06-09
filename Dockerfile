FROM lukemathwalker/cargo-chef:latest-rust-latest AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release --bin bancho-service

# We do not need the Rust toolchain to run the binary!
FROM debian:bookworm-slim AS runtime
# Install openssl
RUN apt-get update && \
    apt-get install -y openssl && \
    apt-get autoremove -y && \
    apt-get clean

WORKDIR /app
COPY --from=builder /app/target/release/bancho-service /app
ENTRYPOINT ["/app/bancho-service"]