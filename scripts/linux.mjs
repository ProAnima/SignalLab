#!/usr/bin/env node
// Linux from any machine with Docker, on the same system as CI's Linux job.
//
//   npm run check:linux     every check of `npm run check`, on Linux
//   npm run build:linux     .deb, .rpm and .AppImage into artifacts/linux/
//   npm run clean:linux     remove the builder image and its volumes (several GB)
//
// The repository is mounted into the container. Linux's own node_modules,
// build output and Cargo downloads live in Docker volumes, so they never mix
// with this machine's and later runs are incremental.

import { mkdirSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { fail, isMain, root, run, seconds } from "./lib.mjs";

const IMAGE = "signallab-linux-builder";
const VOLUMES = {
  "signallab-linux-node-modules": "/work/node_modules",
  "signallab-linux-target": "/work/target",
  "signallab-linux-cargo-registry": "/usr/local/cargo/registry",
  "signallab-linux-npm-cache": "/root/.npm",
};

const BUNDLES = "deb,rpm,appimage";
const COMMANDS = {
  check: "npm ci --no-audit --no-fund && node scripts/check.mjs",
  build: [
    "npm ci --no-audit --no-fund",
    `npm run tauri build -- --bundles ${BUNDLES}`,
    "rm -rf artifacts/linux && mkdir -p artifacts/linux",
    "cp target/release/bundle/deb/*.deb target/release/bundle/rpm/*.rpm target/release/bundle/appimage/*.AppImage artifacts/linux/",
    "cd artifacts/linux && sha256sum * > SHA256SUMS.txt && cat SHA256SUMS.txt",
  ].join(" && "),
};

/** The channel and components pinned in rust-toolchain.toml. */
function toolchain() {
  const text = readFileSync(join(root, "rust-toolchain.toml"), "utf8");
  const channel = /^channel\s*=\s*"([^"]+)"/m.exec(text)?.[1];
  const list = /^components\s*=\s*\[([^\]]*)\]/m.exec(text)?.[1] ?? "";
  const components = [...list.matchAll(/"([^"]+)"/g)].map((found) => found[1]);
  if (!channel) fail("rust-toolchain.toml has no channel");
  return { channel, components };
}

/** Everything the builder keeps in Docker: its image and volumes. */
function clean() {
  for (const volume of Object.keys(VOLUMES)) {
    const removed = run("docker", ["volume", "rm", volume], { capture: true }).ok;
    console.log(`${removed ? "✔ removed" : "· no"} volume ${volume}`);
  }
  const image = run("docker", ["image", "rm", IMAGE], { capture: true }).ok;
  console.log(`${image ? "✔ removed" : "· no"} image ${IMAGE}`);
  // Only what this script created; other projects' images and build cache are not ours to remove.
  console.log("Docker Desktop keeps its disk image at the size it reached; shrink it from Docker Desktop (Troubleshoot) if needed.");
}

function main() {
  const mode = process.argv[2];
  if (mode !== "clean" && !COMMANDS[mode]) fail("usage: node scripts/linux.mjs check | build | clean");
  if (!run("docker", ["info"], { capture: true }).ok) fail("Docker is not running — start Docker Desktop first");
  if (mode === "clean") return clean();

  const started = Date.now();
  const rust = toolchain();
  console.log(`▶ builder image ${IMAGE} with Rust ${rust.channel} (cached after the first build)`);
  const buildArgs = ["--build-arg", `RUST_TOOLCHAIN=${rust.channel}`, "--build-arg", `RUST_COMPONENTS=${rust.components.join(" ")}`];
  if (!run("docker", ["build", "--target", "builder", "-t", IMAGE, ...buildArgs, join(root, "docker", "linux-builder")]).ok) fail("could not build the Linux builder image");

  if (mode === "build") mkdirSync(join(root, "artifacts", "linux"), { recursive: true });
  const mounts = [`${root}:/work`, ...Object.entries(VOLUMES).map(([volume, path]) => `${volume}:${path}`)].flatMap((mount) => ["-v", mount]);
  console.log(`▶ ${mode} on Linux`);
  const result = run("docker", ["run", "--rm", ...mounts, "-w", "/work", IMAGE, "bash", "-lc", COMMANDS[mode]]);
  if (!result.ok) fail(`Linux ${mode} failed after ${seconds(started)}`);
  console.log(`\n✔ Linux ${mode} passed · ${seconds(started)}${mode === "build" ? `\n  packages: ${join(root, "artifacts", "linux")}` : ""}`);
}

if (isMain(import.meta.url)) main();
