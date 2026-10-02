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
  lib/experimentText.ts   what the editor writes about a node: summaries, previews, glyphs (pure)
  lib/experimentWires.ts  wire geometry: paths, ports, the node nearest a point (pure)
  lib/useExperiment*.ts   the editor's hooks: Run (run, stop, timeline), NodeTests (Send now,
                          preview), Canvas (drag, pan, wires), Shortcuts, Fullscreen
  components/Experiment*.tsx  the editor's parts: Toolbar, Canvas(+Node, Tools), Properties,
                          NodeTest, Timeline, AddMenu; NodeFields, EmulatorFields, FaultFields the forms
  lib/errors.ts           describeError: one renderer for engine errors and legacy text
  components/ErrorMessage.tsx  where · what — why, technical detail folded
  views/*.tsx             one screen per module; ExperimentView composes the node editor
  styles.css              the stylesheet: styles/*.css by area, @imported in cascade order
  lib/emulators.ts        new emulators, presets, Mock this, the starter set's texts (pure)
  lib/emulatorStore.tsx   the emulator library, which ones run, what they received
  components/EmulatorEditor.tsx  an emulator's rules: the Emulators screen and the node's dialog
  lib/impairments.ts      impairment presets, which preset a profile is, its notation (pure)
  lib/httpAuth.ts         switching an auth scheme (pure); components/HttpAuthFields.tsx the fields,
                          plain on the HTTP screen, templated in the HTTP node
  lib/websocket.ts        subprotocol lists, JSON of a message (pure); views/WsView.tsx the screen,
                          components/ExperimentWsFields.tsx the four nodes' fields
  components/ImpairProfileFields.tsx  a profile's preset chips and values: the Impairment screen and the fault nodes
engine/src/               signal-lab-engine — no Tauri, no window
  service.rs              THE command table: Service::invoke(name, json) for both front doors
  host.rs                 Host = EventSink + capture bus; Recorder for tests
  paths.rs                the data folder (Documents/SignalLab, or the server's)
  osc_codec.rs            hand-written OSC 1.0 encode/decode (no external OSC crate)
  osc.rs                  monitor + waveform generator
  broadcast.rs            broadcast / multicast / CIDR sweep emitter
  discovery.rs            the discovery listener: peers, multicast joins, auto-reply
  inspect.rs              the capture bus: bounded ring + batch pump to the UI
  http.rs                 request runner + concurrent burst (closed, or at a rate; missed counted)
  latency.rs              LatencyHistogram: p50…p99 within 0.5 %, atomics, constant memory
  netsim.rs               UDP impairment relay: profiles, seeded per-leg decisions, live changes, phases
  netsim_run.rs           a run's relays (Impairment nodes), opened before the first step
  storm.rs                UDP/TCP load generator
  error.rs                EngineError {code, params, node, field, detail}
  transport.rs            network failure causes (refused, timeout, dns, …)
  experiment.rs           the document model: nodes, edges, outputs, versions
  experiment_validate.rs  structural and run-time validation: the document, the graph, loops
  experiment_fields.rs    one node's fields, its Retry and Repeat: present, in range, well formed
  experiment_data.rs      parameters, templated fields, extraction, value checks
  experiment_actions.rs   one network action, failure classified
  experiment_steps.rs     what one step does (send, check, extract, branch, wait)
  experiment_run.rs       a run started and followed (RunHandle, RunResult), Send now
  experiment_flow.rs      how a run moves: listeners armed, branches, joins, retry, repeat
  experiment_report.rs    the run report file
  matching.rs             OSC address patterns, argument rules, UDP payloads, comparisons
  listen.rs               wait listeners: one socket per bind, bounded queue
  template.rs             the {{template}} language and seeded generators (pure)
  secrets.rs              SystemStore (OS credential store), FileStore (server), masking
  experiment_files.rs     load/save/import/export, version migration
  scan.rs                 TCP connect scanner
  mqtt_codec.rs           hand-written MQTT 3.1.1 codec (no external crate)
  mqtt.rs                 one broker connection as a job; MqttHub routes commands
  http_auth.rs            Basic, Bearer, Digest (RFC 7616): challenges parsed, answers, a burst's memory
  cookies.rs              CookieJar: reqwest's cookie store, held so a jar can be listed and cleared
  ws.rs                   WebSocket client: one owner task per socket (Connection), the screen's job
                          and WsHub, a run's connections (RunSocket, an Inbox), one exchange
  mqtt_dial.rs            CONNECT/CONNACK, failures as causes, the one-shot publish
  signals.rs              signal library: file + starter set (storage only)
  jobs.rs                 job registry: start / list / stop
  emulator.rs             emulators: the document (protocols, rules, responses, faults, Outage)
  emulator_match.rs       which HTTP requests a route or wait takes: method, path pattern, conditions
  emulator_rules.rs       a document checked and compiled; which response a request gets
  emulator_state.rs       what serving shares: counters, kept exchanges, the reply Context
  emulator_http.rs        the HTTP emulator (hyper): routes, sequences, faults, outages
  emulator_net.rs         OSC/UDP responders (also a run listener's Tap) and the TCP device
  emulator_mqtt.rs        the MQTT broker: routing, QoS 0/1/2, retained, wills, device rules
  emulator_job.rs         an emulator as a job of its own; EmulatorHub answers what arrived
  emulator_run.rs         a run's emulators and HTTP listeners, opened before the first step
  emulator_files.rs       the emulator library (emulators.json) and its starter set
  firewall.rs             Windows Firewall per program: status (COM, any language), allow (UAC)
engine/tests/burst.rs     the HTTP burst against loopback: closed, paced, missed, stopped
engine/tests/websocket.rs the screen's job, a run's connect/send/wait/close, server closes, failed upgrades
engine/tests/http_auth.rs Basic/Bearer/Digest checked by the server's own hashing, a burst's one challenge, jars
engine/tests/ping_reply.rs  an experiment run end to end over loopback (repeat_loop.rs: Repeat and Loop;
                          emulators.rs: Emulator nodes, Wait for HTTP request, the flaky-API template)
tests/e2e/tour.ts         the end-to-end tour, run inside the page: every screen, by its visible labels
  dsl.ts, steps/*.ts      finding by label, clicking, waiting; the steps by screen (STEPS in tour.ts)
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
  src/matrix.rs           --matrix / --matrix-file: combinations, each a run of its own (prepare in run.rs)
  src/mcp.rs              `signallab mcp`: the Model Context Protocol on stdio, for an LLM
  src/mcp_tools.rs        its tools' schemas and the experiment tools; mcp_send.rs, mcp_library.rs the rest
  src/emulate.rs          `emulate` / `emulators`: emulators from files or the library, followed until Ctrl+C
  src/catalog.rs          every kind of node with fields, outputs, an example (describe_nodes, `nodes`)
  src/doctor.rs           `doctor` and `firewall allow`
cli/tests/cli.rs          the binary as a pipeline runs it, against loopback and a server started there
cli/tests/nodes.rs        every kind of node in one experiment, here and on a server, against loopback gear (common/)
cli/tests/mcp.rs          `signallab mcp` driven like an LLM client: every tool, progress, cancel, --server
cli/tests/emulate.rs      `signallab emulate` in the background, here and on a server
action.yml                the GitHub Action: signallab from the image, a JUnit report
Dockerfile, deploy/compose.yaml, scripts/image.mjs   the server image and its smoke test
deploy/install.sh         the server on a Linux host in one command (Docker, host network, token)
src-tauri/installer/      the installers' artwork (generated, committed; tests/installer.test.mjs),
                          hooks.nsh + path.ps1 (setup: signallab.exe, PATH, firewall), cli.wxs (MSI)
scripts/cli-bundle.mjs    builds signallab for the installers (beforeBundleCommand)
src/components/FirewallBanner.tsx  the desktop app's firewall notice, with Allow
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
- **Every kind of node is in two lists.** `cli/src/catalog.rs` (what an LLM
  and `signallab nodes` read) and `cli/tests/nodes.rs` (one experiment with
  all of them, run here and on a server); both fail on a `NodeKind` they lack.
- **`signallab mcp` owns stdout.** One JSON-RPC message per line and nothing
  else; people's text goes to stderr. Tools only read or are marked as reaching
  the outside world; every action is an engine command, as in the app.
- **The firewall changes only when a person says so.** The desktop app's notice
  and `signallab firewall allow` run the system's administrator prompt
  (`firewall::allow`); the setup *for everyone* adds one allow rule per program
  (private, domain; `/NOFIREWALL` skips it) and its uninstaller removes them;
  a server never changes its host's firewall (`firewall.server`). Reading the
  rules goes through the firewall's COM interface, never netsh's localized text.
- **An emulator is one document everywhere.** The Emulators screen, the
  *Emulator* node (`NodeKind::Emulator { emulator }`), `signallab emulate`, MCP
  and the API all hand an `emulator::Emulator` to `emulator_rules::compile`
  (`emulator::check` outside the engine), which checks it (problems carry
  `rule`/`retained`/`response` params, shown by `describeError` as *Rule n ·
  Response n*) and the protocol modules serve it. Its matching patterns take
  parameters only (`emulator.params_only`); replies are templates
  read with `request` and parameters, never secrets. A run opens its emulators
  in `emulator_run::arm_run` before the first step, like waits: an HTTP emulator's
  server is also what *Wait for HTTP request* reads (`HttpListener` inbox; a bind
  with only waits answers 204), and an OSC/UDP emulator answers through the run's
  `Listener` on its port (`listen::Tap`), so waits there see the same datagrams.
  Two emulators of one transport cannot share a port (`check_run_binds`). An
  `Outage` is a schedule from the start, read by `Context::down_for`; what met
  it is counted as `down`, never as *no rule*. The MQTT broker keeps the
  client's rules — 3.1.1, plain TCP, clean sessions, QoS 0/1/2 — and an MQTT
  wait subscribes to it like to any broker. Starter emulators stay on
  loopback; `EMULATOR_SEED_IDS` must equal `seed()` (a test).
- **Faults are nodes, opened like waits, closed with the run.** An *Impairment*
  node's relay is opened in `netsim_run::arm_run` before the first step (its
  listen and target take parameters only) and closed when the run ends in any
  way; *Change impairment* calls `Relay::set`, which closes the phase so far
  (`Phase`, counted in the report) without rebinding; *Emulator down/up* sets
  `Emulation::force`, which `Context::down` reads before any outage schedule.
  Both name another node by id (`impair.relay_unknown`, `emulator.node_unknown`).
  Every relay decision draws from the run's seed per direction (`Leg`), so the
  same seed and traffic drop the same packets — keep it that way: no
  `thread_rng` in the relay.
- **Credentials go only into the request.** `HttpRequest::auth` (Basic,
  Bearer, Digest) becomes an `Authorization` header in `http::builder` as the
  request is sent; frames, steps and reports carry the response, never that
  header, and a node's credentials are templated fields (`{{secret.NAME}}`,
  masked). Digest answers a 401's challenge and sends again
  (`http::execute`); a burst shares one `DigestMemory`, so one challenge serves
  all its requests. The RFC 2617/7616 examples are unit tests, and the e2e
  fixture checks answers with hashing of its own.
- **Cookies are a jar a person can see.** `cookies::CookieJar` is reqwest's
  cookie store (RFC 6265 rules) held by us: the HTTP screen's (in `Service`,
  used while *Keep cookies* is on, listed by `http_cookies`) and one per run
  (`Experiment::cookies`, default on; files before version 8 open with it off
  so they run as before). *Send now* uses none.
- **A WebSocket is one socket, one owner.** `ws::Connection::spawn` hands the
  stream to a task that alone reads and writes; sends and closes reach it by
  channel, what arrives goes to a `ws::Sink` (the screen's batch, every 100 ms;
  a run's `Inbox`, read by *Wait for WebSocket* like any wait) and to the
  Inspector under a per-second `inspect::Budget`, so a reply a millisecond after
  its request is never dropped. Dropping the last handle sends a close frame — a
  run that ends or is stopped says goodbye (`RunSockets` cleared in
  `CancelBranches`). A run's connections open when *WebSocket connect* runs (its
  URL and headers are templates, unlike waits' binds); the other three name it by
  id (`ws.connection_unknown`, `ws.not_connected`). wss:// uses reqwest's TLS
  stack (native-tls), so it trusts what https:// does.
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
  persist. No MQTT 5, no TLS — adding either is a decision, not a detail. The
  broker emulator (`emulator_mqtt.rs`) is the other side of the same rules,
  on the same codec (`mqtt_codec`'s broker side): a client asking to keep its
  session gets a fresh one.

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
  every incoming wire; End passes once, in `experiment_flow::run` after the last
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
  to actions: the runner sends again (`experiment_flow::repeated`), rendering
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
  step to add in `tests/e2e/steps/` (and `STEPS` in `tour.ts`, the plan in
  `scripts/e2e.mjs`); `npm run build` type-checks the tour
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
