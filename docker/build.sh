#!/bin/bash
set -e

TARGET="${1:-windows}"
BUILD_TYPE="${2:-release}"
BIN_DIR="bindings/godot/addons/bobbin/bin"

# Set cargo release flag and output suffix
if [ "$BUILD_TYPE" = "release" ]; then
    RELEASE_FLAG="--release"
    PROFILE="release"
    SUFFIX=""
else
    RELEASE_FLAG=""
    PROFILE="debug"
    SUFFIX=".debug"
fi

case "$TARGET" in
    windows)
        cargo +nightly build --manifest-path bindings/godot/Cargo.toml \
            --target-dir target --target x86_64-pc-windows-gnu $RELEASE_FLAG
        cp target/x86_64-pc-windows-gnu/$PROFILE/bobbin_godot.dll "$BIN_DIR/bobbin_godot$SUFFIX.dll"
        ;;
    wasm)
        source /opt/emsdk/emsdk_env.sh
        # Tell bindgen to use Emscripten's sysroot instead of Linux system headers
        export BINDGEN_EXTRA_CLANG_ARGS="--sysroot=$EMSDK/upstream/emscripten/cache/sysroot"
        # Set rustflags for WASM target (side module for Godot)
        export CARGO_TARGET_WASM32_UNKNOWN_EMSCRIPTEN_RUSTFLAGS="-C link-args=-pthread -C target-feature=+atomics -C link-args=-sSIDE_MODULE=2 -C panic=abort -Zlink-native-libraries=no -Cllvm-args=-enable-emscripten-cxx-exceptions=0"
        # WASM always builds debug (nightly required for build-std)
        cargo +nightly build --manifest-path bindings/godot/Cargo.toml \
            --target-dir target -Zbuild-std=std,panic_abort --target wasm32-unknown-emscripten
        # WASM uses fixed name (no .debug suffix) - gdext expects 'bobbin_godot.wasm' at runtime
        cp target/wasm32-unknown-emscripten/debug/bobbin_godot.wasm "$BIN_DIR/bobbin_godot.wasm"
        ;;
    linux)
        cargo +nightly build --manifest-path bindings/godot/Cargo.toml --target-dir target $RELEASE_FLAG
        cp target/$PROFILE/libbobbin_godot.so "$BIN_DIR/libbobbin_godot$SUFFIX.so"
        ;;
    all)
        $0 windows $BUILD_TYPE && $0 wasm && $0 linux $BUILD_TYPE
        ;;
    *)
        echo "Usage: docker run bobbin-build [windows|wasm|linux|all] [debug|release]"
        echo ""
        echo "Examples:"
        echo "  docker run --rm -v .:/workspace bobbin-build windows release"
        echo "  docker run --rm -v .:/workspace bobbin-build windows debug"
        echo "  docker run --rm -v .:/workspace bobbin-build wasm"
        echo "  docker run --rm -v .:/workspace bobbin-build all release"
        exit 1
        ;;
esac

echo "Done! [$TARGET $BUILD_TYPE] Artifact copied to $BIN_DIR/"
