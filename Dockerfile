#syntax=docker/dockerfile:1

FROM rust:1-bullseye AS build

ARG TARGETARCH
ENV TARGETARCH=${TARGETARCH}
ENV SQLX_OFFLINE=true
ENV CARGO_HOME=/build/cargo-cache/
ARG BUILD_PROFILE=release
ENV BUILD_PROFILE=${BUILD_PROFILE}

WORKDIR /build/
COPY . .
RUN --mount=type=cache,target=/build/target/,sharing=locked,id=rust-${TARGETARCH}-${BUILD_PROFILE} \
    --mount=type=cache,target=/build/cargo-cache/,sharing=locked,id=cargo-${TARGETARCH} \
    cargo install --root=/out/ --locked --bin=locksmith-cli --path=/build/crates/locksmith-cli --profile=${BUILD_PROFILE}

FROM debian:bullseye
COPY --from=build /out/bin/locksmith-cli /locksmith-cli

ENTRYPOINT [ "/locksmith-cli" ]