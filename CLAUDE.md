# Signal Lab — developer guide

OSC & network-protocol simulator. **React 19/TypeScript UI + native Rust
networking engine, run two ways:** the desktop app (Tauri 2, Windows first) —
one process, the UI calls the engine through `invoke`, telemetry comes back as
Tauri events — or `signal-lab-server`, which serves the same UI to a browser
and the same engine over HTTP + WebSocket (also as the Docker image
`ghcr.io/proanima/signallab`).

Full product documentation lives in [README.md](README.md); this file is the
short version for working on the code.

## Commands

```bash
npm install                # once
npm run tauri dev          # native window + Vite HMR + Rust rebuilds  <- normal dev loop
npm run check              # EVERYTHING CI checks — run before every push
npm run check:linux        # the same on Linux, in Docker
npm run build              # tsc (strict) + vite build  -> dist/
npm run tauri build        # Windows installers -> target/release/bundle/
npm run build:linux        # .deb/.rpm/.AppImage in Docker -> artifacts/linux/
npm run check:image        # server image built + smoke-tested in Docker
npm run check:webkit       # tooltips in WebKitGTK (the Linux webview), in Docker; CI runs it natively
npm run release -- X.Y.Z --dry-run   # then --push; see docs/delivery.md
cargo test --workspace     # engine, desktop shell and server tests
cargo run -p signal-lab-server   # server on 127.0.0.1:1430 serving dist/ (npm run build first)
python scripts/gen-icon.py && npx tauri icon src-tauri/icons/icon-1024.png  # app icon
```

**Delivery** ([docs/delivery.md](docs/delivery.md)): `scripts/check.mjs` is the one
list of checks — CI (`.github/workflows/ci.yml`, Windows + Linux) runs that file, so
add a check there, not in YAML. clippy runs with `-D warnings` and `--locked`: no
new warnings, no lock-file drift. The version is written only in `package.json`
(`node scripts/version.mjs set X.Y.Z` updates the Rust copies). Note user-visible
changes under *Unreleased* in `CHANGELOG.md` — it becomes the release notes.
Releases are tags pushed by `npm run release`; `.github/workflows/release.yml`
builds a draft release, a person publishes it, and publishing starts
`.github/workflows/image.yml` (the GHCR image, amd64 + arm64). Never upload
installers or push images by hand. CI also builds and smoke-tests the image
(`scripts/image.mjs smoke`) on both architectures.

`npm run dev` alone serves the UI at http://localhost:1420, but **every engine
call fails there** — no Tauri runtime and no server behind it. Use it only to
look at layout/CSS; anything functional needs `npm run tauri dev`, or
`npm run build` + `cargo run -p signal-lab-server` for the browser path.

Prerequisites: Node 18+, Rust stable, MSVC build tools, WebView2 (ships with
Win 10/11). All present on this machine as of the initial setup.

## Layout

```
Cargo.toml                workspace: engine, server, src-tauri (one version, one lock, target/)
src/                      React UI
  lib/transport.ts        the one place that knows desktop vs browser: invoke or fetch + WebSocket
  lib/platform.ts         fullscreen, downloads, sign-out — per platform
  lib/api.ts              typed command wrappers + event channels — mirrors the Rust types
  lib/store.tsx           shared jobs + console state, app_info, connection state
  lib/i18n.tsx            locale provider, t() with {placeholder} interpolation
  lib/locales/en.ts       source of truth for every string; ru.ts is typed against it
  components/Scope.tsx    canvas oscilloscope (no chart library)
  components/OscArgs.tsx  typed OSC argument editor (OSC + Broadcast + Signals share it)
  components/Palette.tsx  Ctrl+K signal palette, mounted once in the shell
  components/TooltipLayer.tsx  the one tooltip: any `data-tip`, hover + keyboard focus (placement: lib/tooltip.ts)
  lib/signals.ts          firing, describing and capturing signals
  lib/experimentGraph.ts  pure graph edits for the experiment editor (add, splice, layout)
  lib/experimentData.ts   template suggestions, upstream variables, JSON paths
  lib/errors.ts           describeError: one renderer for engine errors and legacy text
  components/ErrorMessage.tsx  where · what — why, technical detail folded
  views/*.tsx             one screen per module; ExperimentView is the node editor
engine/src/               signal-lab-engine — no Tauri, no window
  service.rs              THE command table: Service::invoke(name, json) for both front doors
  host.rs                 Host = EventSink + capture bus; Recorder for tests
  paths.rs                the data folder (Documents/SignalLab, or the server's)
  osc_codec.rs            hand-written OSC 1.0 encode/decode (no external OSC crate)
  osc.rs                  monitor + waveform generator
  broadcast.rs            broadcast / multicast / CIDR sweep emitter + discovery listener
  inspect.rs              the capture bus: bounded ring + batch pump to the UI
  http.rs                 request runner + concurrent burst
  netsim.rs               UDP impairment relay (latency/jitter/loss/dup/corrupt)
  storm.rs                UDP/TCP load generator
  error.rs                EngineError {code, params, node, field, detail}
  transport.rs            network failure causes (refused, timeout, dns, …)
  experiment.rs           the document model: nodes, edges, outputs, versions
  experiment_validate.rs  structural and run-time validation
  experiment_data.rs      parameters, templated fields, extraction, value checks
  experiment_actions.rs   one network action, failure classified
  experiment_steps.rs     what one step does (send, check, extract, branch, wait)
  experiment_run.rs       the runner: branches, joins, listeners, reports, Send now
  matching.rs             OSC address patterns, argument rules, UDP payloads, comparisons
  listen.rs               wait listeners: one socket per bind, bounded queue
  template.rs             the {{template}} language and seeded generators (pure)
  secrets.rs              SystemStore (OS credential store), FileStore (server), masking
  experiment_files.rs     load/save/import/export, version migration
  scan.rs                 TCP connect scanner
  mqtt_codec.rs           hand-written MQTT 3.1.1 codec (no external crate)
  mqtt.rs                 one broker connection as a job; MqttHub routes commands
  signals.rs              signal library: file + starter set (storage only)
  jobs.rs                 job registry: start / list / stop
engine/tests/ping_reply.rs  an experiment run end to end over loopback
src-tauri/src/lib.rs      the desktop shell: one `engine` command + Tauri events
server/src/               signal-lab-server (axum)
  config.rs               options + env vars, checked once (no token => loopback only)
  auth.rs                 token, sessions, Host/Origin rules
  routes.rs               /api/invoke, /api/events, /api/files, /api/health, /login, static UI
  events.rs               engine events fanned out to every page's WebSocket
server/tests/server.rs    the server end to end, like a browser and a script
Dockerfile, deploy/compose.yaml, scripts/image.mjs   the server image and its smoke test
```

## Invariants worth not breaking

- **Adding an engine command** means three edits: the function in
  `engine/src/*.rs`, one arm in `Service::invoke` (`engine/src/service.rs`,
  arguments through `args!`, camelCase, unknown fields refused), and a typed
  wrapper in `src/lib/api.ts`. If it starts a job, add it to `JOB_COMMANDS`
  (the server logs those with the client address). Neither front door changes:
  the desktop shell and the server both just forward to `Service`.
- **The engine never depends on Tauri.** Events go through `Host::emit`, the
  Inspector through `host.capture()`, files under `paths::data_dir()`. Anything
  desktop-only (fullscreen, a file dialog) lives in `src/lib/platform.ts` with
  a browser equivalent, and capabilities come from `app_info`, never guessed.
- **The server is safe by default.** No token means it binds only loopback and
  accepts only loopback `Host` names; any other address without a token is a
  startup error (exit 2). Commands take JSON only, state-changing requests and
  the WebSocket must match `Origin`, sessions are `HttpOnly` + `SameSite=Strict`.
  Keep `Referrer-Policy: same-origin` — `no-referrer` makes the sign-in form
  send `Origin: null`. `server/tests/server.rs` pins these rules.
- **Long-running work is a job.** Register it with `JobRegistry` so the console
  strip can list and stop it, and call `finish(id)` when it ends on its own.
- **Modules never call the Inspector directly** — publish a normalized `Frame`
  to `engine::inspect`. Capture is armed explicitly; while disarmed the publish
  path is a single atomic load. High-rate sources go through the rate gate and
  report what wasn't drawn as *"N not shown"*.
- **`en.ts` defines the `Dict` type.** A key missing from `ru.ts` is a compile
  error, so `npm run build` is the translation check. Console lines store
  key + params, never finished text — that's what makes live language switching
  re-render the backlog.
- **Every clickable thing is a real `<button>`**, sidebar nav and filter chips
  included; they need focus rings and Space/Enter. Never put a click target
  inside a `<label>`. `--text-faint` carries the 9.5px labels and is tuned to
  clear WCAG AA on all three surfaces — don't darken it.
- **No explanatory captions on screen.** Labels, values, states and errors only;
  any help, shortcut or "0 = …" goes in a localized `data-tip` on the control or
  its label, shown by the one `TooltipLayer` (hover + keyboard focus, top layer,
  `aria-describedby`). Never the native `title`; `tests/tooltip.test.mjs` fails
  on `title=` and on the old caption classes. Empty states are a few words.
- **One socket, one owner.** The MQTT connection is a single task that owns the
  stream; `MqttHub` maps a job id to its command channel, so publish/subscribe
  from the UI reach it without a lock on the wire. Inbound messages are batched
  to the UI every 100 ms — a `#` scan is a firehose — and the batch reports what
  it had to shed rather than dropping it silently.
- **MQTT is 3.1.1, plain TCP, clean session, QoS 0/1/2.** Show and installation
  gear routinely uses QoS 2 for subscribe, publish *and* its last-will, so none
  of that is optional; clean sessions are why there is no offline queue to
  persist. No MQTT 5, no TLS — adding either is a decision, not a detail.

- **A signal is not a second send path.** `engine/signals.rs` only stores; the
  UI fires through `osc_send` / `broadcast_send` / `http_request`, so a library
  signal is byte-identical to a hand-typed one and shows up in the Inspector
  under its real source. Adding a transport means a `SignalBody` variant plus a
  branch in `fireSignal` — never a new command.
- **The library file is someone else's document.** It lives in
  `Documents/SignalLab/signals.json`, is written whole on a debounce, and a
  parse error is reported with the path rather than silently overwritten with
  the starter set. Shipped seed targets stay on loopback; a test enforces it.

- **Templates are resolved only by the engine.** `template.rs` is the language;
  the editor's preview calls `experiment_resolve` and *Send now* calls
  `experiment_send_node` (the runner's own `experiment_steps::execute`) instead of
  reimplementing it, so a field means the same thing everywhere. Unknown names
  are errors, never empty strings. A new templated field goes in
  `experiment_data::texts_mut` — validation, preview and execution all read it.
- **Secret values never leave the engine.** They live in the OS credential
  store on a desktop, and in read-only files (`/run/secrets/signallab/<NAME>`)
  or `SIGNALLAB_SECRET_<NAME>` on a server (`engine/src/secrets.rs`); a server
  refuses to set one (`secret.read_only`). No command returns one, and while a
  run or a *Send now* uses them every string it reports is passed through
  `secrets::mask`, and Inspector frames are redacted. Tests use `MemoryStore`,
  never the real credential store.

- **The experiment engine never builds sentences.** It fails with an
  `EngineError` whose `code` is the translation key `err.<code>`, with values
  in `params`, the `node` and `field` it is about, and the OS/library text in
  `detail`. Write codes as literals (`EngineError::new("wait.timeout")`,
  `Field::new("bind")`): a test scans the engine and fails if `en.ts` has no
  `err.<code>` / `field.<key>` text, or keeps one nobody uses. The UI shows any
  failure through `describeError`/`ErrorMessage` and stores the failure, not its
  text. Other modules still return strings; converting one is local.
- **Screens stay mounted.** `App.tsx` mounts a view on first visit and only
  hides it afterwards, so nothing typed or received is lost on a tab switch. A
  view must therefore not assume it is visible (a canvas measures 0 wide while
  hidden — see `Scope`) and must not grab global keys.
- **Waits listen from the start of the run.** `listen::Listener`s are opened in
  `experiment_run::start` before the job exists (a taken port is a Run error at
  the wait), shared per bind, and dropped with the run. A wait counts datagrams
  since the latest action on its branch and consumes the one it matches.

- **Keep `cargo test --workspace` green**: it covers the OSC codec, the
  CIDR/target resolver, socket-option paths, the capture ring, the signal
  library, an experiment run end to end, and the server's security rules.
- **The image runs unprivileged.** uid 10001, read-only root, no capabilities;
  it writes only to `/data`. Its Rust and Node versions equal
  `rust-toolchain.toml` and the Linux builder (a test checks). Broadcast,
  multicast and discovery reach the LAN only with `--network host` on Linux.

## Responsible use

Storm, Scanner and Broadcast emit real traffic at real hosts, and sweep/
broadcast reach the whole segment. Engine guard rails cap a sweep at 1024 hosts
and a beacon at 50 000 pps — guard rails, not permission. **Never commit a
default target pointing at a host we don't own.**
