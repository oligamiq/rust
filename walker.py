#!/usr/bin/env python3

# this code find unknown import functions in .a files

import os
import subprocess

def explore_directory(directory, target):
    # make tmp directory
    try:
        os.mkdir("tmp")
    except FileExistsError:
        pass
    os.chdir("tmp")

    for dirpath, dirnames, filenames in os.walk(directory):
        # 最後が.aでおわるファイルを見つける
        for filename in filenames:
            if filename.endswith(".a"):
                file = os.path.join(dirpath, filename)
                # print(file)
                # copy file to tmp directory
                os.system(f"ar x {file}")

                # tmpの中を探索し、削除
                for tmp_dirpath, tmp_dirnames, tmp_filenames in os.walk("."):
                    for tmp_filename in tmp_filenames:
                        # print(tmp_filename)
                        if tmp_filename.endswith(".o") or tmp_filename.endswith(".obj"):
                            tmp_file = os.path.join(tmp_dirpath, tmp_filename)
                            # print(tmp_file)
                            result = subprocess.run([f"{wasi_sdk_path}/bin/nm", tmp_file], capture_output=True, text=True)
                            output = result.stdout
                            warnings = result.stderr
                            # if warnings:
                                # print(warnings)
                                # throw Exception()

                            # outputにtargetという文字列があるかどうか
                            is_found = False
                            for t in target:
                                if t in output:
                                    is_found = True
                                    break
                            if is_found:
                                print("Found!")
                                print(file)
                                print(tmp_file)
                                print(output)

                # tmpの中を削除
                os.system("rm -rf *")

now_dir = os.getcwd()
wasi_sdk_path = f"{now_dir}/wasi-sdk-22.0/"
# explore_directory(f"{now_dir}/build/wasm32-wasip1-threads/", ["dlopen", "dlsym", "dlerror", "dlclose", "mmap", "munmap"])
# explore_directory(f"{now_dir}/build/x86_64-unknown-linux-gnu/stage1-rustc/wasm32-wasip1-threads/", ["dlopen", "dlsym", "dlerror", "dlclose", "mmap", "munmap"])

# explore_directory(f"{now_dir}/build/wasm32-wasip1-threads/", ["dlopen", "dlsym", "dlerror", "dlclose"])
# explore_directory(f"{now_dir}/build/x86_64-unknown-linux-gnu/stage1-rustc/wasm32-wasip1-threads/", ["dlopen", "dlsym", "dlerror", "dlclose"])

explore_directory(f"{now_dir}/build/", ["LLVMIsMultithreaded"])
