FROM golang:1.24.2-alpine3.20 AS builder

RUN apk add --no-cache git

RUN mkdir -p /server/ \
    && cd /server/ \
    && git clone https://github.com/satlayer/satlayer-bvs.git \
    && cd satlayer-bvs/ && git checkout feat-cwindexer

RUN cd /server/satlayer-bvs/modules/cosmwasm-indexer \
    && go mod download \
    && go build -v -o indexer ./cmd/indexer/main.go

FROM alpine:3.20

ARG INDEXER_USER=indexer
ARG INDEXER_USER_UID=1000
ARG INDEXER_USER_GID=1000

RUN addgroup -S ${INDEXER_USER} -g ${INDEXER_USER_GID} \
    && adduser -S ${INDEXER_USER} -u ${INDEXER_USER_UID} -G ${INDEXER_USER}

RUN apk add --no-cache bash && rm -rf /var/cache/apk/\*

ENV WORKDIR=/home/indexer

COPY --from=builder /server/satlayer-bvs/modules/cosmwasm-indexer/indexer /bin/indexer

WORKDIR ${WORKDIR}
RUN chown -R ${INDEXER_USER_UID}:${INDEXER_USER_GID} ${WORKDIR}
USER ${INDEXER_USER}
