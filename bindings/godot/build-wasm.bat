@echo off
set GODOT4_BIN=C:\Program Files\Godot_v4.4.1\Godot_v4.4.1-stable_win64.exe
set LIBCLANG_PATH=C:\Program Files\LLVM\bin
set BINDGEN_EXTRA_CLANG_ARGS=--target=wasm32-unknown-emscripten --sysroot=C:/Users/phili/emsdk/upstream/emscripten/cache/sysroot -isystem C:/Users/phili/emsdk/upstream/emscripten/cache/sysroot/include
set EMSDK=C:\Users\phili\emsdk
set PATH=C:\Users\phili\emsdk;C:\Users\phili\emsdk\upstream\emscripten;%PATH%
cd /d d:\Philippe\Projects\bobbin\bindings\godot
C:\Users\phili\.cargo\bin\cargo.exe +nightly-2025-12-01 build -Zbuild-std --target wasm32-unknown-emscripten %*
