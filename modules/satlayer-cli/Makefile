.PHONY: build install test

build:
	go build -o dist/satlayer-cli .

install:
	go build -o dist/satlayer-cli .
	cp ./dist/satlayer-cli /usr/local/bin

test: export SATLAYER_CONFIG=${PWD}/tests/config.toml
test:
	go test -v ${PWD}/tests/e2e/...
