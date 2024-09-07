#!/bin/bash

# スクリプト名: wrapper_linker.sh

# 表示
echo "wrapper_linker.sh: $@" >> /tmp/wrapper_linker.log

# 新しい引数リストを作成しますわ
filtered_args=()

# 引数を一つずつ確認しますの
while [[ $# -gt 0 ]]; do
  if [[ $1 == "-flavor" ]]; then
    # "-flavor"が見つかったら、次の引数も飛ばしますわ
    shift 2
    continue
  fi
  # 新しい引数リストに追加しますわ
  filtered_args+=("$1")
  shift
done

# 引数をそのままlinkerに渡しますわ
/home/oligami_dev/rust/wasi-sdk-22.0/bin/wasm32-wasip1-threads-clang++ -lwasi-emulated-mman "${filtered_args[@]}"

# 終了コードをそのまま返しますの
exit $?
