#!/bin/bash
set -e

babylond testnet --v 1 --output-dir .localnet/ --keyring-backend test --chain-id sat-bbn-localnet

sed -i 's/timeout_commit = \"5s\"/timeout_commit = \"1s\"/' .localnet/node0/babylond/config/config.toml

babylond keys import-hex genesis 230FAE50A4FFB19125F89D8F321996A46F805E7BCF0CDAC5D102E7A42741887A --keyring-backend test --home .localnet/node0/babylond

babylond start --home .localnet/node0/babylond
