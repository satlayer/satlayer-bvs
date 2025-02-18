#!/bin/bash
set -o errexit -o nounset -o pipefail

if [ $# -eq 0 ]; then
  echo "Please provide at least one package name (as in Cargo.toml's [package].name)."
  exit 1
fi

packages="$@"
echo "Packages to build: $packages"

rm -f /code/target/wasm32-unknown-unknown/release/*.wasm

mkdir -p /code/artifacts

for package_name in $packages; do
  echo "Building package: $package_name"

  cargo build --release --lib --target wasm32-unknown-unknown -p "$package_name"

  artifact_prefix="$(echo "$package_name" | tr '-' '_')"

  wasm_files=$(find /code/target/wasm32-unknown-unknown/release \
    -name "${artifact_prefix}*.wasm" -print || true)

  if [ -z "$wasm_files" ]; then
    echo "No .wasm file found for $package_name. Skipping optimization..."
    continue
  fi

  for wasm in $wasm_files; do
    echo "Optimizing $wasm ..."
    filename=$(basename "$wasm")

    wasm-opt -Os --signext-lowering "$wasm" -o "/code/artifacts/$filename"

    echo "Copied optimized wasm to: /code/artifacts/$filename"
  done
done

echo "Generating checksums in /code/artifacts ..."
(
  cd /code/artifacts
  if ls *.wasm 1>/dev/null 2>&1; then
    sha256sum -- *.wasm | tee checksums.txt
  else
    echo "No .wasm found in /code/artifacts."
  fi
)

echo "Done optimizing."
