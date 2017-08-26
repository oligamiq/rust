export RUSTC=../../no_llvm_build/bootstrap/debug/rustc
export RUSTC_BOOTSTRAP=1
export RUSTC_FORCE_UNSTABLE=1
export RUSTC_REAL=../../no_llvm_build/build/x86_64-apple-darwin/stage1/bin/rustc
export RUSTC_LIBDIR=../../no_llvm_build/build/x86_64-apple-darwin/stage1/lib
export RUST_SYSROOT=../../no_llvm_build/build/x86_64-apple-darwin/stage1/lib/rustlib/x86_64-apple-darwin/lib
export RUSTFLAGS="--sysroot ../../no_llvm_build/build/x86_64-apple-darwin/stage1/lib/rustlib/x86_64-apple-darwin/lib"
cargo-miri miri --verbose
