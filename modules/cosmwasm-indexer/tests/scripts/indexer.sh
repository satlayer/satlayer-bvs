#!/bin/bash
set -e

cd /home/indexer && mkdir -p data

# generate config.yaml
cat > ./data/config.yaml << EOF
chain:
  bech32_prefix: "bbn"
  modules: ["wasm"]
node:
  type: "remote"
  config:
    rpc:
      client_name: "babylon"
      address: "http://babylon:26657"
      max_connections: 10
    grpc:
      address: "babylon:9090"
      insecure: true
    api:
      address: "http://babylon:1317"
parser:
  workers: 1
  start_height: 1
  listen_new_blocks: true
  average_block_time: 5s
  parse_old_blocks: true
  parse_genesis: true
database:
  url: "postgresql://docker:password@postgres:5432/indexer?sslmode=disable&search_path=public"
  max_open_connections: 1
  max_idle_connections: 1
  partition_size: 100000
  partition_batch_size: 1000
  ssl_mode_enable: "false"
  ssl_root_cert: ""
  ssl_cert: ""
  ssl_key: ""
logging:
  level: "debug"
  format: "json"
wasm:
  contracts:
    "bbn14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9sw76fy2": "cw_20"
  code_ids: [1, 2]
EOF

# start indexer
indexer start --home /home/indexer/data/
