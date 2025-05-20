FROM golang:1.24.2-alpine3.20 AS builder

RUN apk add --no-cache git

RUN git clone https://github.com/satlayer/satlayer-bvs.git

RUN cd satlayer-bvs/modules && \
    go mod download && \
    go build -v -o indexer ./cmd/indexer

FROM alpine:3.20

ENV WORKDIR=/app

WORKDIR ${WORKDIR}

COPY --from=builder /server/cosmwasm-indexer/indexer ${WORKDIR}/

ENTRYPOINT ["/app/indexer"]
