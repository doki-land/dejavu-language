# Runtime image for the `dejavu` CLI (Rust dejavu-tools).
#
# Build from repo root:
#   docker build -f projects/dejavu.rs/Dockerfile -t dejavu:local projects/dejavu.rs
#
# Image (CI): ghcr.io/doki-land/dejavu

FROM rust:1-bookworm AS builder
WORKDIR /src
COPY . .
RUN cargo build --release -p dejavu-tools --bin dejavu \
  && strip target/release/dejavu

FROM debian:bookworm-slim
RUN apt-get update \
  && apt-get install -y --no-install-recommends ca-certificates \
  && rm -rf /var/lib/apt/lists/*

COPY --from=builder /src/target/release/dejavu /usr/local/bin/dejavu

ENTRYPOINT ["dejavu"]
CMD ["--help"]
