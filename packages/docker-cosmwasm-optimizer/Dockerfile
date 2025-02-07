ARG TARGETARCH

FROM cosmwasm/optimizer:0.16.0 AS amd64

FROM cosmwasm/optimizer-arm64:0.16.0 AS arm64

# Select the correct build stage
FROM ${TARGETARCH:-amd64}

# Use custom optimizer script with better monorepo support
RUN apk update && apk add clang && apk add binaryen

# Use custom optimizer script with better monorepo support
ADD optimizer.sh /usr/local/bin/optimizer.sh
ENTRYPOINT ["/usr/local/bin/optimizer.sh"]