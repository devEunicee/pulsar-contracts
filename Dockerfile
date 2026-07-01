FROM rust:1.88-bookworm AS builder
WORKDIR /app
RUN rustup target add wasm32-unknown-unknown
COPY Cargo.toml ./
COPY contracts/payment-processing-contract/Cargo.toml ./contracts/payment-processing-contract/Cargo.toml
COPY contracts/payment-processing-contract/src ./contracts/payment-processing-contract/src
RUN cargo build --release --target wasm32-unknown-unknown --manifest-path contracts/payment-processing-contract/Cargo.toml

FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/* \
    && groupadd --system app && useradd --system --gid app app
WORKDIR /app
COPY --from=builder /app/target/wasm32-unknown-unknown/release/payment_processing_contract.wasm /app/payment_processing_contract.wasm
USER app
EXPOSE 8080
CMD ["/bin/sh", "-c", "echo 'Pulsar contract WASM image ready at /app/payment_processing_contract.wasm'"]
