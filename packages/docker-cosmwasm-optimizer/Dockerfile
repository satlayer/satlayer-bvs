FROM cosmwasm/optimizer:0.16.0

# Update apk repositories and install clang
RUN apk update && apk add clang && apk add binaryen

# Use custom optimizer script with better monorepo support
ADD optimizer.sh /usr/local/bin/optimizer.sh
ENTRYPOINT ["/usr/local/bin/optimizer.sh"]