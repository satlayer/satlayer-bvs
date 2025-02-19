ARG TARGETARCH

FROM cosmwasm/optimizer:0.16.0 AS amd64

FROM cosmwasm/optimizer-arm64:0.16.0 AS arm64

# Select the correct build stage
FROM ${TARGETARCH:-amd64} AS optimizer
RUN apk update && apk add clang && apk add binaryen
COPY . /code

FROM optimizer AS builder
ARG CRATE
RUN /usr/local/bin/optimize.sh ./${CRATE}

FROM scratch
ARG CRATE
COPY --from=builder /code/artifacts ./