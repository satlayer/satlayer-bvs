FROM ghcr.io/satlayer/cosmwasm-optimizer:local AS builder
ARG CRATE

COPY . /code
RUN /usr/local/bin/optimizer.sh ./${CRATE}

FROM scratch
ARG CRATE

COPY --from=builder /code/${CRATE}/artifacts ./