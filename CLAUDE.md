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
npm run e2e                # every screen end to end: desktop app (WebView2) + server (Edge) -> artifacts/e2e/
npm run e2e:linux          # the same on Linux in Docker (WebKitGTK); -- --image signallab:dev tours the image
npm run release -- X.Y.Z --dry-run   # then --push; see docs/delivery.md
cargo test --workspace     # engine, desktop shell and server tests
cargo run -p signal-lab-server   # server on 127.0.0.1:1430 serving dist/ (npm run build first)
cargo run -p signal-lab-cli -- run empty   # signallab, the command line (docs/automation.md)
python scripts/gen-icon.py && npx tauri icon src-tauri/icons/icon-1024.png  # app icon
python scripts/gen-installer-art.py   # installer sidebar/header/dialog/banner from the icon
node scripts/install-test.mjs --image signallab:dev   # deploy/install.sh end to end on this Docker
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
(`scripts/image.mjs smoke`) on both architectures, and runs the end-to-end tour
(`scripts/e2e.mjs`) of the desktop app, the server and the image.

`npm run dev` alone serves the UI at http://localhost:1420, but **every engine
call fails there** — no Tauri runtime and no server behind it. Use it only to
look at layout/CSS; anything functional needs `npm run tauri dev`, or
`npm run build` + `cargo run -p signal-lab-server` for the browser path.

Prerequisites: Node 18+, Rust stable, MSVC build tools, WebView2 (ships with
Win 10/11). All present on this machine as of the initial setup.

## Layout

```
Cargo.toml                workspace: engine, server, src-tauri, cli (one version, one lock, target/)
src/                      React UI
  lib/transport.ts        the one place that knows desktop vs browser: invoke or fetch + WebSocket
  lib/platform.ts         fullscreen, downloads, sign-out — per platform
  lib/api.ts              typed command wrappers + event channels — mirrors the Rust types
  lib/store.tsx           shared jobs + console state, app_info, connection state
  lib/i18n.tsx            language provider, detection, t()
  lib/translate.ts        {placeholder} and ICU {n, plural, …} filling, numbers per language (pure)
  lib/locales/index.ts    LOCALES: the languages; adding one = a dictionary + a line (docs/localization.md)
  lib/locales/en.ts       source of truth for every string; ru.ts is typed against it
  components/Scope.tsx    canvas oscilloscope (no chart library)
  components/OscArgs.tsx  typed OSC argument editor (OSC + Broadcast + Signals share it)
  components/Palette.tsx  Ctrl+K signal palette, mounted once in the shell
  components/TooltipLayer.tsx  the one tooltip: any `data-tip`, hover + keyboard focus (placement: lib/tooltip.ts)
  components/Splitter.tsx the pane handle (console, properties, timeline): drag or arrow keys, sizes kept
  lib/signals.ts          firing, describing and capturing signals
  lib/library.ts          library folders as "/" paths, canonical bodies, the starter set's texts (pure)
  components/SaveSignal.tsx  Save… / Save / Save as… on HTTP, OSC, MQTT; Ctrl+S
  components/SignalTree.tsx  folders that open and close, drag & drop, F2 / Delete
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
engine/tests/ping_reply.rs  an experiment run end to end over loopback (repeat_loop.rs: Repeat and Loop)
tests/e2e/tour.ts         the end-to-end tour, run inside the page: every screen, by its visible labels
scripts/e2e.mjs           its runner: builds, starts the app / server + a browser, steps, screenshots
scripts/e2e/fixtures.mjs  loopback stand-ins the tour talks to: HTTP API, UDP/TCP sinks, OSC device, MQTT broker
src-tauri/src/lib.rs      the desktop shell: one `engine` command + Tauri events
server/src/               signal-lab-server (axum)
  config.rs               options + env vars, checked once (no token => loopback only)
  auth.rs                 token, sessions, Host/Origin rules
  routes.rs               /api/invoke, /api/events, /api/files, /api/health, /login, static UI
  events.rs               engine events fanned out to every page's WebSocket
  run.rs                  POST /api/run: a run waited for, or its steps as NDJSON lines
server/tests/server.rs    the server end to end, like a browser and a script (run.rs: /api/run)
docs/api/openapi.json     the API described (served at /api/openapi.json; a test checks it)
cli/                      `signallab`, the command line for scripts and CI (docs/automation.md)
  src/run.rs              run / validate / templates, in this process or on a server (remote.rs)
  src/send.rs             send osc|udp|http|mqtt, fire a library signal — the app's own commands
  src/i18n.rs             the interface's dictionaries (build.rs embeds them) and translate.ts in Rust
  src/junit.rs            the JUnit report; src/fail.rs the exit codes 0/1/2/3
cli/tests/cli.rs          the binary as a pipeline runs it, against loopback and a server started there
action.yml                the GitHub Action: signallab from the image, a JUnit report
Dockerfile, deploy/compose.yaml, scripts/image.mjs   the server image and its smoke test
deploy/install.sh         the server on a Linux host in one command (Docker, host network, token)
src-tauri/installer/      the installers' artwork (generated, committed; tests/installer.test.mjs)
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
  `--generate-token` (on in the image) is the one way to start with nothing set
  up: it makes `<data dir>/token` (0600) once, prints it once, and keeps it —
  never a server without a token on a reachable address.
- **A run is followed, not polled.** `Service::run` / `experiment_run::start_followed`
  give a `RunHandle` (each step, then the `RunResult`) for the same run
  `experiment_start` makes — one runner, one report. The server's `/api/run`
  and the command line both use it; a client that goes away does not stop the
  run. `signallab` speaks the interface's texts (`cli/build.rs` reads
  `src/lib/locales`), exits 0 passed / 1 failed / 2 invalid / 3 could not run,
  and its own codes need `err.<code>` texts like the engine's (the scan in
  `engine/src/error.rs` covers `cli/src`).
- **Long-running work is a job.** Register it with `JobRegistry` so the console
  strip can list and stop it, and call `finish(id)` when it ends on its own.
- **Modules never call the Inspector directly** — publish a normalized `Frame`
  to `engine::inspect`. Capture is armed explicitly; while disarmed the publish
  path is a single atomic load. High-rate sources go through the rate gate and
  report what wasn't drawn as *"N not shown"*.
- **`en.ts` defines the `Dict` type.** A key missing from `ru.ts` is a compile
  error, so `npm run build` is the translation check; `tests/i18n.test.mjs`
  adds that every language asks for the same values, has every plural form its
  rules need, and that every literal key in the code exists. Console lines store
  key + params, never finished text — that's what makes live language switching
  re-render the backlog. A count is a plural (`{n, plural, one {# x} other {# xs}}`)
  and gets the number itself, not `fmtNum(n)`; numbers and sizes go through
  `fmtNum`/`fmtBytes`, which follow the language. See docs/localization.md.
- **Every clickable thing is a real `<button>`**, sidebar nav and filter chips
  included; they need focus rings and Space/Enter. Never put a click target
  inside a `<label>`. `--text-faint` carries the 9.5px labels and is tuned to
  clear WCAG AA on all three surfaces — don't darken it.
- **Every field has a name.** A `.field` label is tied to its control
  (`htmlFor` / `id` from `useFieldIds()`, unique per mounted screen) or wraps
  it; a control without a visible label, and an icon-only button, gets an
  `aria-label`. The e2e tour fails a screen that has a nameless control.
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
  A folder is a "/" path in `group`; `folders` (version 2) keeps empty ones.
  The starter set is renamed into the interface language once, when it is
  written (`seeded`, `localizeSeed`); `SEED_IDS` must equal `seed()` (a test).
- **A sender stays tied to its signal.** HTTP, OSC and MQTT keep the id of the
  signal they were saved as or opened from; *Save* overwrites that one,
  "changed" compares `canonicalBody` (sorted keys, OSC floats as f32 — what the
  engine stores), never the raw JSON.
- **The Inspector lives in the bottom panel**, a tab next to the console, so it
  is there on every screen; it mounts on first show and then only hides, like a
  screen. Anything that shows a frame (`showFrame`) opens that tab. It is not a
  sidebar screen again.

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

- **The engine never builds sentences.** Every command fails with an
  `EngineError` whose `code` is the translation key `err.<code>`, with values
  in `params`, the `node` and `field` it is about, and the OS/library text in
  `detail`. Write codes as literals (`EngineError::new("wait.timeout")`,
  `Field::new("bind")`): a test scans the engine and fails if `en.ts` has no
  `err.<code>` / `field.<key>` text, or keeps one nobody uses. The UI shows any
  failure through `describeError`/`ErrorMessage` and stores the failure, not its
  text. That holds for the tool screens too (OSC, Broadcast, MQTT, …) and for
  event payloads (`job://ended`, `osc://message`, `mqtt://state`); a job's
  label is `job.<kind>` filled from `JobInfo::params` (a test in `jobs.rs`
  checks every kind has one), `label` stays English for server logs. Network
  causes reuse `transport.*` (`transport::of_io`, `net::bind_error`). What
  the Inspector decodes (summaries, verdicts) is protocol notation, not a
  sentence.
- **Screens stay mounted.** `App.tsx` mounts a view on first visit and only
  hides it afterwards, so nothing typed or received is lost on a tab switch. A
  view must therefore not assume it is visible (a canvas measures 0 wide while
  hidden — see `Scope`) and must not grab global keys.
- **An output may have several wires.** Each wire past the first runs as a
  parallel branch with a copy of the variables (as Fork does); a Join waits for
  every incoming wire; End passes once, in `experiment_run::run` after the last
  branch, and not at all if one failed. `connect()` adds, never replaces;
  `disconnect()` and an `Anchor` name one wire with `to`. A dragged wire always
  adds (`addBranch` on empty canvas); *A* / *Add next* / ＋ splice (`addAfter`).
  The same wire twice is `doc.connection_duplicate`.
- **Retry and replies are node settings, not nodes.** `Node::retry` applies to
  actions and waits (`NodeKind::retries`); the runner repeats only execution,
  emits a `retry` step per failed attempt, and Stop ends a pause by aborting the
  branch. An OSC/UDP node with `reply` sends from the listener on `reply.bind`
  (`Listener::send_to`, port 0 allowed) and waits there in the same step; its
  variable is written on Next. Both are document version 4.
- **Repeat is a node setting; Loop is the one cycle.** `Node::repeat` applies
  to actions: the runner sends again (`experiment_run::repeated`), rendering
  templates per send, retrying each, reporting progress at most once a second;
  jitter comes from the seed. A Loop's body (`LoopShape`: reached from Body,
  leading back) may wire back to it — validation, ordering and the editor
  (`isBackEdge`, `loopBody`) see the graph without those wires, so any other
  cycle is still an error. A body runs as one branch: no fan-out, Fork, Join,
  End or nested Loop, entered only through Body. The exit condition is read
  after each iteration and is not rendered on entry. Both are document version 5.
- **UDP receive loops survive ICMP news.** Windows reports "port unreachable"
  for an earlier send as `ConnectionReset` on the next receive (a connected
  socket elsewhere as `ConnectionRefused`); `net::udp_transient` says so, and
  every receive loop (monitor, discovery, relay, listeners) carries on. A loop
  that really cannot receive ends its job with the reason, never silently.
- **Inspector frames are numbered under the ring's lock**, so the ring is in
  number order and `drain` ships each frame once; the view also drops a frame
  it already holds (snapshot and first batch overlap).
- **Waits read an `Inbox`.** A UDP `Listener` and an MQTT `Subscription`
  (`subscribe.rs`) each fill one; matching and consumption are the same.
  Subscriptions open before the first step, so their broker and topic take
  parameters only; retained replays are ignored. A matched datagram carries its
  Inspector frame number (`Datagram::frame`), reported as the step's `frame`.
- **Waits listen from the start of the run.** `listen::Listener`s are opened in
  `experiment_run::start` before the job exists (a taken port is a Run error at
  the wait), shared per bind, and dropped with the run. A wait counts datagrams
  since the latest action on its branch and consumes the one it matches.

- **Keep `cargo test --workspace` green**: it covers the OSC codec, the
  CIDR/target resolver, socket-option paths, the capture ring, the signal
  library, an experiment run end to end, and the server's security rules.
- **Keep the e2e tour green on all four targets** (Windows/Linux × desktop/
  server). A new screen or control the tour cannot find by its label is a
  step to add in `tests/e2e/tour.ts`; `npm run build` type-checks the tour
  against `en.ts`, so a renamed text breaks the build, not the run.
- **The image runs unprivileged.** uid 10001, read-only root, no capabilities;
  it writes only to `/data`. Its Rust and Node versions equal
  `rust-toolchain.toml` and the Linux builder (a test checks). Broadcast,
  multicast and discovery reach the LAN only with `--network host` on Linux.

## Responsible use

Storm, Scanner and Broadcast emit real traffic at real hosts, and sweep/
broadcast reach the whole segment. Engine guard rails cap a sweep at 1024 hosts
and a beacon at 50 000 pps — guard rails, not permission. **Never commit a
default target pointing at a host we don't own.**
