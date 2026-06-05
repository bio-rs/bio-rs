FROM rust:1-slim AS builder

ARG BIORS_VERSION=0.54.0
RUN cargo install biors --version "${BIORS_VERSION}" --locked

FROM debian:stable-slim
COPY --from=builder /usr/local/cargo/bin/biors /usr/local/bin/biors

EXPOSE 8787
ENTRYPOINT ["biors", "serve", "--host", "0.0.0.0", "--port", "8787"]
