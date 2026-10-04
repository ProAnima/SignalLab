# Changelog

Notable changes to Signal Lab. Versions follow [semantic versioning](https://semver.org);
dates are release dates.

Write changes under **Unreleased** as they land. `npm run release -- <version>` turns
that section into the version's section, and the release workflow uses it as the
release notes — so what is written here is what users read. See
[docs/delivery.md](docs/delivery.md).

## [Unreleased]

## [1.0.0] - 2026-10-04

### Added

- **Nine more languages.** The interface, its tooltips and errors, the command
  line, the server's sign-in page and the installer now speak Spanish, French,
  German, Portuguese (Brazilian), Chinese (Simplified), Japanese, Korean, Hindi
  and Arabic besides English and Russian. Arabic mirrors the page right to left;
  addresses, hex dumps, code and the experiment canvas stay left to right.
- **A language switch with flags.** The header shows the current language's flag
  and letters and opens the list of every language, each by its flag and its own
  name; the arrows, a letter, Enter and Escape work in it.

- **TCP impairment.** The Impairment screen and the experiment's Impairment node
  take a protocol: over TCP each connection to the relay is joined to one of its
  own to the target, and both streams are delayed (in order, whatever the
  jitter), held to a bandwidth limit (the sender slows down; nothing is dropped),
  reset (both sides get a reset) or left half-open (nothing more goes through,
  and nobody is told), or paused while offline — every decision drawn from the
  seed. The presets have TCP versions; the screen counts connections, resets,
  half-open connections and how often a stream was held back; a run's report
  counts them per phase, and *Change impairment* switches a TCP relay too.

- **Load on an HTTP node.** An experiment's HTTP request can run under a
  profile — constant, ramp, steps, spike or random (Poisson) arrivals, up to
  100 000 requests a second and 512 at once — drawn in the properties before it
  runs. The step measures p50/p90/p95/p99, mean and slowest, the error rate, the
  rate achieved, answers by status or cause, each second and a latency histogram,
  and passes only when its **thresholds** hold (`p95 < 300 ms`, `errors < 1 %`):
  the first that does not fails it and says by how much. The timeline counts up
  every second; the properties show the result. Experiment files are version 9.
- **Compare two runs.** *Compare* in the timeline sets this run beside the one
  before it, or any earlier run of the experiment: each load step's metrics before
  and after, the change, a regression in red, and how each threshold went.
  `signallab run` prints each threshold's verdict and fails on one that does not
  hold (JUnit `type="load.threshold"`); MCP has `list_runs` and `compare_runs`;
  the API `experiment_runs` and `experiment_compare`. Run reports are version 5.

- **The Inspector keeps whole frames.** A frame keeps its bytes up to 256 KiB
  instead of the first KiB: the list still shows a preview, *Show all* brings every
  row, *Save as signal* replays a datagram of any size byte for byte (it refused
  anything over 1 KiB before), and both exports write every byte kept (`.jsonl` as
  base64 `data`). The capture holds up to 8192 frames and 64 MiB; a frame larger
  than it keeps says so.

- **Several nodes at once in the experiment editor.** `Shift` or `Ctrl` and a click
  adds a node to the selection, `Shift` and a drag on the empty canvas selects with a
  frame, `Ctrl+A` selects everything. The selection moves, duplicates (`Ctrl+D`) and
  deletes together, each one step to undo, and `Ctrl+C`, `Ctrl+X`, `Ctrl+V` copy, cut
  and paste it with the wires between its nodes — into another experiment or window
  too. A pasted node that names another copied node names the copy; a copied Emulator
  or Impairment listens on the next free port.

- **Updates.** The desktop app finds a newer version among the releases published on
  GitHub — once a day, or *Check for updates* in About — says what is new in it and
  installs it when asked: what is pending is saved, running jobs stop, the download's
  signature is checked against the key built into the app, and Signal Lab starts again
  as the new version (Windows: the setup or the MSI; Linux: the AppImage, `.deb` or
  `.rpm`). It asks the studio's hub, which offers a release to a share of installs
  first and can hold it back, and GitHub when the hub cannot be reached; the check
  tells the hub the version, the system and a random number of this install. Drafts,
  pre-releases and commits are never offered. A server updates with its image.
- **About.** Who makes Signal Lab — ProAnimaStudio, Ian Panaev — and how to reach them
  (info@proanima.net), the version, the source and the license; the **?** in the header.
- **Write to the developers.** A form (the **✉** in the header, and in About): a
  message, an address for the answer, screenshots pasted with Ctrl+V, picked or dropped,
  and the console log and the version and system attached on their own — shown before
  sending, each removable, with this computer's name, its address and your folders left
  out. It goes to the studio's hub, which mails it from signal-labs@proanima.net to
  info@proanima.net; the mailbox's password is on the hub and never in the app — see
  `docs/hub.md`.

- **Emulators: Signal Lab as the other side.** A new *Emulators* screen plays the API,
  device or service your system talks to. An **HTTP API** answers by routes — method,
  a path with `:name` segments, conditions on headers, query, body or JSON — with
  responses in sequence (500, 500, then 200, for retries), in turn, or a seeded mix by
  weight, with delays, jitter and faults (no answer, a closed connection). An **OSC**,
  **UDP** or **TCP device** answers by rules: on this address, payload or line, reply
  that, to the sender or elsewhere, after a delay; a TCP device greets and can hang up.
  Replies are templates read with what arrived (`{{request.params.id}}`,
  `{{request.args[0]}}`, `{{request.json.name}}`). Every exchange is counted per rule,
  listed as it happens and goes to the Inspector. A starter set (an API, an OSC, a UDP
  and a TCP device) is in the library, on loopback. **Mock this** on the HTTP screen turns
  a response into a route that answers exactly so.
- **Emulators in experiments.** The *Emulator* node serves for the whole run — open
  before the first step, closed with the run — and the report counts what it received.
  *Wait for HTTP request* waits for a request to it (or to a listener of the run that
  answers 204) by method, path and conditions, so an experiment proves that the system
  under test called it, and with what. An OSC or UDP emulator shares its port with the
  run's waits. The template *Retry a flaky API* shows it end to end (document version 6).
- **Emulators everywhere.** `signallab emulate <file|name>` runs emulators from a file or
  the app's library until Ctrl+C or `--for`, printing every request and the counts at the
  end (`--json` for one object per line, `--server` to run them on a lab server);
  `signallab emulators` lists the library. An assistant gets `start_emulator`,
  `emulator_exchanges` and `list_emulators` over MCP. The server's API has the same
  commands (`emulator_start`, `emulator_exchanges`, `emulators_load`, …).
- **An MQTT broker of its own.** An emulator can be an MQTT 3.1.1 broker: clients
  connect (with a user name and password, when it asks for one), subscribe with `+` and
  `#`, publish at QoS 0, 1 and 2; retained messages, last wills and a client id taking
  over its old connection work as on any broker. Rules make it a device too: on a message
  to `lab/+/set`, publish `lab/{{request.levels[1]}}/state`. MQTT gear and experiments
  with *Wait for MQTT* need no broker installed any more; a demo broker is in the library.
- **Dependencies that flap and break.** Any emulator can go down now and then — up for
  so long, down for so long, from the start: HTTP answers 503 with `Retry-After` (or
  closes, or holds the request), TCP devices and the broker drop and refuse connections,
  OSC and UDP devices go silent; what met an outage is counted apart. A response can be
  *malformed*: complete HTTP whose JSON stops halfway.
- **Faults on a schedule.** Three new nodes break things on cue. **Impairment** puts a
  UDP impairment relay in front of a device for the whole run (*Route through
  impairment* on an OSC or UDP node inserts one and points the node at it); **Change
  impairment** switches it to another profile mid-run; **Emulator down/up** takes one of
  the run's emulators down and brings it back. The relay's decisions come from the run's
  seed — the same seed drops the same packets — each phase is counted in the report, and
  a run that ends in any way closes its relays. Templates *Fault phases* and *Dependency
  outage* show both (document version 7; run reports version 4).
- **More ways a network fails.** The impairment relay adds bursts of loss, reordering, a
  bandwidth limit and *offline*, has presets (*LAN*, *Busy Wi-Fi*, *4G*, *Satellite*,
  *Intermittent*, *Offline*), takes edits while it runs without dropping its port, and
  counts what it received, throttled and reordered. A running emulator can be taken down
  and brought up by hand on the Emulators screen, over the API (`emulator_down`) and by an
  assistant (`set_emulator_down`).
- **WebSocket.** A new *WebSocket* screen connects to `ws://` and `wss://` services with
  the headers and subprotocols they expect, sends text or bytes (`Ctrl+Enter`) and lists
  every message as it comes, JSON formatted, with the close handshake and who closed.
  Experiments get **WebSocket connect**, **WebSocket send**, **Wait for WebSocket** (any,
  contains, regex, hex; the message as `{{reply.text}}`, `{{reply.json.field}}`) and
  **WebSocket close**; a token extracted earlier can be in the URL or a header, and a run
  that ends in any way closes its connections with a proper close frame. *Send now* on a
  send or a wait opens the connection its connect node describes. The template *WebSocket
  echo*; `signallab send ws <url> --text … --expect …` and the assistant's `send_ws`;
  the API's `ws_connect`, `ws_send`, `ws_close`, `ws_exchange`. Experiment files are now
  version 8 — this release's last; files of every earlier version open as before.
- **A parameter matrix for the command line.** `signallab run` and `validate` take
  `--matrix NAME=V1,V2` (repeat for more names: every combination runs) and
  `--matrix-file` (axes, or a list of combinations), and `--fail-fast` stops at the
  first run that does not pass. Each combination is a run of its own — named with its
  values, a JUnit suite with `param.NAME` properties, a report of its own, a `matrix`
  object next to the `file` it came from in `--json` — all checked before the first
  sends anything, here or on a server. A value given twice runs once, at most 256 runs
  start from one command, and a run `--fail-fast` never started is a skipped suite of
  the JUnit report. The GitHub Action has `matrix`, `matrix-file` and `fail-fast` inputs.
- **HTTP authentication and cookies.** A request — on the HTTP screen, in an experiment's
  HTTP node, in a burst, from `signallab send http` (`-u name:password`, `--digest`,
  `--bearer`) and the assistant's `send_http` — authenticates with **Basic**, **Bearer**
  or **Digest**: the server's 401 challenge is answered (RFC 7616: MD5 and SHA-256, their
  `-sess` variants, `qop=auth` and `auth-int`) and the request sent again; a burst
  answers one challenge for all its requests — in parallel too, never sending a count
  twice — and a stale nonce once more. Behind a redirect the URL that asks is the one
  answered; a challenge from another origin is not. The credentials go only into the
  request: nothing in the Inspector, a step or a report shows them, a node's password
  can be `{{secret.NAME}}`, and a secret is masked in Basic's base64 too. A request
  saved as a signal keeps its credentials — the Signals screen shows and edits them, and
  the HTTP screen takes them back from it after a restart. A **cookie jar**
  sends back what servers set with Set-Cookie, as a browser does (domain, path, Secure,
  expiry): the HTTP screen keeps one, listed and cleared there (*Keep cookies*; the
  library's HTTP signals use it too, and on a server every page shares it), and every
  run keeps its own (*Keep cookies between requests* in the Parameters panel). Files
  from before version 8 open with the run's jar off, so they run as they did.
- **Load at a rate, read in percentiles.** The HTTP screen's burst can start requests on
  a schedule of its own — *Rate, req/s*, 0.1 to 100 000 — instead of each worker sending
  again on its answer. A request that finds every worker busy is skipped and counted as
  *Missed* rather than sent late, so a server that cannot keep up shows it instead of
  being given a breather. Every burst reports p50, p90, p95 and p99 next to the average
  and min/max (read to within half a percent, in constant memory however long it runs),
  in a row of their own; a short latency reads with two decimals instead of "0 ms", one
  from a second on in seconds, and the last report rates the whole burst.
  `http_burst_start` takes `rate`.
- Every HTTP route has **Copy the URL**, and **Mock this** is offered on a *Send now*
  response in the experiment editor as well, a URL template becoming a path pattern
  (`{{api}}/orders/{{id}}` → `/orders/:id`). A problem in an emulator says which rule
  and which response it is in.
- **For an assistant (LLM).** `signallab mcp` serves Signal Lab over the Model Context
  Protocol: an assistant in Claude Code, Claude Desktop, Cursor or VS Code can learn what
  an experiment is made of, write one, validate it, run it and read step by step why it
  failed, send one OSC message, datagram, HTTP request or MQTT publish, listen on a port,
  and fire library signals — here, or on a lab server with `--server`.
  `signallab mcp --print-config <client>` prints the configuration to paste.
- **`signallab` comes with the app.** The Windows setup and the MSI install it next to the
  app and put it on `PATH`; the `.deb` and `.rpm` put it in `/usr/bin`. `signallab nodes`
  lists every kind of node with its fields and an example.
- **The firewall, handled.** The Windows setup for everyone allows Signal Lab and
  `signallab` through Windows Firewall on private and domain networks (and removes the
  rules on uninstall; `/NOFIREWALL` skips it). Otherwise the app, the first time it listens,
  says when the firewall is in the way — a *Cancel* at the system's prompt, or a public
  network — and offers **Allow** with the system's administrator prompt.
  `signallab doctor` shows the firewall, the network, the data folder and a server's token;
  `signallab firewall allow` fixes the firewall from a terminal. The server's install script
  opens its port in ufw or firewalld when asked (`--open-udp` for monitor ports) and closes it
  again on `--uninstall`.
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
  ready* (document version 5).
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
  (Document version 4.)
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

- The Impairment sliders' values sat against their labels (“Latency40 ms”) instead
  of at the far end of the row.
- The Inspector had no MQTT filter chip, although it captures MQTT.
- A job that ended before it was registered — a burst of one against a refused port —
  stayed in the console strip's list for good.
- A check of the last HTTP response right after a Loop (*HTTP status* on Done, an
  *Extract*) was refused as having no request before it, although the Loop's body, which
  always runs, made one.
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

[Unreleased]: https://github.com/ProAnima/SignalLab/compare/v1.0.0...HEAD
[1.0.0]: https://github.com/ProAnima/SignalLab/compare/v0.3.1...v1.0.0
[0.3.1]: https://github.com/ProAnima/SignalLab/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/ProAnima/SignalLab/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/ProAnima/SignalLab/releases/tag/v0.2.0
