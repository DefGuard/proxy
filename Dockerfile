FROM node:20-alpine as web

WORKDIR /app
COPY web/package.json .
COPY web/pnpm-lock.yaml .
COPY web/.npmrc .
RUN npm i -g pnpm
RUN pnpm install --ignore-scripts --frozen-lockfile
COPY web/ .
RUN pnpm run generate-translation-types
RUN pnpm build

FROM rust:1.80 as chef

WORKDIR /build

# install & cache necessary components
RUN cargo install cargo-chef
RUN rustup component add rustfmt

FROM chef as planner
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
COPY web/src/shared/images/svg ./web/src/shared/images/svg
RUN apt-get update && apt-get -y install protobuf-compiler libprotobuf-dev
COPY Cargo.toml Cargo.lock build.rs ./
COPY src src
COPY proto proto
RUN cargo install --locked --path . --root /build

# run
FROM debian:bookworm-slim as runtime
RUN apt-get update -y && \
    apt-get install --no-install-recommends -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /build/bin/defguard-proxy .
ENTRYPOINT ["./defguard-proxy"]
