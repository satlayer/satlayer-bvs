ARG TARGETARCH

FROM cosmwasm/optimizer:0.16.0 AS amd64

FROM cosmwasm/optimizer-arm64:0.16.0 AS arm64

# Select the correct build stage
FROM ${TARGETARCH:-amd64} AS optimizer
RUN apk update && apk add clang && apk add binaryen
COPY . /code

FROM optimizer AS builder
RUN mkdir /dist

ARG DIRECTORY

# Build & Optimize
RUN /usr/local/bin/optimize.sh ${DIRECTORY}
RUN mv artifacts/*.wasm /dist/contract.wasm
RUN mv artifacts/checksums.txt /dist/checksums.txt

# Generate schema
WORKDIR /code/${DIRECTORY}
RUN cargo run --bin schema
RUN mv schema/raw/ /dist/schema/
RUN mv schema/*.json /dist/schema.json

FROM scratch
COPY --from=builder /dist/ ./
