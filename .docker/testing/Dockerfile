# Build stage
FROM rust:1.76
RUN rustup component add rust-src && cargo install --force --locked --version 4.1.1 cargo-contract && rustup component add clippy
WORKDIR /inkscope-fuzzer
COPY Cargo.toml .
COPY src/ ./src/
COPY test-contracts/ ./test-contracts/
RUN cd test-contracts && python3 build.py && cd ..
RUN cargo build
ENTRYPOINT ["cargo", "test"]
