FROM rust:1.89 AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./

COPY src ./src

RUN cargo build --release

FROM debian:bookworm-slim

WORKDIR /app

COPY --from=builder /app/target/release/api-test-server /app/api-test-server

EXPOSE 8080

CMD ["/app/api-test-server"]