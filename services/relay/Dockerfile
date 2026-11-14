FROM rust:1.100-bookworm AS build
WORKDIR /src
COPY rust-toolchain.toml Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY services ./services
RUN cargo build --release -p ouroboros-relay

FROM gcr.io/distroless/cc-debian12:nonroot
COPY --from=build /src/target/release/ouroboros-relay /ouroboros-relay
USER nonroot:nonroot
ENV SQLITE_PATH=/tmp/relay.sqlite
ENV SNAPSHOT_PATH=/tmp/relay.snapshot
EXPOSE 8080
ENTRYPOINT ["/ouroboros-relay"]
