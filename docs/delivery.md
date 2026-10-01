# Delivery: checks, builds, releases, server mode

Status: 2026-10-01. Parent plan: [ROADMAP.md](../ROADMAP.md#delivery).

How a change gets from this machine to a user: what is checked, where it is
built, how a release is cut, and how Signal Lab runs headless — as a server or a
Docker image — with the interface in a browser.

## 1. Principles

- **One definition of "green".** `scripts/check.mjs` is the list of checks. `npm run check`
  runs it here, CI runs the same file on Windows and Linux. A check is added there, once.
- **Releases come from tags, builds come from CI.** A release is a version tag on `main`.
  The installers users download are built by GitHub Actions from that tag, not on a
  developer machine, so every artifact traces to a commit and a public build log.
- **A person publishes.** CI produces a *draft* release with every installer and
  `SHA256SUMS.txt`; someone looks at it and presses *Publish*. Nothing reaches users
  unreviewed, and a published release is never modified by automation.
- **One version, written once.** `package.json` holds it; Tauri reads it from there
  (`tauri.conf.json` → `"../package.json"`), the UI gets it at build time, and
  `scripts/version.mjs` writes the copy in the Rust manifests and checks they agree.
- **Notes are written as work lands.** `CHANGELOG.md` → *Unreleased* is the next
  release's notes; the release turns it into the version's section.
- **Safe defaults.** Installers are unsigned until signing is set up (§6) and say so on
  the release page; the server listens on loopback unless given a token, and the image
  runs without privileges (§7).

## 2. The local loop

| Command | What it does |
| --- | --- |
| `npm run tauri dev` | The app with hot reload — the normal development loop |
| `npm run check` | Every check, in order, stopping at the first failure (see below) |
| `npm run check:linux` | The same checks on Linux, in Docker (`docker/linux-builder`, Ubuntu 22.04 like CI) |
| `npm run tauri build` | Windows installers (`.exe` NSIS, `.msi`) into `target/release/bundle/` |
| `npm run build:linux` | Linux packages (`.deb`, `.rpm`, `.AppImage`) and their checksums into `artifacts/linux/`, in Docker |
| `npm run check:image` | The server image (`Dockerfile`) built as `signallab:dev` and smoke-tested (§7) |
| `npm run check:webkit` | Tooltips in WebKitGTK — the Linux desktop webview — driven with real pointer and key events, in Docker (`docker/webkit`, Ubuntu 22.04) |
| `cargo run -p signal-lab-server` | The server on `127.0.0.1:1430` against `dist/` (after `npm run build`) |
| `npm run release -- X.Y.Z [--dry-run \| --push]` | Cut a release (§4) |
| `node scripts/version.mjs [check \| set X.Y.Z]` | Print, check or set the version |

`npm run check` runs, in this order:

1. **versions agree** — `package.json`, `package-lock.json`, `Cargo.toml`, `Cargo.lock`, and `tauri.conf.json` pointing at `package.json`;
2. **UI unit tests** — `npm test`;
3. **UI type check and build** — `tsc` (strict) and `vite build`;
4. **Rust lints** — `cargo clippy --workspace --all-targets --locked -- -D warnings`;
5. **Rust tests** — `cargo test --workspace --locked`: the engine (including an
   experiment run end to end over loopback), the desktop shell and the server (its
   token, host and origin rules, sign-in, files and the event stream).

`--locked` makes a stale lock file a failure instead of a silent rewrite. Formatting is
not enforced: the code base predates `rustfmt` and a reformat would bury history; it can
become a check after a one-time formatting commit.

The Linux builder keeps Linux's `node_modules`, Cargo build output and downloads in
Docker volumes (`signallab-linux-*`), so they never mix with this machine's and later runs
are incremental. Together with the image that is several gigabytes inside Docker's disk
image — on Windows usually on drive C: (Docker Desktop → Settings → Resources → Advanced
→ *Disk image location* moves it). `npm run clean:linux` removes the image and volumes.

## 3. Continuous integration

`.github/workflows/ci.yml` — on every push to `main`, every pull request, by hand, and as
the first stage of a release:

| Job | Runner | Steps |
| --- | --- | --- |
| Checks (Windows) | `windows-latest` | Node 24, Rust stable + clippy, cached Cargo build, `npm ci`, `node scripts/check.mjs` |
| Checks (Linux) | `ubuntu-22.04` | the same, plus WebKitGTK 4.1, librsvg, libxdo, OpenSSL, patchelf; then tooltips in WebKitGTK (`xvfb-run -a node scripts/webkit.mjs --native`) |
| Server image (x64) | `ubuntu-24.04` | the `Dockerfile` built with Buildx (layers cached in GitHub's cache), then `node scripts/image.mjs smoke` |
| Server image (arm64) | `ubuntu-24.04-arm` | the same, natively on Arm |

Linux runs on 22.04 rather than the newest image so packages built against its glibc
also run on older distributions. A newer push to a pull request cancels the older run;
runs on `main` are kept. Dependabot (`.github/dependabot.yml`) opens grouped update pull
requests — Actions weekly, npm and Cargo monthly, majors separately — which CI checks like
any change. `actionlint` validates the workflow files.

**Recommended repository settings** (made by an owner, not by these files): protect
`main` — require *Checks (Windows)*, *Checks (Linux)* and both *Server image* jobs to pass and branches to be up to
date before merging; restrict who can push `v*` tags.

## 4. Releases

```
npm run release -- 0.4.0 --dry-run     # everything is checked, nothing changes
npm run release -- 0.4.0 --push        # version, changelog, commit, tag, push
```

`scripts/release.mjs` refuses unless all of this holds: on `main`, nothing uncommitted,
not behind `origin/main`, the tag is new, the version is higher than the current one,
*Unreleased* in `CHANGELOG.md` is not empty, and `npm run check` passes. It then sets the
version everywhere, dates the changelog section, commits `Release 0.4.0`, creates the
annotated tag `v0.4.0` carrying the notes, and with `--push` pushes `main` and the tag
atomically. Without `--push` it prints the push command and how to undo.

The tag starts `.github/workflows/release.yml`:

1. **Checks** — the whole CI workflow, on the tag.
2. **Draft release** — the tag must equal `v` + the version in `package.json`; a draft
   is created (or refreshed) with the notes from `CHANGELOG.md`. A version with a
   pre-release part (`0.4.0-rc.1`) is marked pre-release. A published release is never
   touched.
3. **Build** — in parallel, Tauri builds and uploads to the draft:
   - Windows x64: `*-setup.exe` (NSIS, per-user or per-machine, English/Russian) and `*.msi`;
   - Linux x64: `.deb`, `.rpm`, `.AppImage`.
4. **Checksums** — `SHA256SUMS.txt` over every asset, uploaded with them.

Then a person reviews the draft — notes, assets, sizes, a quick install — and publishes it.
To rebuild the assets of an unpublished tag, run the workflow by hand with that tag.

Publishing starts `.github/workflows/image.yml`, which puts the server image on GHCR
(§7). Nothing goes to the registry before a person has published the release; to rebuild
the image of a published release, run that workflow by hand with its tag.

**Versions.** Semantic versioning. Pre-releases (`-rc.N`) for builds handed out for
testing. The document format version (`experiment.rs`) is independent of the app version.

## 5. Status

| Piece | State |
| --- | --- |
| `npm run check`, single version source, CHANGELOG | Done |
| CI on Windows and Linux, Dependabot | Done |
| Release workflow: draft, Windows + Linux installers, checksums | Done |
| `npm run release`, `npm run check:linux`, `npm run build:linux` | Done |
| Server mode: engine without Tauri, `signal-lab-server`, the UI in a browser (§7) | Done — D1–D3 |
| Server image on GHCR, smoke-tested in CI for x64 and arm64 (§7) | Done — D4; first image with the next release |
| Code signing (§6) | Declared |

## 6. Code signing (declared)

Unsigned installers make Windows SmartScreen warn about an unknown publisher. Plan:
Azure Trusted Signing (or an OV certificate in a hardware-backed service), wired through
Tauri's `bundle.windows.signCommand` in the release job, with the credentials as GitHub
environment secrets available only to tag builds. Linux packages get a detached signature
of `SHA256SUMS.txt` (minisign or GPG) and the public key in the README. Until then every
release page states the installers are unsigned.

## 7. Server mode and Docker image

**Goal.** Run the engine headless on a Linux machine — a rack PC next to the gear, a
show-control VM, a shared lab box — and use the full interface from any browser on the
network. Same engine, same experiments, same files.

### Architecture

```
                 ┌ src-tauri  (desktop: one Tauri command `engine`, events over Tauri)
engine (crate) ──┤
                 └ server     (signal-lab-server: HTTP + WebSocket, static UI)

browser ──HTTP──▶ POST /api/invoke/<command>  ─┐
        ◀──WS───  GET  /api/events             ├─ engine::Service ── modules
                  GET  /  (the built UI)      ─┘
```

The Cargo workspace has three crates:

- **`engine/`** (`signal-lab-engine`) — every protocol module, the capture bus, the
  experiment runner. It knows nothing about Tauri: modules emit through
  `engine::Host` (an `EventSink` plus the capture bus), and `engine::Service` is the
  **one command table** — `invoke(name, json) → json | Failure` — used by both front
  doors, so the desktop app and the server cannot drift. Engine tests run whole
  experiments over loopback with a recording host (`engine/tests/ping_reply.rs`).
- **`src-tauri/`** — the desktop shell: one Tauri command, `engine(command, args)`,
  forwarding to the `Service`, and a Tauri `EventSink`.
- **`server/`** (`signal-lab-server`) — axum: the same `Service`, events fanned out to
  every connected page over a WebSocket, and the built UI as static files.

The UI picks its transport in one place, `src/lib/transport.ts`: Tauri inside the app,
`fetch` + one shared WebSocket (reconnecting with backoff, a banner while it is down) in
a browser. Desktop-only abilities are not guessed: `app_info` reports the mode,
whether secrets can be written and where the data folder is, and `src/lib/platform.ts`
covers fullscreen and downloads for both.

### HTTP API

| Route | |
| --- | --- |
| `GET /api/health` | `{status, version, auth}` — open to all, for health checks |
| `POST /api/invoke/<command>` | JSON arguments, exactly the desktop command's; `200` with the result, `422` with the same `EngineError` (or text) the desktop app gets |
| `GET /api/events` | WebSocket of `{event, payload}`; a page that falls behind is told how many events it missed (`server://lagged`) |
| `GET /api/files?path=` | download a file the engine wrote (reports, exports) — only inside the data folder |
| `GET/POST /login`, `POST /logout` | the sign-in page (English or Russian by `Accept-Language`) |

A script uses `Authorization: Bearer <token>`; a browser exchanges the token once at
`/login` for a session cookie.

### Security

- **Loopback by default.** Without options the server listens on `127.0.0.1:1430`
  and needs no token. Any other address needs a token (`--token-file`, `SIGNALLAB_TOKEN_FILE`
  or `SIGNALLAB_TOKEN`; at least 24 characters — `signal-lab-server token` prints a
  64-character one); without it the server exits with code 2 instead of starting.
- **Sessions.** The cookie is `HttpOnly`, `SameSite=Strict`, valid 7 days, `Secure`
  with `--secure-cookie` (behind an HTTPS proxy). Sessions live in memory: a restart
  signs everyone out. A wrong token costs a one-second delay and a warning in the log.
- **Cross-site protection.** The `Host` header must be a loopback name, or — with a token
  — any name unless `--allowed-host` narrows it (DNS rebinding). State-changing requests
  and the WebSocket must come from the server's own `Origin`; commands accept only JSON,
  so a form on another site cannot run one. No CORS.
- **Headers.** A strict `Content-Security-Policy` (`default-src 'self'`), `nosniff`,
  `X-Frame-Options: DENY`, `Referrer-Policy: same-origin` (`no-referrer` would make
  browsers send `Origin: null` with the sign-in form), `no-store` on the API.
- **Audit.** Every job start (storm, scan, broadcast, experiment run, …) is logged with
  the client address; so is every sign-in. The engine's guard rails are unchanged — and
  are guard rails, not permission.
- **Secrets.** No OS credential store in a container: secrets are read, read-only, from
  `/run/secrets/signallab/<NAME>` (the Docker secrets layout, `--secrets-dir`) or
  `SIGNALLAB_SECRET_<NAME>`. Setting one from the browser is refused with
  `secret.read_only`, never stored somewhere weaker; values still never leave the engine.
- **TLS** is left to a reverse proxy (Caddy, nginx, Traefik) in front of the server; it
  must pass WebSocket upgrades and the `Host` header. Set `--secure-cookie` there.

### Options

Every option has an environment variable, for containers.

| Option | Variable | Default |
| --- | --- | --- |
| `--listen` | `SIGNALLAB_LISTEN` | `127.0.0.1:1430` (`0.0.0.0:1430` in the image) |
| `--token-file` / `--token` | `SIGNALLAB_TOKEN_FILE` / `SIGNALLAB_TOKEN` | none — loopback only |
| `--data-dir` | `SIGNALLAB_DATA_DIR` | `Documents/SignalLab` (`/data` in the image) |
| `--secrets-dir` | `SIGNALLAB_SECRETS_DIR` | `/run/secrets/signallab` |
| `--ui-dir` | `SIGNALLAB_UI_DIR` | `ui` next to the executable, else `./dist` |
| `--allowed-host` | `SIGNALLAB_ALLOWED_HOSTS` (comma-separated) | any name with a token |
| `--secure-cookie` | `SIGNALLAB_SECURE_COOKIE` | off |
| `--log` | `SIGNALLAB_LOG` | `info` |
| `--log-format` | `SIGNALLAB_LOG_FORMAT` (`text` or `json`) | `text`, coloured only on a terminal |

`signal-lab-server healthcheck` exits 0 when a server answers on `--listen` (the image's
health check). Ctrl+C or SIGTERM closes every page's connection, stops every job and exits 0.

### Networking — what works where

| Docker networking | Works | Does not |
| --- | --- | --- |
| `--network host` (Linux hosts) | Everything: OSC/UDP/TCP/HTTP/MQTT to the LAN, listening ports, broadcast, multicast, discovery | — |
| Bridge (default), ports published | Unicast to reachable hosts; listeners on published ports (`-p 9000:9000/udp`) | Broadcast and multicast across the bridge; replies to unpublished ports |
| Docker Desktop (Windows/macOS) | Unicast and published listeners | Host networking to the physical LAN — use the desktop app there |

### Image

`Dockerfile`, three stages:

1. **interface** — `node:24-bookworm-slim`, `npm ci` and `npm run build`: the same `dist/`
   as the desktop app.
2. **server** — `rust:<rust-toolchain.toml>-bookworm`, `cargo build --release --locked
   -p signal-lab-server`. Dependencies are compiled first against placeholder sources, so
   a change to Signal Lab's own code reuses that layer; the desktop crate's manifest is
   present for Cargo but nothing of Tauri or WebKit is built.
3. **runtime** — `debian:bookworm-slim` with CA certificates and OpenSSL; the binary,
   the UI in `/usr/share/signal-lab/ui`, user `signallab` (uid/gid 10001), `/data` as a
   volume, `EXPOSE 1430`, a health check, OCI labels. About 35 MB to download, 140 MB
   unpacked.

The image runs with a read-only root filesystem and no capabilities (the smoke test does
exactly that): it writes only to `/data`. A named volume starts out owned by uid 10001; a
bind-mounted folder must be writable by it (`chown 10001:10001`).

```
docker run --rm ghcr.io/proanima/signallab token > signallab_token.txt
sudo chown 10001 signallab_token.txt && sudo chmod 400 signallab_token.txt
docker run -d --name signallab --network host \
  -v signallab-data:/data \
  -v "$PWD/signallab_token.txt:/run/secrets/signallab_token:ro" \
  -e SIGNALLAB_TOKEN_FILE=/run/secrets/signallab_token \
  --read-only --cap-drop ALL --security-opt no-new-privileges \
  ghcr.io/proanima/signallab:latest
```

`deploy/compose.yaml` is the same as a Compose file with the token as a secret.

**Smoke test** (`scripts/image.mjs smoke`, also `npm run check:image`): `--version` matches
`package.json`; without a token the container exits 2; `token` prints 64 hex characters;
started hardened with the token from a file and a fresh volume, it becomes healthy; health
answers openly with the version; commands need the token and report server mode and
`/data`; `/` sends a browser to `/login`, a wrong token is refused, the right one sets an
`HttpOnly`, `SameSite=Strict` cookie that opens the UI; a saved experiment lands in `/data`
owned by uid 10001; an OSC send reaches a signed-in page as an Inspector batch over the
WebSocket; `docker stop` ends it with exit code 0; nothing panicked.

### Publishing to GHCR

`.github/workflows/image.yml` runs when a release is **published** (never on a push), or
by hand for a published tag:

1. **Build**, natively on `ubuntu-24.04` (amd64) and `ubuntu-24.04-arm` (arm64): the tag
   must match the version and be a published release; build, smoke test, then push the
   same build by digest with an SBOM and BuildKit provenance (`mode=max`). Either
   architecture failing stops both.
2. **Tag and attest**: one multi-architecture index, tagged by `scripts/image.mjs tags` —
   `X.Y.Z` always; `X.Y` while it is the newest stable release of its line; `latest` while
   it is the newest stable release (so rebuilding an old release never moves a tag
   backwards). Pre-releases — by version (`-rc.1`) or marked so on the releases page —
   get only their own tag. It must contain exactly
   `linux/amd64` and `linux/arm64`; a signed GitHub attestation is pushed with it, and the
   published image is pulled and started once.

Verify an image: `gh attestation verify oci://ghcr.io/proanima/signallab:X.Y.Z -R ProAnima/SignalLab`.

**One-time, by an owner:** after the first publish, open the package (GitHub → the
organisation's *Packages* → `signallab` → *Package settings*), make it **public** if it is
not, and check it is linked to this repository (the `org.opencontainers.image.source`
label does that).

### Phases

| Phase | Delivers | State |
| --- | --- | --- |
| **D1** Host abstraction | The engine independent of Tauri; desktop behaviour unchanged | Done — `engine/tests/ping_reply.rs` runs *OSC ping → reply* end to end over loopback |
| **D2** Server | `signal-lab-server`: command table, invoke + events + health + files, token rules, data dir, file secrets | Done — `server/tests/server.rs` |
| **D3** Browser UI | Transport selection, capabilities, desktop-only features adapted | Done — checked in a browser against the server: experiment run with report download, reconnect banner, sign-in and sign-out |
| **D4** Docker | Image, `deploy/compose.yaml`, GHCR publishing, documentation | Done — the image is smoke-tested in CI on both architectures; first published with the next release |
