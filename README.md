# Signal Lab

[![License: MIT](https://img.shields.io/badge/License-MIT-3ee6b0.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/engine-Rust-b7410e.svg)](src-tauri)
[![Tauri 2](https://img.shields.io/badge/shell-Tauri%202-24c8db.svg)](https://tauri.app)

*An open-source tool by [ProAnimaStudio](https://github.com/ProAnima).*

A lightweight, cross-platform simulator and toolbox for **OSC signals, HTTP,
network impairment, broadcast/discovery, traffic storms, and port scanning** —
with live signal display and a cross-protocol packet inspector. Built with
**Tauri 2 + React/TypeScript** on a native **Rust** networking engine, so it
ships as a small binary yet has full raw UDP/TCP access.

The interface is a dark instrument panel and ships **bilingual (English /
Russian)**; it picks the language from the OS on first run and remembers the
choice. See [Interface & localization](#interface--localization).

> Windows is the first-class target; Android/iOS run as companions
> (see [Mobile](#mobile-androidios)).

---

## Modules

| Module | What it does |
| --- | --- |
| **OSC** | Send OSC 1.0 messages with typed arguments, monitor an incoming port with live decoding, and drive continuous waveforms (sine / triangle / saw / square / ramp / random) into any endpoint with an on-screen oscilloscope. |
| **Broadcast** | Fan a payload — OSC, text, or raw hex — out to a **list** of hosts, a **broadcast** address (`SO_BROADCAST`), a **multicast** group, or every host in a **CIDR sweep**. One-shot or as a repeating beacon. The paired **discovery listener** joins multicast groups, tables every peer that answers, and can auto-reply to impersonate a device. |
| **Inspector** | One timeline for every module: each OSC send, monitor packet, beacon, discovery probe and impaired relay frame, decoded, with a hex dump and the relay's verdict on it. Filter by protocol / direction / text, then export the buffer to `.jsonl` or `.txt`. |
| **HTTP** | Inspect a single request/response (status, latency, headers, body), then run a concurrent **load burst** with live RPS and latency percentiles. |
| **Impairment** | A UDP relay that sits between a client and a target and injects **latency, jitter, packet loss, duplication and corruption** — a software network conditioner. |
| **Storm** | A controlled **UDP/TCP traffic generator** for stress-testing your own servers, with live pps / Mbps metering and a bounded duration. |
| **Scanner** | Concurrency-bounded **TCP connect port scan** with best-effort service banners and progress. |

Every long-running action is a **job**: it streams telemetry to the UI over Tauri
events and can be stopped individually or all at once from the console strip.

---

## Architecture

```
src/                     React + TypeScript UI (Vite)
  lib/api.ts             typed wrappers over Tauri invoke + event channels
  lib/store.tsx          shared jobs + console state
  lib/i18n.tsx           locale provider + t() with {placeholder} interpolation
  lib/locales/en.ts      source-of-truth dictionary (every key)
  lib/locales/ru.ts      Russian, typed against en.ts
  components/Scope.tsx   canvas oscilloscope / charts (no chart libs)
  components/OscArgs.tsx typed OSC argument editor (shared by OSC + Broadcast)
  components/Brand.tsx   ProAnimaStudio inline-SVG mark + lockup
  views/*.tsx            one screen per module
src-tauri/src/engine/    the Rust engine
  osc_codec.rs           self-contained OSC 1.0 encoder/decoder (no deps)
  osc.rs                 monitor + waveform generator
  broadcast.rs           broadcast / multicast / sweep emitter + discovery listener
  inspect.rs             the capture bus every module publishes to
  http.rs                request runner + concurrent burst
  netsim.rs              UDP impairment relay
  storm.rs               UDP/TCP load generator
  scan.rs                TCP connect scanner
  jobs.rs                job registry (start / list / stop)
  commands.rs            thin #[tauri::command] layer
```

The engine uses `tokio` for async sockets, `socket2` for the socket options
tokio can't set before bind (`SO_REUSEADDR`), and `reqwest` (native-tls /
schannel on Windows) for HTTP. The OSC codec is hand-written, so there are no
OSC crate version risks and the dependency tree stays small.

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
- `t("key", { name })` fills `{name}` placeholders. Unknown keys fall back to
  English, then to the key string itself — which is how raw engine error
  messages (not translatable) pass straight through.
- Console log lines store a **key + params**, not finished text, so switching
  language re-renders the whole history in the new one.
- The initial language comes from `navigator.language`, and the choice persists
  in `localStorage`.

**Adding a language:** copy `en.ts` to e.g. `de.ts`, translate the values, then
register it in `LANGS` and `DICTS` in `src/lib/i18n.tsx`. TypeScript will list
any keys you missed.

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

## Prerequisites

- **Node.js** 18+ and npm
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

## Build a desktop bundle

```bash
npm run tauri build
```

Produces an installer / executable under `src-tauri/target/release/bundle/`.
The release profile is size-optimized (`opt-level = "s"`, LTO, stripped).

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

- `cargo test` in `src-tauri/` covers the OSC codec, the CIDR/target resolver,
  the socket-option paths and the capture ring. Please keep it green.
- `npm run build` runs `tsc` in strict mode — it will catch a missing
  translation key for you.
- The UI has invariants that are easy to undo by accident (keyboard-reachable
  controls, label contrast, no click targets inside `<label>`). See
  [UI conventions](#ui-conventions) before changing the interface.
- **Never** add a default target that points at a host you do not own. The
  Storm and Scanner modules generate real traffic.

## License

[MIT](LICENSE) © 2026 ProAnimaStudio.
