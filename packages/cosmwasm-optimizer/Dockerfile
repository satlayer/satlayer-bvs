ARG TARGETARCH

FROM cosmwasm/optimizer:0.16.0 AS amd64

FROM cosmwasm/optimizer-arm64:0.16.0 AS arm64

# Select the correct build stage
FROM ${TARGETARCH:-amd64} AS base
RUN mkdir /dist

COPY . /code
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/code/target \
    cargo fetch

FROM base AS optimizer
ARG DIRECTORY

RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/code/target \
    /usr/local/bin/optimize.sh ${DIRECTORY}
RUN mv artifacts/*.wasm /dist/contract.wasm
RUN mv artifacts/checksums.txt /dist/checksums.txt

FROM base AS schema
ARG DIRECTORY

WORKDIR /code/${DIRECTORY}
ENV CARGO_PROFILE_DEV_OPT_LEVEL=0
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/code/target \
    cargo run --bin schema
RUN mv schema/raw/ /dist/schema/
RUN mv schema/*.json /dist/schema.json

FROM scratch
COPY --from=optimizer /dist/ ./
COPY --from=schema /dist/ ./
