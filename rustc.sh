#!/bin/bash

wasmtime run /home/bjorn/Documenten/rust2/target/wasm32-wasi/release/rustc_binary.wasm --dir / -- $@
