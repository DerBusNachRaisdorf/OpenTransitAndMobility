# syntax=docker/dockerfile:1.7-labs

# +------------------------------+
# |            BUILD             |
# +------------------------------+

FROM rust:1.80.0-slim-bookworm AS build

# View app name in Cargo.toml
ARG APP_NAME=web

WORKDIR /build

# copy scripts
COPY scripts scripts

# install build dependencies.
RUN scripts/docker/debian-build-deps.sh

# copy remaining files into the container
COPY ./ ./

RUN --mount=type=cache,target=/build/target/ \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    cargo build --release --locked && cp /build/target/release/$APP_NAME /bin/server

# +------------------------------+
# |            FINAL             |
# +------------------------------+

# run image
FROM debian:bookworm-slim AS final

WORKDIR /home

COPY scripts/docker/debian-runtime-deps.sh scripts/docker/debian-runtime-deps.sh
RUN scripts/docker/debian-runtime-deps.sh

COPY --from=build /bin/server /home
COPY resources resources

ENV RUST_LOG=info

ENTRYPOINT ["./server"]
EXPOSE 8080
