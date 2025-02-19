# Cosmwasm Optimizer

`Dockerfile.optimizer` creates a custom cosmwasm optimizer with better monorepo support.
`Dockerfile.builder` build on top of the `Dockerfile.optimizer` is used to build the wasm binary.
The output is directly written to the `artifacts` folder using `--output=./artifacts`.
