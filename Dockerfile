FROM node:24-alpine AS web

WORKDIR /app
COPY webnext/package.json webnext/pnpm-lock.yaml ./
RUN npm i -g pnpm
RUN pnpm install --ignore-scripts --frozen-lockfile
COPY webnext/ .
RUN pnpm build

FROM rust:1 AS chef

WORKDIR /build

# install & cache necessary components
RUN cargo install cargo-chef
RUN rustup component add rustfmt

FROM chef AS planner
# prepare recipe
COPY Cargo.toml Cargo.lock ./
COPY src src
COPY proto proto
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
# build deps from recipe & cache as docker layer
COPY --from=planner /build/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# build project
COPY --from=web /app/dist ./webnext/dist
RUN apt-get update && apt-get -y install protobuf-compiler libprotobuf-dev
COPY Cargo.toml Cargo.lock build.rs ./
# for vergen
COPY .git .git
COPY src src
COPY proto proto
RUN cargo install --locked --path . --root /build

# run
FROM debian:13-slim AS runtime
RUN echo "deb http://security.debian.org/ trixie-security main" \
  >> /etc/apt/sources.list
RUN apt-get update -y && apt upgrade -y && \
    apt-get install --no-install-recommends -y ca-certificates libssl-dev && \
    rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /build/bin/defguard-proxy .
ENTRYPOINT ["./defguard-proxy"]
