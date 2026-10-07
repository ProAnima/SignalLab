<p align="center"><img src="docs/public/icon.png" width="88" height="88" alt=""></p>

<h1 align="center">Signal Lab</h1>

<p align="center">
  <b>The lab for the protocols your show, installation and IoT gear speaks.</b><br>
  Send, capture, emulate and impair OSC, UDP/TCP, HTTP, WebSocket and MQTT —<br>
  by hand, as repeatable experiments, and from CI or an AI assistant.
</p>

<p align="center">
  <a href="https://github.com/ProAnima/SignalLab/actions/workflows/ci.yml"><img alt="CI" src="https://github.com/ProAnima/SignalLab/actions/workflows/ci.yml/badge.svg"></a>
  <a href="https://github.com/ProAnima/SignalLab/releases/latest"><img alt="Release" src="https://img.shields.io/github/v/release/ProAnima/SignalLab?color=3ee6b0"></a>
  <a href="https://proanima.github.io/SignalLab/"><img alt="Documentation" src="https://img.shields.io/badge/docs-8%20of%2011%20languages-38c9ec"></a>
  <a href="LICENSE"><img alt="License: MIT" src="https://img.shields.io/badge/license-MIT-8b7cff"></a>
</p>

<p align="center">
  <a href="https://proanima.github.io/SignalLab/"><b>Documentation</b></a> ·
  <a href="https://github.com/ProAnima/SignalLab/releases/latest"><b>Download</b></a> ·
  <a href="https://proanima.github.io/SignalLab/server/">Server &amp; Docker</a> ·
  <a href="CHANGELOG.md">Changelog</a>
</p>

<p align="center">
  Documentation in:
  <a href="https://proanima.github.io/SignalLab/">English</a> ·
  <a href="https://proanima.github.io/SignalLab/ru/">Русский</a> ·
  <a href="https://proanima.github.io/SignalLab/es/">Español</a> ·
  <a href="https://proanima.github.io/SignalLab/fr/">Français</a> ·
  <a href="https://proanima.github.io/SignalLab/de/">Deutsch</a> ·
  <a href="https://proanima.github.io/SignalLab/pt/">Português</a> ·
  <a href="https://proanima.github.io/SignalLab/zh/">中文</a> ·
  <a href="https://proanima.github.io/SignalLab/ja/">日本語</a> ·
  <a href="https://proanima.github.io/SignalLab/ko/">한국어</a> ·
  <a href="https://proanima.github.io/SignalLab/hi/">हिन्दी</a> ·
  <a href="https://proanima.github.io/SignalLab/ar/">العربية</a>
</p>

---

Signal Lab is a desktop app for **Windows and Linux** and a **server** you use from
a browser (also as a Docker image). It is built for bringing up shows, installations
and networked devices before the rest of the system exists — and for testing the
services and APIs they talk to. In one window: an OSC sender and monitor, a UDP/TCP
terminal, an HTTP and WebSocket client, an MQTT client and broker, mock servers for
the devices and APIs that are not there yet, a network impairment relay, a load
tester and a test runner for CI.

## What you can do

| | |
| --- | --- |
| **Talk to devices** | [OSC](https://proanima.github.io/SignalLab/protocols/osc.html) with typed arguments, a monitor and a waveform generator; raw [UDP and TCP](https://proanima.github.io/SignalLab/protocols/udp-tcp.html); [HTTP](https://proanima.github.io/SignalLab/protocols/http.html) with Basic, Bearer and Digest, cookies and bursts; [WebSocket](https://proanima.github.io/SignalLab/protocols/websocket.html); [MQTT 3.1.1](https://proanima.github.io/SignalLab/protocols/mqtt.html) with QoS 0–2, retained messages and a last will; [broadcast, multicast and discovery](https://proanima.github.io/SignalLab/protocols/broadcast.html). |
| **See every byte** | The [Inspector](https://proanima.github.io/SignalLab/tools/inspector.html) captures what Signal Lab sends and receives, decoded and in hex; export it, or save a frame as a signal. |
| **Keep a library** | [Signals](https://proanima.github.io/SignalLab/tools/signals.html) in folders, fired from anywhere with <kbd>Ctrl</kbd>+<kbd>K</kbd>. |
| **Fake what is not there** | [Emulators](https://proanima.github.io/SignalLab/tools/emulators.html): HTTP APIs, OSC/UDP responders, TCP devices and an MQTT broker, with rules, templated replies, faults and outages. |
| **Break the network on purpose** | An [impairment relay](https://proanima.github.io/SignalLab/tools/impairment.html) for UDP and TCP: latency, jitter, loss and bursts of loss, duplication, corruption, reordering, a bandwidth limit, resets, half-open connections and going offline, with presets from a LAN to a satellite link. |
| **Push and probe** | [Storm](https://proanima.github.io/SignalLab/tools/storm.html), a controlled flood of UDP datagrams or TCP connections with the rate, throughput and errors live; the [Scanner](https://proanima.github.io/SignalLab/tools/scanner.html), a TCP connect scan with the greeting each service sends. |
| **Make it repeatable** | [Experiments](https://proanima.github.io/SignalLab/experiments/): a visual flow of sends, waits, checks, extraction, branches, loops, parallel branches, retries and fault phases, with parameters, profiles and secrets — seeded, reproducible, with a report of every run. |
| **Load it** | [Load profiles](https://proanima.github.io/SignalLab/experiments/load.html) on an HTTP request — constant, ramp, steps, spike, random arrivals — with thresholds on p95, errors and rate, and two runs compared. |
| **Automate it** | [`signallab`](https://proanima.github.io/SignalLab/automation/cli.html), the command line with JUnit reports for [CI and a GitHub Action](https://proanima.github.io/SignalLab/automation/ci.html); [MCP](https://proanima.github.io/SignalLab/automation/mcp.html) for AI assistants; an [HTTP API](https://proanima.github.io/SignalLab/api/) for everything the interface does. |

The interface speaks **eleven languages**: English, Russian, Spanish, French, German,
Portuguese, Chinese, Japanese, Korean, Hindi and Arabic. The documentation is complete
in eight of them; in Portuguese, Japanese and Arabic most of its pages are still being
translated.

## Get it

- **Windows** — `Signal.Lab_<version>_x64-setup.exe` (just for you, or for everyone) or the `.msi`
  from the [latest release](https://github.com/ProAnima/SignalLab/releases/latest).
  The installers are not code-signed yet, so SmartScreen may ask: *More info → Run anyway*.
- **Linux** — `.deb`, `.rpm` or `.AppImage` from the same page.
- **Server** — on a Linux host with Docker: in one command (it offers to install Docker
  if it is missing), or by hand:

  ```bash
  curl -fsSL https://raw.githubusercontent.com/ProAnima/SignalLab/main/deploy/install.sh | sh
  ```

  ```bash
  docker run -d --name signallab --network host --restart unless-stopped \
    -v signallab-data:/data --read-only --cap-drop ALL --security-opt no-new-privileges \
    ghcr.io/proanima/signallab:latest
  docker logs signallab    # the access token, on the first start
  ```

- **Command line** — `signallab` comes with the Windows installers, the `.deb` and the `.rpm`
  (not the AppImage) and is in the server image; archives for CI are on the release page.

The desktop app looks for signed releases once a day and installs one when you say so. Details:
[installing and updating](https://proanima.github.io/SignalLab/guide/install.html),
[running a server](https://proanima.github.io/SignalLab/server/).

## Documentation

The documentation is [online](https://proanima.github.io/SignalLab/) and, from the
release after 1.0.0, inside the app and the server, so it works offline: press
<kbd>F1</kbd> on any screen.

| | |
| --- | --- |
| [Getting started](https://proanima.github.io/SignalLab/guide/) | what it is, installing, the window, a first session, the ideas |
| [Protocols](https://proanima.github.io/SignalLab/protocols/osc.html) | OSC, UDP and TCP, HTTP, WebSocket, MQTT, broadcast and discovery |
| [Tools](https://proanima.github.io/SignalLab/tools/signals.html) | signals, the Inspector, emulators, impairment, Storm, Scanner |
| [Experiments](https://proanima.github.io/SignalLab/experiments/) | the editor, every node, data and templates, flow, load, faults, runs |
| [Automation](https://proanima.github.io/SignalLab/automation/cli.html) | the command line, CI, MCP |
| [Server](https://proanima.github.io/SignalLab/server/) · [HTTP API](https://proanima.github.io/SignalLab/api/) | running it, its security, every command and event |
| [Reference](https://proanima.github.io/SignalLab/reference/shortcuts.html) | shortcuts, files, troubleshooting, every error message |

## Build from source

```bash
npm install
npm run tauri dev        # the desktop app with live reload
npm run check            # the checks CI runs; CI also tours every screen and builds the image
```

You need Node 22.18 or newer (CI uses 24), Rust as pinned in `rust-toolchain.toml`
and, on Windows, the MSVC build tools and WebView2 (part of Windows 10 and 11); on
Linux, WebKitGTK 4.1 and the libraries Tauri needs. See
[building and running](https://proanima.github.io/SignalLab/develop/building.html)
and the [architecture](https://proanima.github.io/SignalLab/develop/).

## Responsible use

Storm, Scanner and Broadcast send real traffic to real hosts, and a broadcast or a
sweep reaches every device on the segment. Point them only at equipment you own or
are authorised to test. Every default unicast target is on loopback — Broadcast's
broadcast, multicast and sweep modes start on the local segment
(`255.255.255.255`, `239.1.1.1`, `192.168.1.0/24`); the engine's guard rails — a
sweep of at most 1024 hosts, a beacon of at most 50 000 packets a second — are
guard rails, not permission.

## Contributing

Issues, ideas, translations and pull requests are welcome — see
[CONTRIBUTING.md](CONTRIBUTING.md). Report security problems privately as
[SECURITY.md](SECURITY.md) describes. Everyone follows the
[code of conduct](CODE_OF_CONDUCT.md).

## License

[MIT](LICENSE) © 2026 [ProAnimaStudio](https://github.com/ProAnima) —
[info@proanima.net](mailto:info@proanima.net).
