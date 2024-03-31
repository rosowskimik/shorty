# syntax=docker/dockerfile:1

FROM rust:slim as build

RUN cargo install cargo-leptos

WORKDIR /build

ADD --link . .

RUN cargo leptos build --release

# ===== Final =====
FROM gcr.io/distroless/cc-debian12 as final

WORKDIR /app

COPY --from=build --link /build/target/release/shorty /app/
COPY --from=build --link /build/target/site /app/site

ENV RUST_LOG="info"
ENV LEPTOS_SITE_ADDR="0.0.0.0:80"
ENV LEPTOS_SITE_ROOT="site"

ENV APP_DB="redis://localhost"
ENV APP_PUB_URL="http://localhost"

EXPOSE 80

ENTRYPOINT ["/app/shorty"]
