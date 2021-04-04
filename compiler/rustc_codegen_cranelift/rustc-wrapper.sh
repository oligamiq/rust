#!/usr/bin/env bash
set -euo pipefail

dir=$(cd "$(dirname "${BASH_SOURCE[0]}")"; pwd)

for arg in "$@"; do
    if [[ "$arg" = --target* ]]; then
        exec -- wasmtime run "$dir/target/wasm32-wasi/release/cg_clif.wasm" "$@"
    fi
done

exec -- rustc +nightly "$@"
