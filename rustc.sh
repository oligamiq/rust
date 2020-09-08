#!/bin/bash

if echo "$@" | grep -q -- "--target"; then
wasmtime run /home/bjorn/Documenten/rust2/target/wasm32-wasi/release/rustc_binary.wasm --dir / --dir /tmp --mapdir sysroot_src::/home/bjorn/Documenten/cg_clif/build_sysroot/sysroot_src --mapdir alloc_system::/home/bjorn/Documenten/cg_clif/build_sysroot/alloc_system -- --sysroot /home/bjorn/Documenten/cg_clif/build_sysroot/sysroot $@
else
rustc $@
fi
