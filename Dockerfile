FROM cosmwasm/optimizer-arm64:0.16.0

# Update apk repositories and install clang
RUN apk update && apk add clang && apk add binaryen
