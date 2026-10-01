# Signal Lab — developer guide

OSC & network-protocol simulator. **Tauri 2 shell + React 19/TypeScript UI +
native Rust networking engine.** Windows is the first-class target; the whole
app is one process — the UI calls the engine through `invoke`, the engine
streams telemetry back as Tauri events.

Full product documentation lives in [README.md](README.md); this file is the
short version for working on the code.

## Commands

```bash
npm install                # once
npm run tauri dev          # native window + Vite HMR + Rust rebuilds  <- normal dev loop
npm run check              # EVERYTHING CI checks — run before every push
npm run check:linux        # the same on Linux, in Docker
npm run build              # tsc (strict) + vite build  -> dist/
npm run tauri build        # Windows installers -> src-tauri/target/release/bundle/
npm run build:linux        # .deb/.rpm/.AppImage in Docker -> artifacts/linux/
npm run release -- X.Y.Z --dry-run   # then --push; see docs/delivery.md
cd src-tauri && cargo test # engine unit tests
python scripts/gen-icon.py && npx tauri icon src-tauri/icons/icon-1024.png  # app icon
```

**Delivery** ([docs/delivery.md](docs/delivery.md)): `scripts/check.mjs` is the one
list of checks — CI (`.github/workflows/ci.yml`, Windows + Linux) runs that file, so
add a check there, not in YAML. clippy runs with `-D warnings` and `--locked`: no
new warnings, no lock-file drift. The version is written only in `package.json`
(`node scripts/version.mjs set X.Y.Z` updates the Rust copies). Note user-visible
changes under *Unreleased* in `CHANGELOG.md` — it becomes the release notes.
Releases are tags pushed by `npm run release`; `.github/workflows/release.yml`
builds a draft release, a person publishes it. Never upload installers by hand.

`npm run dev` alone serves the UI at http://localhost:1420, but **every engine
call fails there** — there is no Tauri runtime in a plain browser. Use it only
to look at layout/CSS; anything functional needs `npm run tauri dev`.

Prerequisites: Node 18+, Rust stable, MSVC build tools, WebView2 (ships with
Win 10/11). All present on this machine as of the initial setup.

## Layout

```
src/                      React UI
  lib/api.ts              typed invoke wrappers + event channels — mirrors the Rust types
  lib/store.tsx           shared jobs + console state (StoreProvider)
  lib/i18n.tsx            locale provider, t() with {placeholder} interpolation
  lib/locales/en.ts       source of truth for every string; ru.ts is typed against it
  components/Scope.tsx    canvas oscilloscope (no chart library)
  components/OscArgs.tsx  typed OSC argument editor (OSC + Broadcast + Signals share it)
  components/Palette.tsx  Ctrl+K signal palette, mounted once in the shell
  lib/signals.ts          firing, describing and capturing signals
  lib/experimentGraph.ts  pure graph edits for the experiment editor (add, splice, layout)
  lib/experimentData.ts   template suggestions, upstream variables, JSON paths
  lib/errors.ts           describeError: one renderer for engine errors and legacy text
  components/ErrorMessage.tsx  where · what — why, technical detail folded
  views/*.tsx             one screen per module; ExperimentView is the node editor
src-tauri/src/
  lib.rs                  Tauri builder: manages JobRegistry + Capture, registers commands
  commands.rs             thin #[tauri::command] layer — no logic here
  engine/osc_codec.rs     hand-written OSC 1.0 encode/decode (no external OSC crate)
  engine/osc.rs           monitor + waveform generator
  engine/broadcast.rs     broadcast / multicast / CIDR sweep emitter + discovery listener
  engine/inspect.rs       the capture bus: bounded ring + batch pump to the UI
  engine/http.rs          request runner + concurrent burst
  engine/netsim.rs        UDP impairment relay (latency/jitter/loss/dup/corrupt)
  engine/storm.rs         UDP/TCP load generator
  engine/error.rs         EngineError {code, params, node, field, detail}
  engine/transport.rs     network failure causes (refused, timeout, dns, …)
  engine/experiment.rs    the document model: nodes, edges, outputs, versions
  engine/experiment_validate.rs  structural and run-time validation
  engine/experiment_data.rs  parameters, templated fields, extraction, value checks
  engine/experiment_actions.rs  one network action, failure classified
  engine/experiment_steps.rs    what one step does (send, check, extract, branch, wait)
  engine/experiment_run.rs      the runner: branches, joins, listeners, reports, Send now
  engine/matching.rs      OSC address patterns, argument rules, UDP payloads, comparisons
  engine/listen.rs        wait listeners: one socket per bind, bounded queue
  engine/template.rs      the {{template}} language and seeded generators (pure)
  engine/secrets.rs       credential-store secrets, masking, Inspector redaction
  engine/experiment_files.rs  load/save/import/export, version migration
  engine/scan.rs          TCP connect scanner
  engine/mqtt_codec.rs    hand-written MQTT 3.1.1 codec (no external crate)
  engine/mqtt.rs          one broker connection as a job; MqttHub routes commands
  engine/signals.rs       signal library: file + starter set (storage only)
  engine/jobs.rs          job registry: start / list / stop
```

## Invariants worth not breaking

- **Adding an engine command** means four edits: the function in `engine/*.rs`,
  a wrapper in `commands.rs`, the name in the `generate_handler!` list in
  `lib.rs`, and a typed wrapper in `src/lib/api.ts`.
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
  store (`engine/secrets.rs`); no command returns one, and while a run or a
  *Send now* uses them every string it reports is passed through
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

- **Keep `cargo test` green**: it covers the OSC codec, the CIDR/target
  resolver, socket-option paths, the capture ring and the signal library.

## Responsible use

Storm, Scanner and Broadcast emit real traffic at real hosts, and sweep/
broadcast reach the whole segment. Engine guard rails cap a sweep at 1024 hosts
and a beacon at 50 000 pps — guard rails, not permission. **Never commit a
default target pointing at a host we don't own.**
