# Build stage: compile the release binary
FROM rust:1.99-bookworm AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

# Runtime stage: minimal image, SQLite is statically linked in
FROM debian:bookworm-slim
WORKDIR /app
COPY --from=builder /app/target/release/rust-shortlink-api /app/rust-shortlink-api
EXPOSE 3000
CMD ["./rust-shortlink-api"]
