# WebKitGTK 4.1 from Ubuntu 22.04 — the webview the Linux desktop app runs on,
# the same system as CI's Linux job — with its WebDriver, a virtual display and
# Node. Used by `npm run check:webkit` (scripts/webkit.mjs); the repository is
# mounted, not copied.
FROM ubuntu:22.04

ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update \
 && apt-get install -y --no-install-recommends \
      libwebkit2gtk-4.1-0 webkit2gtk-driver xvfb xauth fonts-dejavu-core curl ca-certificates xz-utils \
 && rm -rf /var/lib/apt/lists/*

# Node from nodejs.org, checked against its published SHA-256 (as in docker/linux-builder).
ARG NODE_VERSION=24.13.0
RUN curl -fsSLO --retry 5 --retry-all-errors "https://nodejs.org/dist/v${NODE_VERSION}/node-v${NODE_VERSION}-linux-x64.tar.xz" \
 && curl -fsSL --retry 5 --retry-all-errors "https://nodejs.org/dist/v${NODE_VERSION}/SHASUMS256.txt" | grep " node-v${NODE_VERSION}-linux-x64.tar.xz\$" | sha256sum -c - \
 && tar -xJf "node-v${NODE_VERSION}-linux-x64.tar.xz" -C /usr/local --strip-components=1 \
 && rm "node-v${NODE_VERSION}-linux-x64.tar.xz" \
 && node --version

WORKDIR /work
