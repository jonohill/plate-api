FROM rust:1.66.0 AS builder

WORKDIR /build
COPY . .

RUN cargo install --path .


FROM debian:stable-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Adds support for running in Lambda
ENV READINESS_CHECK_PATH=/ok
COPY --from=public.ecr.aws/awsguru/aws-lambda-adapter:0.6.0 /lambda-adapter /opt/extensions/lambda-adapter

COPY --from=builder /usr/local/cargo/bin/plate-api /usr/local/bin/plate-api

ARG PLATE_API_USER_AGENT

ENTRYPOINT [ "plate-api" ]
