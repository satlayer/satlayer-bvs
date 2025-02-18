FROM cosmwasm/optimizer-arm64:0.16.0
RUN apk update && apk add --no-cache bash clang binaryen

ENV RUSTFLAGS="-C link-arg=-s"

COPY optimize.sh /usr/local/bin/optimize.sh
RUN chmod +x /usr/local/bin/optimize.sh

ENTRYPOINT ["/usr/local/bin/optimize.sh"]
