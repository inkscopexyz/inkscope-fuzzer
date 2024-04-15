# Build stage
FROM rust:1.76
RUN rustup component add rust-src && cargo install --force --locked --version 3.2.0 cargo-contract
WORKDIR /inkscope-fuzzer
COPY Cargo.toml .
COPY src/ ./src/
COPY test-contracts/ ./test-contracts/
RUN cd test-contracts && python3 build.py && cd ..
RUN cargo build
ENTRYPOINT ["cargo", "test"]
