FROM rust:1.62.1-alpine AS builder

WORKDIR /build
COPY . .

RUN apk add musl-dev \
    openssl-dev

RUN cargo install --path .


FROM alpine

RUN apk add --no-cache libc6-compat

# Adds support for running in Lambda
ENV READINESS_CHECK_PATH=/ok
COPY --from=public.ecr.aws/awsguru/aws-lambda-adapter:0.3.3 /lambda-adapter /opt/extensions/lambda-adapter

COPY --from=builder /usr/local/cargo/bin/plate-api /usr/local/bin/plate-api

ARG PLATE_API_USER_AGENT

ENTRYPOINT [ "plate-api" ]
