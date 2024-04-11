# Build stage
FROM rust:1.76 AS builder
WORKDIR /inkscope-fuzzer
COPY config.yaml .
COPY Cargo.toml .
COPY src/ ./src/
RUN cargo build --release

# Final stage
FROM rust:1.76
COPY --from=builder /inkscope-fuzzer/config.yaml /config.yaml
COPY --from=builder /inkscope-fuzzer/target/release/inkscope-fuzzer /inkscope-fuzzer
ENTRYPOINT ["/inkscope-fuzzer"]