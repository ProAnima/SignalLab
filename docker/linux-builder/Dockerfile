# Checks and builds Signal Lab for Linux on any machine with Docker — the same
# base system and packages as the Linux job in CI (ubuntu-22.04), so a package
# built here is the package CI would build. Used by `npm run check:linux` and
# `npm run build:linux` (scripts/linux.mjs); the source is mounted, not copied.

FROM ubuntu:22.04

ENV DEBIAN_FRONTEND=noninteractive
# Tauri's Linux prerequisites (WebKitGTK 4.1, librsvg, patchelf for AppImage),
# OpenSSL for reqwest's native TLS, and the usual build tools.
RUN apt-get update \
 && apt-get install -y --no-install-recommends \
      build-essential pkg-config curl wget file ca-certificates git xz-utils \
      libwebkit2gtk-4.1-dev librsvg2-dev libxdo-dev libssl-dev patchelf \
 && rm -rf /var/lib/apt/lists/*

# Node from nodejs.org, checked against its published SHA-256.
ARG NODE_VERSION=24.13.0
RUN curl -fsSLO "https://nodejs.org/dist/v${NODE_VERSION}/node-v${NODE_VERSION}-linux-x64.tar.xz" \
 && curl -fsSL "https://nodejs.org/dist/v${NODE_VERSION}/SHASUMS256.txt" | grep " node-v${NODE_VERSION}-linux-x64.tar.xz\$" | sha256sum -c - \
 && tar -xJf "node-v${NODE_VERSION}-linux-x64.tar.xz" -C /usr/local --strip-components=1 \
 && rm "node-v${NODE_VERSION}-linux-x64.tar.xz" \
 && node --version

# The toolchain pinned in src-tauri/rust-toolchain.toml (scripts/linux.mjs
# passes it in), with its components, so nothing is downloaded per run.
ARG RUST_TOOLCHAIN=stable
ARG RUST_COMPONENTS=clippy
ENV RUSTUP_HOME=/usr/local/rustup CARGO_HOME=/usr/local/cargo PATH=/usr/local/cargo/bin:$PATH
RUN curl -fsSL https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain "${RUST_TOOLCHAIN}" \
 && rustup component add --toolchain "${RUST_TOOLCHAIN}" ${RUST_COMPONENTS} \
 && cargo --version

# The AppImage tools are AppImages themselves; containers have no FUSE.
ENV APPIMAGE_EXTRACT_AND_RUN=1

WORKDIR /work
