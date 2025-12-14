FROM rust:1.83-bookworm

# Build dependencies
RUN apt-get update && apt-get install -y \
    clang libclang-dev \
    mingw-w64 \
    unzip wget git \
    && rm -rf /var/lib/apt/lists/*

# Godot 4.3 (for api-custom binding generation)
ENV GODOT_VERSION=4.3
RUN wget -q https://github.com/godotengine/godot/releases/download/${GODOT_VERSION}-stable/Godot_v${GODOT_VERSION}-stable_linux.x86_64.zip \
    && unzip Godot_v${GODOT_VERSION}-stable_linux.x86_64.zip -d /opt \
    && mv /opt/Godot_v${GODOT_VERSION}-stable_linux.x86_64 /opt/godot \
    && chmod +x /opt/godot \
    && rm *.zip
ENV GODOT4_BIN=/opt/godot

# Emscripten SDK 3.1.74 (for WASM)
ENV EMSDK_VERSION=3.1.74
RUN git clone https://github.com/emscripten-core/emsdk.git /opt/emsdk \
    && cd /opt/emsdk \
    && ./emsdk install ${EMSDK_VERSION} \
    && ./emsdk activate ${EMSDK_VERSION}
ENV EMSDK=/opt/emsdk
ENV PATH="${EMSDK}:${EMSDK}/upstream/emscripten:${PATH}"

# Rust nightly toolchain with targets
RUN rustup toolchain install nightly \
    && rustup target add x86_64-pc-windows-gnu --toolchain nightly \
    && rustup component add rust-src --toolchain nightly

# Copy entrypoint script
COPY build.sh /usr/local/bin/build
RUN chmod +x /usr/local/bin/build

WORKDIR /workspace
ENTRYPOINT ["build"]
CMD ["windows"]
