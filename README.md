# Signal Lab

A lightweight, cross-platform simulator and toolbox for **OSC signals, HTTP,
network impairment, traffic storms, and port scanning** — with live signal
display. Built with **Tauri 2 + React/TypeScript** on a native **Rust** networking
engine, so it ships as a small binary yet has full raw UDP/TCP access.

> Windows is the first-class target; Android/iOS run as companions
> (see [Mobile](#mobile-androidios)).

---

## Modules

| Module | What it does |
| --- | --- |
| **OSC** | Send OSC 1.0 messages with typed arguments, monitor an incoming port with live decoding, and drive continuous waveforms (sine / triangle / saw / square / ramp / random) into any endpoint with an on-screen oscilloscope. |
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
  components/Scope.tsx   canvas oscilloscope / charts (no chart libs)
  views/*.tsx            one screen per module
src-tauri/src/engine/    the Rust engine
  osc_codec.rs           self-contained OSC 1.0 encoder/decoder (no deps)
  osc.rs                 monitor + waveform generator
  http.rs                request runner + concurrent burst
  netsim.rs              UDP impairment relay
  storm.rs               UDP/TCP load generator
  scan.rs                TCP connect scanner
  jobs.rs                job registry (start / list / stop)
  commands.rs            thin #[tauri::command] layer
```

The engine uses `tokio` for async sockets and `reqwest` (native-tls / schannel on
Windows) for HTTP. The OSC codec is hand-written, so there are no OSC crate
version risks and the dependency tree stays small.

---

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

Mobile targets are companions — great for the **OSC sender/monitor/generator** and
the **HTTP** tools. Raw traffic **storms** and **port scans** are constrained by
the iOS/Android network sandbox and should be run from the desktop build.

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

The **Storm** and **Scanner** modules generate real traffic and probe real hosts.
Only point them at systems you own or are explicitly authorized to test.

## License

Internal Hello tool. © Hello.
