# syntax=docker/dockerfile:1
#
# Signal Lab server: the engine and its interface, without a window, for a
# browser. Published as ghcr.io/proanima/signallab by .github/workflows/image.yml;
# `npm run check:image` builds and tests it locally.
#
# The container listens on 0.0.0.0:1430, so it needs a token: without one given,
# it makes one in /data/token on the first start (shown once in the log) and
# keeps it. Broadcast, multicast and discovery reach the physical network only
# with --network host on a Linux host. See deploy/compose.yaml and docs/develop/delivery.md.
#
# The command line `signallab` (docs/automation/cli.md) is in the image too:
#   docker run --rm --network host -v "$PWD:/work" -w /work --entrypoint signallab \
#     ghcr.io/proanima/signallab run tests/smoke.json

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
# The documentation, built into dist/docs by `npm run build` (served at /docs/).
COPY docs ./docs
COPY scripts/docs.mjs scripts/lib.mjs ./scripts/
COPY CHANGELOG.md ./
RUN npm run build

# ---- the server and the command line: Tauri and WebKit are never built ----------
FROM rust:${RUST_VERSION}-${DEBIAN_RELEASE} AS server
WORKDIR /src
# Dependencies first, against placeholder sources, so a change to Signal Lab's
# own code reuses this layer. The desktop crate only has to exist for Cargo
# to read the workspace; nothing of it is compiled.
COPY Cargo.toml Cargo.lock ./
COPY engine/Cargo.toml engine/
COPY server/Cargo.toml server/
COPY cli/Cargo.toml cli/
COPY src-tauri/Cargo.toml src-tauri/
RUN mkdir -p engine/src server/src cli/src src-tauri/src \
 && touch engine/src/lib.rs server/src/lib.rs src-tauri/src/lib.rs \
 && echo 'fn main() {}' > server/src/main.rs \
 && echo 'fn main() {}' > cli/src/main.rs
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    cargo build --release --locked -p signal-lab-server -p signal-lab-cli
# What the binaries embed: the templates, the API description, and — for the
# command line's messages — the interface's dictionaries.
COPY experiments ./experiments
COPY docs/api ./docs/api
COPY src/lib/locales ./src/lib/locales
COPY engine ./engine
COPY server ./server
COPY cli ./cli
# COPY keeps the sources' times, which may be older than the placeholder build.
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    touch engine/src/lib.rs server/src/lib.rs server/src/main.rs cli/src/main.rs cli/build.rs \
 && cargo build --release --locked -p signal-lab-server -p signal-lab-cli \
 && ./target/release/signal-lab-server --version \
 && ./target/release/signallab version

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
COPY --from=server /src/target/release/signallab /usr/local/bin/signallab
COPY --from=interface /src/dist /usr/share/signal-lab/ui
LABEL org.opencontainers.image.title="Signal Lab" \
      org.opencontainers.image.description="OSC and network protocol simulator, served to a browser" \
      org.opencontainers.image.source="https://github.com/ProAnima/SignalLab" \
      org.opencontainers.image.licenses="MIT" \
      org.opencontainers.image.vendor="ProAnimaStudio"
ENV SIGNALLAB_LISTEN=0.0.0.0:1430 \
    SIGNALLAB_DATA_DIR=/data \
    SIGNALLAB_UI_DIR=/usr/share/signal-lab/ui \
    SIGNALLAB_GENERATE_TOKEN=true
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
