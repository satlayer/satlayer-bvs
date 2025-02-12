.PHONY: build install test

build:
	go build -o build/satlayer-cli .

install:
	go build -o build/satlayer-cli .
	cp ./build/satlayer-cli /usr/local/bin

test: export SATLAYER_CONFIG=${PWD}/tests/config.toml
test:
	go test -v tests/e2e/chain_test.go
	go test -v tests/e2e/directory_test.go
	go test -v tests/e2e/delegation_test.go
	go test -v tests/e2e/reward_test.go
	go test -v tests/e2e/strategy_test.go
	go test -v tests/e2e/strategy_base_test.go
	go test -v tests/e2e/strategy_base_tvl_test.go
	go test -v tests/e2e/strategy_factory_test.go
