# Signal Lab

[![License: MIT](https://img.shields.io/badge/License-MIT-3ee6b0.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/engine-Rust-b7410e.svg)](engine)
[![Tauri 2](https://img.shields.io/badge/shell-Tauri%202-24c8db.svg)](https://tauri.app)

*An open-source tool by [ProAnimaStudio](https://github.com/ProAnima).*

A lightweight, cross-platform simulator and toolbox for **OSC signals, HTTP,
MQTT, emulated APIs and devices, network impairment, broadcast/discovery,
traffic storms, and port scanning** —
with live signal display, a cross-protocol packet inspector, and a library of
named signals you can fire again. Built with
**Tauri 2 + React/TypeScript** on a native **Rust** networking engine, so it
ships as a small binary yet has full raw UDP/TCP access. The same engine and
interface also run **headless as a server, used from a browser** — on a rack PC or
a Linux box next to the gear, or as the Docker image `ghcr.io/proanima/signallab`
(see [Run as a server](#run-as-a-server-browser-docker)).

The interface is a dark instrument panel and ships **bilingual (English /
Russian)**; it picks the language from the OS on first run and remembers the
choice. See [Interface & localization](#interface--localization).

> Windows is the first-class target; Android/iOS run as companions
> (see [Mobile](#mobile-androidios)).

---

## Modules

| Module | What it does |
| --- | --- |
| **Experiments** | A node canvas for mixed HTTP, OSC, UDP, TCP and MQTT tests. Add a node after the selected one with `A`, drag a wire out of a port to create or connect the next step, send any single action node on its own, branch on a status or a value, and run work in parallel — several wires out of one output (Start included) run their nodes at the same time, and a Join waits for all of them. Focus and native fullscreen modes give the graph more room. Runs highlight each step and save a JSON report under `Documents/SignalLab/runs/`. Event listeners, loops and retries are planned in [ROADMAP.md](ROADMAP.md). |
| **Emulators** | Signal Lab as the other side: the API, device or service your system talks to. An **HTTP API** answers by routes (method, a path with `:name` segments, conditions on headers, query, body or JSON) with responses in sequence — 500, 500, then 200 for retries — in turn, or a seeded mix by weight, with delays, jitter and faults (no answer, a closed connection). An **OSC, UDP or TCP device** answers by rules: on this address, payload or line, reply that — to the sender or elsewhere, after a delay; a TCP device greets and can hang up. Replies are templates read with what arrived (`{{request.params.id}}`, `{{request.args[0]}}`). Each exchange is counted per rule, listed live and sent to the Inspector. **Mock this** on the HTTP screen turns a response into a route. The library is `Documents/SignalLab/emulators.json`, starting with a demo API and a demo OSC, UDP and TCP device on loopback. |
| **Signals** | The library: a named, editable packet you can fire again — OSC, raw UDP, an HTTP request or an MQTT publish — in nested folders that open and close, searchable, and fired from anywhere with `Ctrl+K`. *Save…* on the HTTP, OSC and MQTT screens files what you just sent into a folder and keeps the screen tied to it: *Save* (`Ctrl+S`) updates it, *Save as…* copies it, and a chip says where it lives and whether it changed. Folders are made, renamed (`F2`), dragged and removed (their contents move up); *Open in…* loads a signal back into its screen. Ships with the recipes for the gear it was written against, saves itself as hand-editable JSON in `Documents/SignalLab/signals.json`, and turns any frame the Inspector caught into a byte-exact replay. |
| **OSC** | Send OSC 1.0 messages with typed arguments, monitor an incoming port with live decoding, and drive continuous waveforms (sine / triangle / saw / square / ramp / random) into any endpoint with an on-screen oscilloscope. |
| **MQTT** | Connect to a broker, subscribe to `#` and watch every topic it holds build up as a live tree — last value, retain flag, QoS, message count. Publish at QoS 0/1/2, announce a last will, and **clear a retained value** (the empty-payload trick), which is the one thing a stuck broker needs and no other tool makes easy. MQTT 3.1.1, hand-written, plain TCP. |
| **Broadcast** | Fan a payload — OSC, text, or raw hex — out to a **list** of hosts, a **broadcast** address (`SO_BROADCAST`), a **multicast** group, or every host in a **CIDR sweep**. One-shot or as a repeating beacon. The paired **discovery listener** joins multicast groups, tables every peer that answers, and can auto-reply to impersonate a device. |
| **Inspector** | One timeline for every module, in the bottom panel next to the console so it is there on every screen: each OSC send, monitor packet, beacon, discovery probe and impaired relay frame, decoded, with a hex dump and the relay's verdict on it. Its tab shows when capture is on and how many frames it holds; the panel can be maximised. Filter by protocol / direction / text, then export the buffer to `.jsonl` or `.txt`. |
| **HTTP** | Inspect a single request/response (status, latency, headers, body), then run a concurrent **load burst** with live RPS and min/avg/max latency (percentiles and rate profiles are planned in [ROADMAP.md](ROADMAP.md)). |
| **Impairment** | A UDP relay that sits between a client and a target and injects **latency, jitter, packet loss, duplication and corruption** — a software network conditioner. |
| **Storm** | A controlled **UDP/TCP traffic generator** for stress-testing your own servers, with live pps / Mbps metering and a bounded duration. |
| **Scanner** | Concurrency-bounded **TCP connect port scan** with best-effort service banners and progress. |

Every long-running action is a **job**: it streams telemetry to the UI over Tauri
events and can be stopped individually or all at once from the console strip.

### Working with an experiment

The **Experiments** button beside the document name opens the templates:
empty, HTTP status check, HTTP → OSC with a status branch, parallel flows,
*OSC ping → reply* (send `/ping` with the run id, wait for `/pong` carrying it),
*Poll until ready* (ask a device for `/status` until it answers `ready`), and
*Retry a flaky API* (an emulated API that fails twice, and a loop that asks until it answers).
The same dialog imports JSON files of any earlier version (up to 4 MiB; they are
migrated on open) and exports the current document to
`Documents/SignalLab/exports/`. Import checks the file before showing a preview;
**Open experiment** replaces the canvas as one undo action. Invalid files leave
the current experiment intact. Export includes parameters and positions; opening
a template or file does not run it. Incomplete connections are valid drafts.

Building a flow is mostly the keyboard: select a node, press `A`, type a few
letters of the node you want and press Enter. The new node is wired in **after
the selected one** (spliced into its existing connection, or onto its free Yes/No
output), downstream nodes make room, and its main field — URL, OSC address, topic,
delay — is focused and selected, so you can type straight away. `Escape` returns
from the form to the canvas for the next `A`. With the mouse, drag from an output
port onto a node to connect it, or onto empty canvas to create the next node right
there; the ＋ on a wire inserts into it. A click on a wire selects it — the
properties show what it connects — and `Delete` removes it; hovering a wire also
shows a × above its ＋. Saved Signals appear in the same menu and become nodes with
their parameters filled in.

The console, the properties pane and the run timeline have handles on their edges:
drag one, or focus it with Tab and use the arrow keys (Shift for bigger steps); a
double click or Enter gives the pane its default size back. Sizes, and whether the
console is open, are kept for the next time.

**Parameters** (`{ }` in the toolbar) hold values such as `api = http://127.0.0.1:8080`;
any text field can use them as `{{api}}/login`. Typing `{{` (or `Ctrl+Space`) suggests
parameters, variables set upstream and generators (`{{uuid}}`, `{{now.iso}}`,
`{{counter}}`, `{{random_int(1, 100)}}`, `{{pick(a, b)}}`). **Extract value** saves a JSON
field, header, status, body or regex match of the latest response as a variable;
**Check value** and **Branch on value** compare it. The properties panel previews what
a node will send with current values. *Send now* on a request also fills in the
variables of the Extract nodes after it, and clicking a value in its JSON response
inserts an Extract node for that path. Every run records its seed; **Pin** stores it in
the experiment so random values repeat exactly.

**Profiles** are named sets of parameter values — *Stage*, *Venue* — edited as tabs in
the Parameters panel: a profile overrides some parameters and inherits the rest. The
switch in the toolbar chooses the profile used by runs, the preview and *Send now*,
and marks with ⚠ a profile that would fail validation before you switch to it.
**Run with…** (the arrow next to Run) runs once with another profile, changed values
or a given seed without changing the experiment; the timeline shows what was used.

**Secrets** — tokens and passwords — are written as `{{secret.API_TOKEN}}` and stored in
the Windows Credential Manager from the Secrets section of the Parameters panel; the
experiment file keeps only the names. Values never reach the interface: *Send now* runs
in the engine, the preview shows `••••`, and every value is masked in the timeline,
responses, run reports and the Inspector while it is in use. A run is refused before
any traffic if a secret it needs is not stored on this computer. The full language is described in
[docs/milestone-3-data.md](docs/milestone-3-data.md).

Outputs that still need a wire pulse amber and nodes that Start cannot reach are
drawn dashed. **Complete the graph** in the toolbar (and **Run** on an invalid
graph) names the problem and jumps to the node. **Send now** (`Ctrl+Enter`) on an
HTTP, OSC, UDP or MQTT node sends just that step through the same path as the
direct instruments and shows the result — for HTTP, the formatted response —
without running the experiment.

An **Emulator** node plays a dependency for the whole run: it opens before the first
step, answers until the run ends, and the run report counts what it received (*Edit…*
opens its rules; it can be taken from, or kept in, the emulator library). **Wait for
HTTP request** then checks what the system under test sent it — method, path and
conditions — and later steps read `{{request.json.…}}`; on an address without an
emulator, the run's own listener answers 204. An OSC or UDP emulator shares its port
with the run's waits.

The OSC and HTTP screens have **Add to experiment**, which appends the message or
request you just tried as the next step. Their fields are kept across screens and
restarts, and a one-line verdict appears under **Send**; the app reopens on the
screen you used last.

Use **Nodes** to find an existing node by type, URL, payload, or ID and jump to it.
**Arrange** places the graph from left to right; **Fit graph** shows its full extent.
Duplicate creates an unconnected copy with independent parameters. Drafts are
autosaved even while connections are incomplete.

| Shortcut | Action |
| --- | --- |
| `A` | Add a node after the selected one (or mid-view when nothing is selected) |
| Double-click empty canvas | Add a node there |
| Drag from an output port | Connect to the node you drop on, or add the next node on empty canvas |
| `Ctrl+Enter` | Send the selected action node on its own |
| `{{` or `Ctrl+Space` in a field | Suggest parameters, variables and generators |
| `Ctrl+Z` / `Ctrl+Shift+Z` (also `Ctrl+Y`) | Undo / redo |
| `Ctrl+D` | Duplicate selected action or check |
| `Ctrl+F` | Find and reveal a node |
| `Ctrl+0` / `Ctrl+1` | Fit graph / actual size |
| `Ctrl` + wheel | Zoom around the pointer |
| Arrow keys on a focused node | Move by 5 px; hold `Shift` for 20 px |
| `Delete` | Remove the selected action or check |
| `Escape` | Leave a node's form for the canvas; close the active popup/connection; leave fullscreen/focus mode |

Text fields retain their native editing shortcuts. A node drag or field-edit
session is one undo action. Up to 100 actions are kept for the current app
session, including while switching between protocol instruments.

---

## Architecture

```
src/                     React + TypeScript UI (Vite)
  lib/transport.ts       desktop (Tauri invoke) or browser (fetch + one WebSocket)
  lib/platform.ts        fullscreen, downloads, sign-out on either platform
  lib/api.ts             typed command wrappers + event channels
  lib/store.tsx          shared jobs, console and signal-library state
  lib/signals.ts         firing, describing and capturing library signals
  lib/library.ts         library folders as paths: tree, rename, move, remove (pure)
  components/SaveSignal.tsx  Save… / Save / Save as… on the sending screens
  components/SignalTree.tsx  the folder tree: open/close, drag & drop, F2, Delete
  components/Splitter.tsx    a resizable pane edge (pointer and keyboard)
  components/TooltipLayer.tsx  the one tooltip: any data-tip, hover + keyboard focus
  lib/experimentGraph.ts graph editing, duplication and DAG layout
  lib/experimentData.ts  template suggestions, upstream variables, JSON paths
  lib/editHistory.ts     bounded, grouped document history (pure reducer)
  lib/useExperimentViewport.ts canvas zoom, fit and reveal behavior
  lib/useExperimentDocument.ts document load, history, validation and autosave
  lib/experimentTemplates.ts template catalog (shared JSON definitions)
  lib/errors.ts          one renderer for every failure: engine errors and legacy text
  components/ErrorMessage.tsx  where · what — why, with the technical detail folded
  lib/i18n.tsx           language provider, detection, useT()
  lib/translate.ts       placeholders, ICU plurals, numbers for a language (pure)
  lib/locales/index.ts   the list of languages
  lib/locales/en.ts      source-of-truth dictionary (every key)
  lib/locales/ru.ts      Russian, typed against en.ts
  components/Scope.tsx   canvas oscilloscope / charts (no chart libs)
  components/OscArgs.tsx typed OSC argument editor (OSC + Broadcast + Signals)
  components/Palette.tsx Ctrl+K palette that fires a signal from any screen
  components/Brand.tsx   ProAnimaStudio inline-SVG mark + lockup
  views/*.tsx            one screen per module
engine/src/              the Rust engine (crate signal-lab-engine, no Tauri)
  service.rs             the command table both front doors forward to
  host.rs                where events go (Tauri, WebSockets, a test recorder) + the capture bus
  paths.rs               the data folder
  osc_codec.rs           self-contained OSC 1.0 encoder/decoder (no deps)
  osc.rs                 monitor + waveform generator
  broadcast.rs           broadcast / multicast / sweep emitter + discovery listener
  inspect.rs             the capture bus every module publishes to
  error.rs               EngineError: code, values, node, field, detail
  transport.rs           network failure causes (refused, timeout, DNS …)
  experiment.rs          the document model: nodes, edges, outputs, versions
  experiment_validate.rs structural and run-time validation
  experiment_data.rs     parameters, templated fields, extraction and value checks
  experiment_actions.rs  one network action, its failure classified
  experiment_steps.rs    what one step does: send, check, extract, branch, wait
  experiment_run.rs      the runner: branches, joins, listeners, events, reports
  matching.rs            OSC address patterns, argument rules, UDP payloads, comparisons
  listen.rs              wait listeners: armed per bind, bounded queues
  template.rs            the {{template}} language and seeded generators
  secrets.rs             secrets: OS credential store or read-only files, masking, redaction
  experiment_files.rs    JSON parsing, atomic working-file replacement and exports
  http.rs                request runner + concurrent burst
  netsim.rs              UDP impairment relay
  storm.rs               UDP/TCP load generator
  mqtt_codec.rs          self-contained MQTT 3.1.1 codec (no deps)
  mqtt.rs                one live broker connection as a job + one-shot publish
  scan.rs                TCP connect scanner
  signals.rs             signal library file + starter set (storage only)
  jobs.rs                job registry (start / list / stop)
src-tauri/src/lib.rs     the desktop shell: one Tauri command into the engine, Tauri events back
server/src/              signal-lab-server (axum): HTTP commands, WebSocket events, sign-in, static UI
```

The three Rust crates form one Cargo workspace (one version, one `Cargo.lock`, one
`target/`). The desktop shell and the server are thin: both hand a command name and
its JSON arguments to `engine::Service`, so a command behaves the same in the app and
in a browser.

Bundled definitions live in `experiments/templates/`. The starter experiment and
template chooser use these same files; Rust tests check that each can run and
round-trip through the document format.

The engine uses `tokio` for async sockets, `socket2` for the socket options
tokio can't set before bind (`SO_REUSEADDR`), and `reqwest` (native-tls /
schannel on Windows) for HTTP. The OSC and MQTT codecs are hand-written, so
there are no protocol-crate version risks and the dependency tree stays small.

### The capture bus

Modules never talk to the Inspector directly — they publish a normalized `Frame`
to `engine::inspect`, which keeps a bounded ring buffer and pumps batches to the
UI. Two consequences worth knowing:

- **Capture is armed explicitly.** While disarmed, publishing is one atomic load,
  so the hot paths cost nothing.
- **High-rate sources are sampled, not dropped silently.** The waveform generator,
  beacons and the impairment relay publish through a rate gate; anything the UI
  never drew is counted and shown as *"N not shown"*, and the full ring is still
  what `Export` writes.

Exports land in `~/Documents/SignalLab/capture-<epoch-ms>.jsonl`.

---

## Interface & localization

The UI is fully bilingual (**English / Russian**), switched live from the header
— no reload, and the console re-translates its backlog too. It works like this:

- `src/lib/locales/en.ts` is the **source of truth**. Its keys define the
  `Dict` type; `ru.ts` is typed against it, so a missing or misspelled key is a
  **compile error**, never a blank label at runtime.
- `t("key", { name })` fills `{name}` placeholders; counts use ICU plurals —
  `{n, plural, one {# signal} other {# signals}}` — chosen by each language's
  own rules (`Intl.PluralRules`: Russian has *one*, *few*, *many*). Numbers and
  sizes are written the language's way. Unknown keys fall back to English, then
  to the key string itself.
- **Errors are translated too.** The engine never builds a sentence: every
  command and every job reports an `EngineError` — a stable `code`, values, the node and field it is
  about, and the system's own wording as `detail`. `src/lib/errors.ts` renders it
  as *Node · Field — message* from `err.<code>` and `field.<key>`, with the
  detail folded underneath, in the banner, the properties panel, the timeline,
  *Send now*, the console and the HTTP screen alike. Network failures are
  classified (refused, timeout, name not found, unreachable, port in use, TLS …)
  because each has a different fix. A `cargo test` scans the engine for every
  code and field key and fails if `en.ts` has no text for one. The jobs in the
  console strip are named the same way (`job.<kind>` with their values).
- Console log lines store a **key + params**, not finished text, so switching
  language re-renders the whole history in the new one.
- The initial language is the first of the system's languages
  (`navigator.languages`) that Signal Lab has, and the choice persists in
  `localStorage`. The starter signals are written in it on first run; the
  server's sign-in page follows the browser's `Accept-Language`.
- Help is in tooltips, in the current language, on hover and on keyboard focus —
  a field shows its label's.

**Adding a language** is a dictionary and one line in `src/lib/locales/index.ts`;
[docs/localization.md](docs/localization.md) walks through it and lists what the
checks catch (missing texts, other placeholders, missing plural forms).

**Translating a label?** Keep single words short, or let them wrap — metric
captions sit in ~112px cards. `.metric .k` uses `overflow-wrap: anywhere` as a
safety net, but a long compound word still reads badly.

### UI conventions

Worth knowing before editing the interface:

- **Everything clickable is a real `<button>`** — the sidebar nav and the
  filter/mode "chips" included. They look like plain elements but need focus
  rings and Space/Enter, so they must not become `<div>`/`<span>` again.
- **`--text-faint` carries 9.5px labels** and is tuned to clear WCAG AA
  (≥4.5:1) on all three surfaces. Darkening it fails contrast where it hurts
  most.
- **Enter submits** in the OSC sender, the Broadcast target, and the HTTP URL.
- **The console collapses** to its header bar (the chevron at its left); the
  running-jobs strip stays visible either way. At the 900×600 minimum window
  that hands ~146px back to the module.
- Click targets that belong next to a field go *beside* the `<label>`, never
  inside it — a click inside a label also activates the labelled input.
- **No captions, tooltips instead.** The screen shows labels, values, states and
  errors. An explanation, a shortcut or what `0` means goes in `data-tip` on the
  control or its label; `components/TooltipLayer.tsx` shows it on hover and on
  keyboard focus, in the current language. The native `title` attribute is not
  used (a test fails on it).
- **Switching screens loses nothing.** A screen is mounted the first time it is
  opened and then kept, hidden, for the session: typed values, the last
  response, a running monitor's list and its job, and the scroll position are
  all where they were left. Fields you would want again after a restart (the
  OSC message, the HTTP request) are also stored in `localStorage`.

## Prerequisites

- **Node.js** 22.18+ and npm (editor tests use native TypeScript stripping)
- **Rust** (stable) — install via [rustup](https://rustup.rs)
- **Windows:** Visual Studio C++ build tools (MSVC) + WebView2 runtime
  (WebView2 ships with Windows 10/11)

## Develop

```bash
npm install
npm run tauri dev
```

This launches the Vite dev server and the native window with hot reload for the
UI and automatic Rust rebuilds.

Before pushing, run **`npm run check`** — the same checks CI runs on Windows and
Linux, in order: versions agree, UI tests, TypeScript + production build, `clippy`
with warnings as errors, Rust tests for the whole workspace. `npm run check:linux` runs
them on Linux in Docker, and `npm run check:image` builds and smoke-tests the server
image.

**`npm run e2e`** walks every screen of the real app — the desktop app and the
server in a browser — sending real traffic to loopback stand-ins (an HTTP API, UDP and
TCP sinks, an OSC device, an MQTT broker) and checking what arrived; `npm run e2e:linux`
does the same on Linux in WebKitGTK, and CI runs both for every change and for the
server image. Screenshots of every step land in `artifacts/e2e/`. How CI, releases and
server mode work: [docs/delivery.md](docs/delivery.md).

## Build a desktop bundle

```bash
npm run tauri build
```

Produces two Windows installers under `target/release/bundle/`:

| Artifact | Use |
| --- | --- |
| `nsis/Signal Lab_<version>_x64-setup.exe` | normal install: English or Russian, for you (no admin rights) or for everyone, *Run* and a desktop shortcut at the end; `/S` installs silently |
| `msi/Signal Lab_<version>_x64_en-US.msi` | unattended / group-policy deployment (`msiexec /i … /qn`) |

`target/release/signal-lab.exe` is the bare executable and needs no
installation at all — handy for a USB stick on a show site. The release profile
is size-optimized (`opt-level = "s"`, LTO, stripped).

The app icon is redrawn from the in-app brand mark (`src/components/Brand.tsx`)
by `scripts/gen-icon.py`; feed the 1024px result to `npx tauri icon` to cut the
platform set. The installers' sidebar, header, dialog and banner come from that
icon too: `python scripts/gen-installer-art.py` (unattended switches and the rest:
[docs/delivery.md](docs/delivery.md#installers)).

**Linux packages** from this machine: `npm run build:linux` builds `.deb`, `.rpm`
and `.AppImage` in Docker on Ubuntu 22.04 (the system CI uses) into
`artifacts/linux/`, with `SHA256SUMS.txt`. Installed packages need WebKitGTK 4.1.

## Run as a server (browser, Docker)

`signal-lab-server` is the engine and the interface without a window: open it in
Chrome, Firefox or Edge and every screen works as in the app — runs, the Inspector,
reports and exports download from the browser.

**On a Linux machine, in one command:**

```bash
curl -fsSL https://raw.githubusercontent.com/ProAnima/SignalLab/main/deploy/install.sh | sh
```

It installs Docker if it is missing (it asks first), starts the server with host
networking, waits until it answers and prints the address to open and the access
token to sign in with. Run it again to update — the data and the token stay;
`--uninstall` removes it (`--purge` with its data). Options: `--version`, `--port`,
`--dir`, `--name`, `--yes`; `sh install.sh --help`.

**Docker by hand** (the image is published with each release, x64 and arm64):

```bash
docker run -d --name signallab --network host --restart unless-stopped \
  -v signallab-data:/data --read-only --cap-drop ALL --security-opt no-new-privileges \
  ghcr.io/proanima/signallab:latest
docker exec signallab cat /data/token     # the token it made on its first start
```

Then open `http://<host>:1430` and sign in with the token. The same as a Compose file:
[`deploy/compose.yaml`](deploy/compose.yaml) — `docker compose up -d`.

| Networking | What works |
| --- | --- |
| `--network host` (Linux) | everything: OSC/UDP/TCP/HTTP/MQTT to the LAN, listening ports, **broadcast, multicast, discovery** |
| bridge with published ports | unicast, and listeners on published ports — no broadcast or multicast |
| Docker Desktop (Windows/macOS) | unicast only — use the desktop app on those systems |

**Without Docker** (from a checkout): `npm run build`, then
`cargo run --release -p signal-lab-server` — it serves `dist/` on
`http://127.0.0.1:1430` for this machine only, no token needed.

**Security.** Without a token the server listens only on loopback and refuses to
start on any other address. With one (`--token-file`, 24+ characters,
`signal-lab-server token` makes one; the image makes and keeps its own in
`/data/token`, `--generate-token`) a browser signs in once and gets an `HttpOnly`,
`SameSite=Strict` session; scripts send `Authorization: Bearer`. Requests must come
from the server's own origin and host name, every job start is logged with the
client's address, and experiment secrets are read-only files in
`/run/secrets/signallab/`. Put TLS in a reverse proxy and add `--secure-cookie`.
All options and their environment variables: `signal-lab-server --help` and
[docs/delivery.md](docs/delivery.md#7-server-mode-and-docker-image).

## Automation, CI/CD and the API

`signallab` runs the same experiments without a window — in a pipeline, a cron
job or a deploy script — and exits with a code a pipeline understands:

```bash
signallab run tests/smoke.json --param api=http://127.0.0.1:8080 --junit junit.xml
signallab run tests/stage.json --server http://lab-pc:1430 --token-file token.txt
signallab send osc 127.0.0.1:9000 /cue/go f:0.75
signallab emulate tests/payments-mock.json --for 120 &   # the dependency, while the app is tested
```

`0` all passed, `1` one failed, `2` invalid input, `3` could not run. Steps print as
they happen in English or Russian, `--json` gives a JSON object per line, `--junit`
a report every CI system shows, and `--server` sends the runs to a lab PC that can
reach the gear. It comes with the desktop app — the installers put it on `PATH`
(`/usr/bin` on Linux) — and in the image; `signallab doctor` says what stands
between it and the gear, the firewall included. A GitHub Action wraps it:

```yaml
- uses: ProAnima/SignalLab@v0.4.0
  with:
    experiments: tests/signallab/*.json
```

**For an assistant:** `signallab mcp` is a Model Context Protocol server, so an
LLM in Claude Code, Claude Desktop, Cursor or VS Code can read what an experiment
is made of, write one, validate and run it, send single messages, listen on a
port, and start an emulator and read what it received — here, or on a lab server
with `--server`. `signallab mcp --print-config claude-code` prints the line to add it.

The server's API does the same over HTTP: `POST /api/run` waits for a run and
answers with its result, or streams its steps as NDJSON; `/api/invoke/<command>`
is the engine's whole command table; `/api/openapi.json` describes it all. Commands,
exit codes, secrets, GitLab and shell recipes: [docs/automation.md](docs/automation.md).

## Releases

Releases are built by GitHub Actions from a version tag, never uploaded by hand:

```bash
npm run release -- 0.4.0 --dry-run   # check everything, change nothing
npm run release -- 0.4.0 --push      # version, changelog, commit, tag, push
```

The tag builds the Windows and Linux installers into a **draft** release with
`SHA256SUMS.txt` and notes taken from `CHANGELOG.md`; review it and publish it on
the releases page. Publishing builds the server image for x64 and arm64, smoke-tests
it and pushes it to `ghcr.io/proanima/signallab` with an SBOM and signed provenance. Write user-visible changes under *Unreleased* in `CHANGELOG.md`
as they land — that text is the release notes. Details and the rules the release
script enforces: [docs/delivery.md](docs/delivery.md).

---

## Mobile (Android/iOS)

Mobile targets are companions — great for the **OSC sender/monitor/generator**,
the **HTTP** tools and the **Inspector**. Raw traffic **storms**, **port scans**
and **CIDR sweeps** are constrained by the iOS/Android network sandbox and should
be run from the desktop build. Broadcast and multicast additionally need the
local-network permission (iOS 14+ prompts for it; Android needs a held multicast
lock for some devices), so treat them as desktop-first too.

Initialize the mobile projects once:

```bash
npm run tauri android init
npm run tauri ios init      # macOS only
```

Then run on a device/emulator:

```bash
npm run tauri android dev
npm run tauri ios dev
```

Android needs the Android SDK/NDK + `ANDROID_HOME`; iOS needs Xcode. Because the
generators use UDP, allow the app through the device firewall / local-network
permission prompt.

---

## Responsible use

The **Storm**, **Scanner** and **Broadcast** modules generate real traffic and
probe real hosts. Only point them at systems you own or are explicitly authorized
to test. Broadcast and sweep reach *every* device on the segment, not just the one
you had in mind — check which network you are on first. Guard rails in the engine
cap a sweep at 1024 hosts and a beacon at 50 000 packets/s aggregate, but they are
guard rails, not permission.

## Contributing

Issues and pull requests are welcome. A few things worth knowing:

- `cargo test --workspace` covers the OSC codec, the CIDR/target resolver, the
  socket-option paths, the capture ring, an experiment run end to end and the
  server's security rules. `npm run check` runs it with everything else; please
  keep it green.
- `npm run build` runs `tsc` in strict mode — it will catch a missing
  translation key for you.
- The UI has invariants that are easy to undo by accident (keyboard-reachable
  controls, label contrast, no click targets inside `<label>`). See
  [UI conventions](#ui-conventions) before changing the interface.
- **Never** add a default target that points at a host you do not own. The
  Storm and Scanner modules generate real traffic.

## License

[MIT](LICENSE) © 2026 ProAnimaStudio.

### Experiment node catalogue

The editor supports Start/End, HTTP requests, OSC messages, UDP datagrams, TCP,
MQTT publishing, Log, Delay, **Wait for OSC**, **Wait for UDP**, **Wait for MQTT** and
**Wait for HTTP request**, the **Emulator**, Extract, value checks and branches, status
branching, parallel branch/join, **Loop**, and HTTP status/body/header/latency checks.
The palette groups actions, waits (*Observe*), *Emulate*, data, checks and flow; search
supports Russian/English labels and protocol names,
with arrow-key selection and Enter to insert. HTTP node forms include request
headers, and OSC forms include typed arguments.

A wait listens on `bind` (`IP:port`) from the moment the run starts, so a device
that answers faster than the next step begins is not missed; replies count from
the latest request on the same path. *Wait for OSC* matches an OSC 1.0 address
pattern (`*`, `?`, `[0-9]`, `{ping,pong}`) and optional argument rules
(`args[0] equals {{nonce}}`); *Wait for UDP* matches any datagram, text, a regular
expression or a hex byte sequence. **Matched** continues with the reply in a
variable (`{{reply.address}}`, `{{reply.args[0]}}`, `{{reply.text}}`,
`{{reply.from}}`, `{{reply.ms}}`); an optional **Timeout** wire handles silence —
without it a timeout fails the step and says how many other messages arrived.
A port that cannot be opened stops the run before any traffic. *Listen now* on a
wait listens with that step alone.

Every step that sends or listens can **retry** (attempts, the same or a doubling
pause), and every step that sends can **repeat** — a number of times or for a time,
every so many milliseconds with an optional seeded jitter; `{{counter}}` numbers the
sends. An OSC message or UDP datagram can **wait for its reply** in the same step.

A **Loop** runs the steps on its *Body* output, which lead back to it, again and
again — at most a set number of times, and until an exit condition holds when it has
one (checked after each iteration, so the body can set what it tests). *Done*
follows; the optional *Limit* when the iterations ran out first. Inside the body
`{{counter}}` is the iteration's number; the wire back is the only one that may go
backwards, and a body runs as one branch (no parallel work inside).

MQTT nodes use unauthenticated brokers (QoS 0–2, retain, 15-second deadline).
Body checks search the bounded HTTP preview: a match succeeds; a missing match in
a truncated preview fails explicitly. Header names are case-insensitive; values
and body text are case-sensitive.
