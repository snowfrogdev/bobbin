@echo off
setlocal

rem Emscripten 3.1.74 as per gdext docs
set EMSDK=C:\Users\phili\emsdk
set PATH=%EMSDK%;%EMSDK%\upstream\emscripten;%PATH%

rem Required for api-custom feature
set GODOT4_BIN=C:\Program Files (x86)\Godot_v4.3-stable_win64\Godot_v4.3-stable_win64.exe

rem Required for bindgen
set LIBCLANG_PATH=C:\Program Files\LLVM\bin
set EMSDK_SYSROOT=C:/Users/phili/emsdk/upstream/emscripten/cache/sysroot
set BINDGEN_EXTRA_CLANG_ARGS=--target=wasm32-unknown-emscripten --sysroot=%EMSDK_SYSROOT% -isystem %EMSDK_SYSROOT%/include

cd /d %~dp0
echo Building WASM with panic=abort (fix for __cpp_exception error)...
cargo +nightly build -Zbuild-std=std,panic_abort --target wasm32-unknown-emscripten

if %ERRORLEVEL% EQU 0 (
    echo Copying to addon bin...
    copy /Y target\wasm32-unknown-emscripten\debug\bobbin_godot.wasm addon\bobbin\bin\bobbin_godot.wasm
    echo Done!
) else (
    echo Build failed with error %ERRORLEVEL%
)
