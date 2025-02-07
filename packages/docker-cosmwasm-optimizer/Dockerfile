FROM --platform=linux/amd64 cosmwasm/optimizer:0.16.0 AS amd64
RUN apk update && apk add clang && apk add binaryen

# Use custom optimizer script with better monorepo support
ADD optimizer.sh /usr/local/bin/optimizer.sh
ENTRYPOINT ["/usr/local/bin/optimizer.sh"]

FROM --platform=linux/arm64 cosmwasm/optimizer-arm64:0.16.0 AS arm64

RUN apk update && apk add clang && apk add binaryen

# Use custom optimizer script with better monorepo support
ADD optimizer.sh /usr/local/bin/optimizer.sh
ENTRYPOINT ["/usr/local/bin/optimizer.sh"]