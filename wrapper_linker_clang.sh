#!/bin/bash

# log file
echo "wrapper_linker.sh: $@" >> /tmp/wrapper_linker.log

# new args
filtered_args=()

# check args
while [[ $# -gt 0 ]]; do
  if [[ $1 == "-flavor" ]]; then
    # skip "-flavor" and its argument
    shift 2
    continue
  fi
    #note: wasm-ld: error: unknown argument: -Wl,--max-memory=1073741824
  if [[ $1 == "-Wl,--max-memory=1073741824" ]]; then
    # skip "-Wl,--max-memory=1073741824" because it is not supported by wasm-ld
    shift 1
    continue
  fi
  # add arg to new args
  filtered_args+=("$1")
  shift
done

# get script directory
DIR=$(cd $(dirname $0); pwd)

# call wasm-ld with new args
$DIR/wasi-sdk-22.0/bin/wasm-ld -lwasi-emulated-mman "${filtered_args[@]}"

# return wasm-ld exit code
exit $?
