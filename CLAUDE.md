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
npm run build              # tsc (strict) + vite build  -> dist/
npm run tauri build        # installer/exe -> src-tauri/target/release/bundle/
cd src-tauri && cargo test # engine unit tests
cd src-tauri && cargo check --all-targets
python scripts/gen-icon.py && npx tauri icon src-tauri/icons/icon-1024.png  # app icon
```

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
  views/*.tsx             one screen per module
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

- **Keep `cargo test` green**: it covers the OSC codec, the CIDR/target
  resolver, socket-option paths, the capture ring and the signal library.

## Responsible use

Storm, Scanner and Broadcast emit real traffic at real hosts, and sweep/
broadcast reach the whole segment. Engine guard rails cap a sweep at 1024 hosts
and a beacon at 50 000 pps — guard rails, not permission. **Never commit a
default target pointing at a host we don't own.**
