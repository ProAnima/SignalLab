# syntax=docker/dockerfile:1
#
# Signal Lab server: the engine and its interface, without a window, for a
# browser. Published as ghcr.io/proanima/signallab by .github/workflows/image.yml;
# `npm run check:image` builds and tests it locally.
#
# The container listens on 0.0.0.0:1430, so it refuses to start without a token.
# Broadcast, multicast and discovery reach the physical network only with
# --network host on a Linux host. See deploy/compose.yaml and docs/delivery.md.

# Equal to rust-toolchain.toml, and Node to the Linux builder (a test checks both).
ARG RUST_VERSION=1.98.1
ARG NODE_VERSION=24.13.0
ARG DEBIAN_RELEASE=bookworm

# ---- the interface: the same `npm run build` as the desktop app ----------------
FROM node:${NODE_VERSION}-${DEBIAN_RELEASE}-slim AS interface
WORKDIR /src
COPY package.json package-lock.json ./
RUN --mount=type=cache,target=/root/.npm npm ci --no-audit --no-fund
COPY index.html tsconfig.json tsconfig.node.json vite.config.ts ./
COPY public ./public
COPY experiments ./experiments
COPY src ./src
RUN npm run build

# ---- the server: only the engine and the server; Tauri and WebKit are never built
FROM rust:${RUST_VERSION}-${DEBIAN_RELEASE} AS server
WORKDIR /src
# Dependencies first, against placeholder sources, so a change to Signal Lab's
# own code reuses this layer. The desktop crate only has to exist for Cargo to
# read the workspace; nothing of it is compiled.
COPY Cargo.toml Cargo.lock ./
COPY engine/Cargo.toml engine/
COPY server/Cargo.toml server/
COPY src-tauri/Cargo.toml src-tauri/
RUN mkdir -p engine/src server/src src-tauri/src \
 && touch engine/src/lib.rs server/src/lib.rs src-tauri/src/lib.rs \
 && echo 'fn main() {}' > server/src/main.rs
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    cargo build --release --locked -p signal-lab-server
COPY experiments ./experiments
COPY engine ./engine
COPY server ./server
# COPY keeps the sources' times, which may be older than the placeholder build.
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    touch engine/src/lib.rs server/src/lib.rs server/src/main.rs \
 && cargo build --release --locked -p signal-lab-server \
 && ./target/release/signal-lab-server --version

# ---- the image: the binary, the interface, a user without privileges ---------
FROM debian:${DEBIAN_RELEASE}-slim
# The engine's HTTPS client uses the system's OpenSSL and certificate store.
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates libssl3 \
 && rm -rf /var/lib/apt/lists/* \
 && groupadd --system --gid 10001 signallab \
 && useradd --system --uid 10001 --gid signallab --home-dir /data --no-create-home --shell /usr/sbin/nologin signallab \
 && install -d -o signallab -g signallab -m 0750 /data
COPY --from=server /src/target/release/signal-lab-server /usr/local/bin/signal-lab-server
COPY --from=interface /src/dist /usr/share/signal-lab/ui
LABEL org.opencontainers.image.title="Signal Lab" \
      org.opencontainers.image.description="OSC and network protocol simulator, served to a browser" \
      org.opencontainers.image.source="https://github.com/ProAnima/SignalLab" \
      org.opencontainers.image.licenses="MIT" \
      org.opencontainers.image.vendor="ProAnimaStudio"
ENV SIGNALLAB_LISTEN=0.0.0.0:1430 \
    SIGNALLAB_DATA_DIR=/data \
    SIGNALLAB_UI_DIR=/usr/share/signal-lab/ui
# A named volume starts out owned by the image's user; a bind mount must be
# writable by uid 10001.
VOLUME ["/data"]
EXPOSE 1430
USER 10001:10001
HEALTHCHECK --interval=30s --timeout=5s --start-period=20s --start-interval=2s --retries=3 \
    CMD ["signal-lab-server", "healthcheck"]
# PID 1 is the server itself: it stops every job on SIGTERM (docker stop).
STOPSIGNAL SIGTERM
ENTRYPOINT ["signal-lab-server"]
