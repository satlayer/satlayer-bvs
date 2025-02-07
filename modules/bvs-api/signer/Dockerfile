# Stage 1: Build Go and Osmosis
FROM ubuntu:20.04 AS builder

ENV GO_VERSION 1.22.5
ENV GO_DOWNLOAD_URL https://golang.org/dl/go${GO_VERSION}.linux-arm64.tar.gz
ENV GO_INSTALL_DIR /usr/local/go

# Install dependencies and Go
RUN apt-get update && apt-get install -y \
    curl \
    git \
    build-essential \
    ca-certificates \
    && curl -LO ${GO_DOWNLOAD_URL} \
    && tar -C /usr/local -xzf go${GO_VERSION}.linux-arm64.tar.gz \
    && rm go${GO_VERSION}.linux-arm64.tar.gz

ENV PATH $PATH:/usr/local/go/bin
ENV GOPATH /go
ENV PATH $GOPATH/bin:$PATH

# Clone and build Osmosis
RUN git clone https://github.com/osmosis-labs/osmosis.git /osmosis \
    && cd /osmosis \
    && make build

# Stage 2: Create the final image
FROM ubuntu:20.04

# Copy the Go binary and Osmosis binary from the builder stage
COPY --from=builder /usr/local/go /usr/local/go
COPY --from=builder /go /go
COPY --from=builder /osmosis/build/osmosisd /usr/local/bin/osmosisd

# Set environment variables
ENV PATH $PATH:/usr/local/go/bin
ENV GOPATH /go
ENV PATH $GOPATH/bin:$PATH

# Create directory for Osmosis configuration
RUN mkdir -p /root/.osmosisd

# Set working directory
WORKDIR /root/.osmosisd

# Expose necessary ports
EXPOSE 26657 26656 1317