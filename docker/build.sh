#!/bin/bash
set -e

BIN_DIR="bindings/godot/addons/bobbin/bin"
mkdir -p "$BIN_DIR"

# Parse arguments - look for --ci flag anywhere
CI_MODE=false
ARGS=()
for arg in "$@"; do
    if [ "$arg" = "--ci" ]; then
        CI_MODE=true
    else
        ARGS+=("$arg")
    fi
done

TARGET="${ARGS[0]:-windows}"
BUILD_TYPE="${ARGS[1]:-release}"

# Select profile based on build type and CI mode
# Fast mode (default): use dev-fast/release-fast profiles
# CI mode: use dev/release profiles (optimized for size)
if [ "$BUILD_TYPE" = "release" ]; then
    if [ "$CI_MODE" = true ]; then
        CARGO_PROFILE_FLAG="--release"
        PROFILE="release"
    else
        CARGO_PROFILE_FLAG="--profile release-fast"
        PROFILE="release-fast"
    fi
    SUFFIX=""
else
    if [ "$CI_MODE" = true ]; then
        CARGO_PROFILE_FLAG=""
        PROFILE="debug"
    else
        CARGO_PROFILE_FLAG="--profile dev-fast"
        PROFILE="dev-fast"
    fi
    SUFFIX=".debug"
fi

# Pass --ci flag to recursive calls
CI_FLAG=""
if [ "$CI_MODE" = true ]; then
    CI_FLAG="--ci"
fi

case "$TARGET" in
    windows)
        cargo +nightly build --manifest-path bindings/godot/Cargo.toml \
            --target-dir target --target x86_64-pc-windows-gnu $CARGO_PROFILE_FLAG
        cp target/x86_64-pc-windows-gnu/$PROFILE/bobbin_godot.dll "$BIN_DIR/bobbin_godot$SUFFIX.dll"
        ;;
    wasm)
        source /opt/emsdk/emsdk_env.sh
        # Tell bindgen to use Emscripten's sysroot instead of Linux system headers
        export BINDGEN_EXTRA_CLANG_ARGS="--sysroot=$EMSDK/upstream/emscripten/cache/sysroot"
        # Set rustflags for WASM target (side module for Godot)
        export CARGO_TARGET_WASM32_UNKNOWN_EMSCRIPTEN_RUSTFLAGS="-C link-args=-pthread -C target-feature=+atomics -C link-args=-sSIDE_MODULE=2 -C panic=abort -Zlink-native-libraries=no -Cllvm-args=-enable-emscripten-cxx-exceptions=0"
        # Use 'wasm' profile (size-optimized, no LTO - LTO + build-std exceeds CI memory)
        cargo +nightly build --manifest-path bindings/godot/Cargo.toml \
            --target-dir target --profile wasm -Zbuild-std=std,panic_abort --target wasm32-unknown-emscripten
        # Run wasm-opt for aggressive size optimization (more efficient than LTO for WASM)
        WASM_FILE="target/wasm32-unknown-emscripten/wasm/bobbin_godot.wasm"
        WASM_OPT="$EMSDK/upstream/bin/wasm-opt"
        echo "Before wasm-opt: $(du -h "$WASM_FILE" | cut -f1)"
        "$WASM_OPT" -Oz --enable-threads --enable-bulk-memory "$WASM_FILE" -o "$WASM_FILE.opt"
        mv "$WASM_FILE.opt" "$WASM_FILE"
        echo "After wasm-opt:  $(du -h "$WASM_FILE" | cut -f1)"
        # WASM uses fixed name (no .debug suffix) - gdext expects 'bobbin_godot.wasm' at runtime
        cp "$WASM_FILE" "$BIN_DIR/bobbin_godot.wasm"
        ;;
    linux)
        cargo +nightly build --manifest-path bindings/godot/Cargo.toml --target-dir target $CARGO_PROFILE_FLAG
        cp target/$PROFILE/libbobbin_godot.so "$BIN_DIR/libbobbin_godot$SUFFIX.so"
        ;;
    all)
        # If no build type specified, build everything (all platforms × all types)
        # If build type specified, build all platforms with that type only
        # Platforms run in parallel (different target dirs), debug/release sequential per platform
        if [ "${ARGS[1]}" = "" ]; then
            ($0 windows release $CI_FLAG && $0 windows debug $CI_FLAG) &
            PID_WIN=$!
            ($0 linux release $CI_FLAG && $0 linux debug $CI_FLAG) &
            PID_LINUX=$!
            $0 wasm $CI_FLAG &
            PID_WASM=$!

            # Wait for all and collect exit codes
            FAILED=0
            wait $PID_WIN || FAILED=1
            wait $PID_LINUX || FAILED=1
            wait $PID_WASM || FAILED=1
            [ $FAILED -eq 0 ] || exit 1
        else
            $0 windows $BUILD_TYPE $CI_FLAG &
            PID_WIN=$!
            $0 linux $BUILD_TYPE $CI_FLAG &
            PID_LINUX=$!
            $0 wasm $CI_FLAG &
            PID_WASM=$!

            FAILED=0
            wait $PID_WIN || FAILED=1
            wait $PID_LINUX || FAILED=1
            wait $PID_WASM || FAILED=1
            [ $FAILED -eq 0 ] || exit 1
        fi
        ;;
    *)
        echo "Usage: build [windows|wasm|linux|all] [debug|release] [--ci]"
        echo ""
        echo "Build types:"
        echo "  release   Release binary (default)"
        echo "  debug     Debug binary"
        echo ""
        echo "Flags:"
        echo "  --ci      Use optimized profiles (slower build, smaller binaries)"
        echo "            Without --ci, uses fast profiles for local development"
        echo ""
        echo "Examples:"
        echo "  # Local development (fast builds)"
        echo "  docker compose run --rm --build godot windows"
        echo "  docker compose run --rm --build godot windows debug"
        echo "  docker compose run --rm --build godot all"
        echo ""
        echo "  # CI builds (optimized, small binaries)"
        echo "  docker compose run --rm --build godot all --ci"
        echo "  docker compose run --rm --build godot windows release --ci"
        exit 1
        ;;
esac

if [ "$CI_MODE" = true ]; then
    echo "Done! [$TARGET $BUILD_TYPE --ci] Artifact copied to $BIN_DIR/"
else
    echo "Done! [$TARGET $BUILD_TYPE] Artifact copied to $BIN_DIR/"
fi

