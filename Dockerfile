# Stage 1: Builder
FROM rust:1.78-bookworm AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/
RUN cargo build --release --bin kce-server

# Stage 2: Runtime
FROM gcr.io/distroless/cc-debian12
COPY --from=builder /app/target/release/kce-server /usr/local/bin/kce-server
EXPOSE 8080
HEALTHCHECK --interval=10s --timeout=3s --start-period=5s --retries=3 \
  CMD ["/usr/local/bin/kce-server", "--healthcheck"] || exit 1
ENTRYPOINT ["/usr/local/bin/kce-server"]
