FROM public.ecr.aws/docker/library/node:26-alpine@sha256:e88a35be04478413b7c71c455cd9865de9b9360e1f43456be5951032d7ac1a66 AS web

WORKDIR /app
COPY web/package.json web/pnpm-lock.yaml web/pnpm-workspace.yaml ./
RUN npm i -g pnpm@11
RUN pnpm install --ignore-scripts --frozen-lockfile
COPY web/ .
RUN pnpm build

FROM rust:1@sha256:44637ff22d0a6571a221bfaf137849711ad02ff4723dbb4736e297538f6a3e60 AS chef

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
COPY --from=web /app/dist ./web/dist
RUN apt-get update && apt-get -y install protobuf-compiler libprotobuf-dev
COPY Cargo.toml Cargo.lock build.rs ./
# for vergen
COPY .git .git
COPY src src
COPY proto proto
RUN cargo install --locked --path . --root /build

# run
FROM debian:13-slim@sha256:28de0877c2189802884ccd20f15ee41c203573bd87bb6b883f5f46362d24c5c2 AS runtime
RUN apt-get update -y && apt-get upgrade -y && \
    apt-get install --no-install-recommends -y ca-certificates libssl-dev lsb-release && \
    rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /build/bin/defguard-proxy .
ENTRYPOINT ["./defguard-proxy"]
