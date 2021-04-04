#!/usr/bin/env bash

set -euo pipefail

RUSTC_INSTALL_BINDIR=bin CFG_COMPILER_HOST_TRIPLE=wasm32-wasi CFG_RELEASE=1.53.0-nightly CFG_RELEASE_CHANNEL=nightly cargo +nightly build --release --target wasm32-wasi
RUSTC=rustc-wrapper.sh cargo check --manifest-path=../../library/test/Cargo.toml --target-dir wasi-target --release --target x86_64-unknown-linux-gnu --locked
