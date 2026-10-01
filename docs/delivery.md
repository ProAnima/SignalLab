# Delivery: checks, builds, releases, server mode

Status: 2026-10-01. Parent plan: [ROADMAP.md](../ROADMAP.md#delivery).

How a change gets from this machine to a user: what is checked, where it is
built, how a release is cut, and — declared here, built next — how Signal Lab
runs headless in Docker with the interface in a browser.

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
  the release page; the server mode listens on loopback unless given a token (§7).

## 2. The local loop

| Command | What it does |
| --- | --- |
| `npm run tauri dev` | The app with hot reload — the normal development loop |
| `npm run check` | Every check, in order, stopping at the first failure (see below) |
| `npm run check:linux` | The same checks on Linux, in Docker (`docker/linux-builder`, Ubuntu 22.04 like CI) |
| `npm run tauri build` | Windows installers (`.exe` NSIS, `.msi`) into `src-tauri/target/release/bundle/` |
| `npm run build:linux` | Linux packages (`.deb`, `.rpm`, `.AppImage`) and their checksums into `artifacts/linux/`, in Docker |
| `npm run release -- X.Y.Z [--dry-run \| --push]` | Cut a release (§4) |
| `node scripts/version.mjs [check \| set X.Y.Z]` | Print, check or set the version |

`npm run check` runs, in this order:

1. **versions agree** — `package.json`, `package-lock.json`, `Cargo.toml`, `Cargo.lock`, and `tauri.conf.json` pointing at `package.json`;
2. **UI unit tests** — `npm test`;
3. **UI type check and build** — `tsc` (strict) and `vite build`;
4. **engine lints** — `cargo clippy --all-targets --locked -- -D warnings`;
5. **engine tests** — `cargo test --locked`.

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
| Checks (Linux) | `ubuntu-22.04` | the same, plus WebKitGTK 4.1, librsvg, libxdo, OpenSSL, patchelf |

Linux runs on 22.04 rather than the newest image so packages built against its glibc
also run on older distributions. A newer push to a pull request cancels the older run;
runs on `main` are kept. Dependabot (`.github/dependabot.yml`) opens grouped update pull
requests — Actions weekly, npm and Cargo monthly, majors separately — which CI checks like
any change. `actionlint` validates the workflow files.

**Recommended repository settings** (made by an owner, not by these files): protect
`main` — require *Checks (Windows)* and *Checks (Linux)* to pass and branches to be up to
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

**Versions.** Semantic versioning. Pre-releases (`-rc.N`) for builds handed out for
testing. The document format version (`experiment.rs`) is independent of the app version.

## 5. Status

| Piece | State |
| --- | --- |
| `npm run check`, single version source, CHANGELOG | Done |
| CI on Windows and Linux, Dependabot | Done |
| Release workflow: draft, Windows + Linux installers, checksums | Done |
| `npm run release`, `npm run check:linux`, `npm run build:linux` | Done |
| Code signing (§6) | Declared |
| Server mode and Docker image (§7) | Declared — phases D1–D4 |

## 6. Code signing (declared)

Unsigned installers make Windows SmartScreen warn about an unknown publisher. Plan:
Azure Trusted Signing (or an OV certificate in a hardware-backed service), wired through
Tauri's `bundle.windows.signCommand` in the release job, with the credentials as GitHub
environment secrets available only to tag builds. Linux packages get a detached signature
of `SHA256SUMS.txt` (minisign or GPG) and the public key in the README. Until then every
release page states the installers are unsigned.

## 7. Server mode and Docker image (declared)

**Goal.** Run the engine headless on a Linux machine — a rack PC next to the gear, a
show-control VM, a shared lab box — and use the full interface from any browser on the
network. Same engine, same experiments, same files.

### Architecture

```
browser ──HTTP──▶ signal-lab-server ── engine (unchanged behaviour)
        ◀──WS───  /api/invoke/<command>   /api/events   static UI
```

- **The engine stops depending on Tauri.** Today about a dozen modules take a Tauri
  `AppHandle` to emit events and reach the Inspector. They will take an
  `engine::host::Host` instead — emit an event, reach the capture bus — implemented by
  the desktop app over Tauri and by the server over WebSockets. This also lets
  `cargo test` run whole experiments against loopback without a window.
- **One command table.** The commands are declared once and generate both the Tauri
  handlers and the server's router, so "adding an engine command" stays a single list and
  the two front doors cannot drift.
- **`signal-lab-server`**, a second binary of the same crate (axum): serves the built UI,
  `POST /api/invoke/<command>` with JSON arguments (an `EngineError` or text on failure,
  exactly as the desktop app sees it), `GET /api/events` as a WebSocket of `{event,
  payload}`, `GET /api/health`.
- **The UI chooses its transport** in one place (`src/lib/transport.ts`): Tauri inside the
  app, HTTP + WebSocket in a browser. Desktop-only features (fullscreen, the OS credential
  store) are reported by the server as capabilities and hidden when absent.

### Security

- Listens on `127.0.0.1:1430` by default. Any other address requires a token
  (`--token`, or `SIGNALLAB_TOKEN`); every request and the WebSocket must carry it, the
  browser receives it once through a login link and keeps it in a cookie. Without a token
  the server refuses to bind to anything but loopback.
- No CORS; the WebSocket checks `Origin`. TLS is left to a reverse proxy (documented).
- Every job start (storm, scan, broadcast, experiment run) is logged with the client
  address. The engine's guard rails are unchanged — and are guard rails, not permission.
- Secrets: there is no OS credential store in a container. The server reads them
  read-only from files (`/run/secrets/signallab/<NAME>`, the Docker secrets layout) or
  `SIGNALLAB_SECRET_<NAME>`; setting a secret from the browser is refused with a coded
  error rather than stored somewhere weaker.
- Data (`experiment.json`, `signals.json`, run reports) lives in `SIGNALLAB_DATA_DIR`
  (`/data` in the image, a volume).

### Networking — what works where

| Docker networking | Works | Does not |
| --- | --- | --- |
| `--network host` (Linux hosts) | Everything: OSC/UDP/TCP/HTTP/MQTT to the LAN, listening ports, broadcast, multicast, discovery | — |
| Bridge (default), ports published | Unicast to reachable hosts; listeners on published ports (`-p 9000:9000/udp`) | Broadcast and multicast across the bridge; replies to unpublished ports |
| Docker Desktop (Windows/macOS) | Unicast and published listeners | Host networking to the physical LAN — use the desktop app there |

### Image

Multi-stage build: the UI (Node) and the server (Rust, release) in builder stages; the
runtime is `debian:bookworm-slim` with CA certificates, a non-root user, `EXPOSE 1430`,
`VOLUME /data` and a health check. Published by the release workflow to
`ghcr.io/proanima/signallab` as `X.Y.Z`, `X.Y` and — for releases, not pre-releases —
`latest`, for `linux/amd64` (arm64 to follow), with build provenance attested.

### Phases

| Phase | Delivers | Done when |
| --- | --- | --- |
| **D1** Host abstraction | The engine independent of Tauri; desktop behaviour unchanged | `cargo test` runs the *OSC ping → reply* experiment end to end over loopback with a test host |
| **D2** Server | `signal-lab-server`: command table, invoke + events + health, token rules, data dir, file secrets | A browser on Linux sends an HTTP request and runs an experiment through it; token and loopback rules are tested |
| **D3** Browser UI | Transport selection, capabilities, desktop-only features hidden | Every screen works in Chrome and Firefox against the server; the desktop app is unchanged |
| **D4** Docker | Image, `compose.yaml` example, GHCR publishing in the release workflow, documentation | `docker run --network host ghcr.io/proanima/signallab:X.Y.Z` on a Linux host runs *OSC ping → reply* against a device on the LAN from a browser |
