# Changelog

Notable changes to Signal Lab. Versions follow [semantic versioning](https://semver.org);
dates are release dates.

Write changes under **Unreleased** as they land. `npm run release -- <version>` turns
that section into the version's section, and the release workflow uses it as the
release notes — so what is written here is what users read. See
[docs/delivery.md](docs/delivery.md).

## [Unreleased]

### Added

- **Signal Lab in CI/CD.** `signallab`, a command line next to the installers and in
  the image: `signallab run tests/*.json --junit junit.xml` runs experiments headless
  and exits 0 when all pass, 1 when one fails, 2 for invalid input, 3 when it could not
  run; steps print as they happen, in English or Russian, and `--json` gives one JSON
  object per line. `--server` sends the runs to a lab server that can reach the gear.
  `signallab send osc|udp|http|mqtt` and `signallab fire` send one message the way the
  app does. Secrets come from `SIGNALLAB_SECRET_*` and are never printed.
- **An API for automation.** `POST /api/run` on the server runs an experiment to its end
  and answers with the result, or streams its steps as NDJSON; the whole API is
  described at `/api/openapi.json`.
- **A GitHub Action** (`uses: ProAnima/SignalLab@vX.Y.Z`) runs experiments in a workflow
  and leaves a JUnit report; GitLab CI and plain shell recipes are in
  `docs/automation.md`.
- **The server in one command.** On a Linux machine,
  `curl -fsSL https://raw.githubusercontent.com/ProAnima/SignalLab/main/deploy/install.sh | sh`
  installs Docker if needed (it asks), starts the server with host networking, waits
  until it answers and prints the address and the access token. Running it again
  updates; `--uninstall` removes it and keeps the data, `--purge` deletes that too.
- The server image makes its own access token on the first start (`/data/token`,
  shown once in the log and kept across updates), so a bare `docker run` or
  `docker compose up -d` is enough; `--generate-token` does the same outside Docker.
  On loopback, where no token is needed, nothing is made.

- **Save from where you send.** The HTTP, OSC and MQTT screens have *Save…*:
  name it, pick a folder, and what you just sent is in the library. The screen
  stays tied to it — *Save* (`Ctrl+S`) updates it, *Save as…* makes a copy, and a
  chip says where it lives and whether it has changed; a click on the chip shows
  it in Signals.
- **Folders in the signal library.** Nested folders that open and close (and
  stay that way), made, renamed (`F2`), moved and removed — what was in a removed
  folder moves up a level; drag a signal or a folder onto another folder.
  *Open in HTTP / OSC / MQTT* loads a signal back into its screen. Library files
  are now version 2 and keep empty folders; older ones open as before.
- **Ready for more languages.** Counts agree with their number in every
  language (“1 signal”, “5 сигналов”), numbers and sizes are written the
  language's way, the language is picked from the system's list of preferred
  ones, and the starter signals are created in it. Adding a language is a
  dictionary and one line — see `docs/localization.md`; the checks name every
  missing text, placeholder or plural form.
- **Every error in your language.** The tool screens — OSC, Broadcast,
  MQTT, impairment, Storm, Scanner, HTTP, the library and the Inspector — now
  report failures the way experiments do: what happened and what to do, in
  English or Russian, with the system's own text folded underneath. A refusing
  MQTT broker says why (protocol, client id, unavailable, credentials, not
  authorized), and jobs in the console strip are named in your language.

- **Repeat.** Any step that sends can send again and again — a number of times
  or for a time, every so many milliseconds with an optional seeded jitter — for
  heartbeats and polling. `{{counter}}` numbers the sends, Retry applies to each,
  a send that waits for a reply waits for its own, and the timeline shows the
  progress once a second.
- **Loop.** A bounded loop node: the steps on *Body* run and come back, up to a
  maximum, until an exit condition holds (checked after each iteration, so the
  body can set what it tests); *Done* follows, or *Limit* when the iterations ran
  out first. The wire back is drawn over the body. New template *Poll until
  ready*. Experiment files are now version 5; older ones open as before.
- **Every screen checked end to end.** `npm run e2e` walks every tab of the real
  app — the desktop app and the server in a browser, on Windows (WebView2, Edge)
  and Linux (WebKitGTK) — sending real traffic to loopback stand-ins and checking
  what arrived; CI does it on every change, also for the published image.
- **Wait for MQTT.** Wait for a message on a topic filter (`+`, `#`) whose payload
  matches; the run subscribes before its first step, so a quick answer is not
  missed, and retained values replayed on subscribing are ignored.
- **Wait for this** on a message in the OSC monitor or a topic in the MQTT tree
  builds the wait for it in the experiment; and a wait that matched links from the
  timeline to its frame in the Inspector.
- **A reply in the same node, and Retry.** An OSC message or a UDP datagram can
  wait for its answer — sent from the port it listens on, so a device that
  answers the sender is heard too — and the reply is a variable for later steps.
  Every step that sends or listens can retry, with the same or a doubling pause;
  each failed attempt and its reason are in the timeline, and Stop ends a pause.
  Experiment files are now version 4; older ones open as before.
- **Parallel work from any output.** Drag several wires out of one output —
  Start included — and their nodes run at the same time, each branch with its
  own copy of the variables. A Join waits for every wire into it; End completes
  the run once, after the last branch, and never if a branch failed.
- **Experiments that use data.** Parameters with defaults and target profiles, a
  `{{template}}` language with seeded generators, *Extract* from a response,
  *Check value* and *Branch on value*, a resolved preview of every field, and
  *Run with…* for one-run overrides and a seed.
- **Secrets** from the Windows Credential Manager (`{{secret.NAME}}`), masked in the
  timeline, previews, responses, reports and the Inspector.
- **Wait for OSC** (OSC 1.0 address patterns, argument rules) and **Wait for UDP**
  (any, contains, regex, hex) with *Matched* and an optional *Timeout* output.
  Listeners open when the run starts, so a fast reply is not missed; the reply is a
  variable (`{{reply.args[0]}}`). *Listen now* runs one wait on its own. New template
  *OSC ping → reply*.
- **Errors in your language.** The experiment engine reports a code, values, the node
  and field, and the system's own text; the interface shows *Node · Field — message*
  with the technical detail folded underneath, in English or Russian. Network
  failures are told apart: refused, timed out, name not found, unreachable, port in
  use, TLS.
- **Server mode.** `signal-lab-server` runs the engine without a window and serves the
  full interface to a browser, with the same commands over HTTP and live updates over a
  WebSocket. It listens on this machine only unless given an access token (sign-in page
  in English or Russian, session cookie); experiment secrets come from read-only files;
  reports and exports download from the browser.
- **Docker image** `ghcr.io/proanima/signallab` for x64 and arm64: unprivileged, with a
  health check and a `/data` volume, plus `deploy/compose.yaml`. Broadcast, multicast
  and discovery reach the network with `--network host` on a Linux host.
- **Linux builds** (`.deb`, `.rpm`, AppImage) next to the Windows installers.
- **CI** on every push and pull request (Windows and Linux), a release workflow that
  builds both platforms from a tag into a draft release with checksums, and the
  local commands `npm run check`, `npm run build:linux` and `npm run release`.

### Changed

- **Installers that look like Signal Lab.** The Windows setup and the MSI carry the
  brand on their pages, the setup has its own icon, and the license page is gone (MIT
  asks for no acceptance) — one click less. Unattended installs are documented
  (`/S`, `/ALLUSERS`, `/P`, `msiexec /qn`).
- **The Inspector lives in the bottom panel**, a tab next to the console, so it
  is at hand on every screen; the tab shows when capture is on and how many
  frames it holds, and the panel can be maximised. *Open frame* links go there.
- **Help where it was missing.** Tooltips, in your language, on the fields whose
  meaning is not obvious — bind addresses, TTL, keepalive, client id, QoS,
  concurrency, timeouts, banners, every impairment slider — on every icon button,
  and on the Inspector's buttons and filters. Keyboard focus on a field shows
  its label's help.
- The server's sign-in page picks the most preferred of the browser's languages
  that it has, not only the first.
- **Wires can be picked and removed.** A click on a wire selects it (the
  properties say what it connects) and `Delete` removes it; hovering a wire shows
  a × above its ＋ that removes it at once. Undo brings it back.
- **Panes you can size.** Drag the top edge of the console, the left edge of the
  properties pane or the top of the run timeline — or focus the handle and use the
  arrow keys; a double click restores the default. Sizes and whether the console is
  open are remembered.
- Ports are easier to hit: the area that takes the pointer is several times the
  dot and grows when the canvas is zoomed out, and a wire released next to a
  node connects to it. Dragging a wire always adds one; *A*, *Add next* and the
  ＋ on a wire still insert into the flow.
- **A lighter interface.** Explanatory captions are gone from every screen and the
  sign-in page; what a control does, its shortcut and what `0` means are tooltips
  in your language, on hover and on keyboard focus, also over dialogs. Labels say
  only a name and a unit, empty states a few words.
- Switching screens keeps everything: typed values, the last response, running
  monitors and the scroll position.
- The version is written once, in `package.json`; the app, the installers and the
  engine read it from there.
- The engine is its own crate, independent of the desktop shell; the desktop app and
  the server share one command table, so they behave the same.

### Fixed

- A failed UDP signal reported its payload instead of the reason; a failed HTTP
  signal showed the HTTP client's raw English; an MQTT broker given as an IPv6
  address was dialled without brackets. The starter signals' notes carried long
  runs of spaces.
- Counts read wrongly for one (“1 signals”, “Joined 1 branches”) and for most
  numbers in Russian (“2 шагов”); units (ms, Mbps, KB) and the run timeline's
  clock were not in the interface language; captured signals went into an
  English “Captured” folder.
- On Windows, a discovery listener that answers probes stopped as soon as a prober
  closed its port, and the impairment relay stopped forwarding — while still
  looking alive — after its target or client went away for a moment. Both carry
  on now, and a relay that really cannot receive any more ends with the reason.
- The Inspector could list the same frames many times over when several sources
  captured at once (a port scan), leaving rows that no filter removed.
- Every field is tied to its label: screen readers name it, and a click on the
  label puts the cursor in it. Argument rows and icon buttons have names too.
- A stray "0" next to the Inspector's filters, an accent bar on every cell of the
  selected frame instead of one, a stretched QoS badge in MQTT subscriptions, and
  a white square where two scrollbars meet.
- The HTTP screen's default URL pointed at an outside service; it is loopback now,
  like every default target.
- Downloading an export from the server never navigates away from the interface,
  even when the server answers with an error.
- `package-lock.json` carried version 0.1.0 instead of the app's version.

## [0.3.1] - 2026-08-27

### Fixed

- The 0.3.0 installers were built from the commit before the cancellation fixes; 0.3.1
  ships them. A scan no longer reports an empty table, and Stop stops the work it
  started (storm, impairment relay, broadcast, HTTP burst, scanner).

## [0.3.0] - 2026-08-26

### Added

- **Signal library**: named OSC, UDP, HTTP and MQTT packets, grouped and searchable,
  fired from any screen with `Ctrl+K`; any Inspector frame can be saved as a signal.
- **MQTT 3.1.1 client**: one live connection as a job, a topic tree, QoS 0/1/2,
  retained values and last-will.

### Fixed

- Job telemetry was dropped before the job id was known, so fast jobs looked empty;
  Stop aborted only the supervising task and left the work running.

## [0.2.0] - 2026-08-26

### Added

- The app's own icon, and NSIS and MSI installers in English and Russian.
- Broadcast, multicast and CIDR sweep with a discovery responder, the packet Inspector,
  a bilingual interface, and the MIT license.

### Fixed

- Storm and the broadcast beacon reported a count up to 250 ms old at the end of a run.
- Checkbox, row and slider alignment.

[Unreleased]: https://github.com/ProAnima/SignalLab/compare/v0.3.1...HEAD
[0.3.1]: https://github.com/ProAnima/SignalLab/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/ProAnima/SignalLab/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/ProAnima/SignalLab/releases/tag/v0.2.0
