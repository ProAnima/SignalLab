---
title: Building and running
description: What to install, the development loop, building installers, the server and the command line from a checkout, and the checks to run before a push.
---

# Building and running

Everything below runs from the root of a checkout. The short version:

```bash
npm install          # once
npm run tauri dev    # the app, with hot reload and Rust rebuilds
npm run check        # every check CI runs — before every push
```

## Prerequisites {#prerequisites}

| What | Version | For |
| --- | --- | --- |
| Node.js and npm | 22.18 or newer (CI uses 24; the Docker builders pin 24.13.0) | the interface, the scripts, `npm test` — the tests import TypeScript directly, which needs Node's built-in type stripping |
| Rust | the toolchain pinned in `rust-toolchain.toml` (with clippy, rustfmt, rust-src) | the engine, the desktop shell, the server, the command line. Install [rustup](https://rustup.rs); it fetches the pinned toolchain the first time `cargo` runs in the checkout |
| Windows: MSVC build tools | Visual Studio's C++ build tools | compiling Rust on Windows |
| Windows: WebView2 runtime | ships with Windows 10 and 11 | the desktop app's webview |
| Linux: Tauri's libraries | WebKitGTK 4.1 and friends, see below | the desktop app on Linux |
| Docker | Docker Desktop or Docker Engine | the Linux checks and packages, the server image, the WebKitGTK checks |
| Microsoft Edge or Google Chrome | any current | `npm run e2e` drives the server's interface in one of them on Windows |
| Python 3 with Pillow and numpy | — | only to redraw the icon and the installers' artwork |

On Linux (Ubuntu 22.04, as CI and `docker/linux-builder` use), the desktop app
needs:

```bash
sudo apt-get install -y build-essential pkg-config curl wget file \
  libwebkit2gtk-4.1-dev librsvg2-dev libxdo-dev libssl-dev patchelf
```

The server and the command line need neither Tauri nor a webview: on Linux
`cargo build -p signal-lab-server -p signal-lab-cli` needs only the Rust
toolchain, `pkg-config` and OpenSSL's headers (`libssl-dev`). Anything
`--workspace` (`cargo test`, clippy, `npm run check`) builds the desktop app too,
so it needs the libraries above.

::: tip
Rust is pinned on purpose: a newer clippy can bring new lints, and CI treats
warnings as errors. Bump `rust-toolchain.toml` on purpose, and run
`npm run check` and `npm run check:linux` before pushing the bump.
:::

## The development loop {#dev-loop}

```bash
npm install
npm run tauri dev
```

`npm run tauri dev` starts Vite on `http://127.0.0.1:1420` (`npm run dev`, the
port is fixed), opens the native window on it, reloads the interface on every
change and rebuilds and restarts the Rust side when it changes. This is the
normal loop.

`npm run dev` alone serves the interface at `http://127.0.0.1:1420` without the
engine: there is no Tauri runtime and no server behind it, so **every engine call
fails**. Use it only to look at layout and CSS.

**The browser path.** To work on what a browser sees, build the interface and
start the server from the checkout:

```bash
npm run build                    # tsc + vite build + the documentation -> dist/
cargo run -p signal-lab-server   # http://127.0.0.1:1430, this machine only, no token
```

The server serves `dist/` (or `--ui-dir`), listens on loopback and needs no
token there; any other address needs one. Its options:
`cargo run -p signal-lab-server -- --help`, and [Server](../server/index.md).

**Data.** The app and the server write under `Documents/SignalLab` in your home
folder: the experiment, the signal library, emulators, reports, exports. Set
`SIGNALLAB_DATA_DIR` to keep a development session's files somewhere else.

## The command line {#cli}

```bash
cargo run -p signal-lab-cli -- run empty          # a bundled template, run here
cargo run -p signal-lab-cli -- doctor             # what stands between it and the gear
cargo run -p signal-lab-cli -- run empty --server http://127.0.0.1:1430   # …on a server
```

The binary is `signallab` (`target/debug/signallab`). It makes an engine in its
own process, or, with `--server`, sends the run to a server. Its messages are
the interface's texts and its templates the app's: `cli/build.rs` embeds the
dictionaries and `experiments/templates/`, so it is rebuilt when one changes.
Everything it does: [Command line](../automation/cli.md).

## Building installers {#installers}

**Windows**, on Windows:

```bash
npm run tauri build
```

Tauri runs `npm run build` first, then `scripts/cli-bundle.mjs` (a release build
of `signallab`, which the installers carry), then bundles. Into
`target/release/bundle/`:

| File | What |
| --- | --- |
| `nsis/Signal Lab_<version>_x64-setup.exe` | the setup people download: for me or for everyone, in the system's language |
| `msi/Signal Lab_<version>_x64_en-US.msi` | for IT deployment |

`target/release/signal-lab.exe` is the bare app and needs no installation. The
release profile is size-optimized (`opt-level = "s"`, LTO, one codegen unit,
stripped). Tauri downloads NSIS and WiX itself the first time it bundles.
`npm run tauri build -- --bundles nsis` builds only the setup. No signing key is
needed: only the release workflow signs the updater's files.

**Linux**, from any machine with Docker:

```bash
npm run build:linux    # .deb, .rpm, .AppImage + SHA256SUMS.txt -> artifacts/linux/
npm run clean:linux    # remove the builder image and its volumes (several GB)
```

It builds in `docker/linux-builder` (Ubuntu 22.04, the system CI uses, so the
packages run on older distributions too) and keeps Linux's `node_modules`, build
output and Cargo downloads in Docker volumes, so later runs are incremental.
Installed packages need WebKitGTK 4.1.

**The server image**, from any machine with Docker:

```bash
npm run check:image                  # build signallab:dev and smoke-test it
node scripts/image.mjs build [name]  # build only
```

Installers and images that reach users are built only by GitHub Actions from a
release tag, never by hand: [Checks, builds and releases](delivery.md#releases).
The installers' switches and what they install:
[delivery.md](delivery.md#installers).

## The documentation {#docs}

```bash
npm run docs:dev     # live reload at http://localhost:5173/docs/
npm run docs:build   # dist/docs, as the app and the server serve it
node scripts/docs.mjs lint docs/guide/install.md   # front matter, ids, labels, links of some pages
```

`npm run build` builds the documentation too, so the app and the server always
carry the pages of their own version. `reference/errors.md` is generated from the
dictionaries before every build (`node scripts/docs.mjs generate`) and is not in
git. How pages are written: [Writing the documentation](writing-docs.md).

## Tests and checks {#checks}

| Command | What it covers |
| --- | --- |
| `npm test` | the interface's pure logic (placeholders and plurals, the library, graph edits, load profiles, emulators, impairments, …), every language complete, tooltips, the delivery rules (versions, workflows, the image), the installers' artwork and languages, and the documentation against the code (every language's pages and ids, every API command, event, kind of node, command of `signallab` and server option described) |
| `cargo test --workspace` | the engine — codecs, the capture ring, experiments run end to end over loopback, emulators, the relay, load, authentication, WebSocket —, the desktop shell, the server's security rules and `/api/run`, and the command line as a pipeline runs it, every kind of node and `signallab mcp` included |
| `npm run check` | everything CI checks, in order, stopping at the first failure: the versions agree, `npm test`, `npm run build` (`tsc` in strict mode, Vite, the documentation), `cargo clippy --workspace --all-targets --locked -- -D warnings`, `cargo test --workspace --locked`. `node scripts/check.mjs --list` prints the steps |
| `npm run check:linux` | the same on Linux, in Docker |
| `npm run e2e` | every screen of the real app, end to end, against loopback fixtures: the desktop app (WebView2) and the server (Edge or Chrome). Screenshots and a report in `artifacts/e2e/`. `-- --target server` or `desktop`, `--steps a,b`, `--no-build` |
| `npm run e2e:linux` | the same on Linux in Docker (WebKitGTK); `-- --image signallab:dev` tours the image instead of a build |
| `npm run check:image` | the server image built and smoke-tested |
| `npm run check:webkit` | tooltips in WebKitGTK, the Linux desktop's webview, with real pointer and key events, in Docker |
| `node scripts/install-test.mjs --image signallab:dev` | `deploy/install.sh` run for real against this machine's Docker: install, sign in, update, uninstall, purge |

Run `npm run check` before every push. A check is added to `scripts/check.mjs`,
never to a workflow file: CI runs that file on Windows and Linux. Lints are
errors and the lock files must not change (`--locked`); formatting is not
enforced. The whole story — CI, releases, the image, the tour:
[Checks, builds and releases](delivery.md).

## Versions and notes {#versions}

The version is written only in `package.json`. `node scripts/version.mjs` prints
it, `check` compares every copy, and `set X.Y.Z` writes it everywhere
(`package.json`, `package-lock.json`, `Cargo.toml`, `Cargo.lock`); Tauri reads
`package.json` itself. A user-visible change gets a line under *Unreleased* in
`CHANGELOG.md` as it lands — that text becomes the release notes. Releases
are cut with `npm run release` ([Releases](delivery.md#releases)).

## The icon and the installers' artwork {#icons}

```bash
python scripts/gen-icon.py && npx tauri icon src-tauri/icons/icon-1024.png
python scripts/gen-installer-art.py
```

`gen-icon.py` redraws the brand mark of `src/components/Brand.tsx` at 1024 px into
`src-tauri/icons/icon-1024.png` (keep the two in sync); `npx tauri icon` cuts the
platform set from it. `gen-installer-art.py` draws the NSIS sidebar and header
and the MSI's dialog and banner from that icon and the interface's colours into
`src-tauri/installer/`. Both outputs are committed; `tests/installer.test.mjs`
checks the artwork is there at the sizes the installers draw it.
