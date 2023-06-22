FROM lukemathwalker/cargo-chef:latest-rust-1.70.0 AS chef
ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

COPY . .
RUN cargo build --release --bin siege-bot

FROM debian:bullseye-slim AS runtime
WORKDIR /app
COPY --from=builder /app/target/release/siege-bot /usr/local/bin
ENTRYPOINT ["/usr/local/bin/siege-bot"]
