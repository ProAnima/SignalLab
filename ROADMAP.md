# SignalLab roadmap: network behaviour laboratory

Status: 2026-10-02. This document fixes product direction, delivery order and the design of each step. It is not a release schedule; sizes are relative (S, M, L).

## Positioning

Postman tests an API. k6 tests load. **SignalLab tests how a networked system behaves** — devices, controllers, brokers and services talking OSC, UDP, MQTT, HTTP and TCP at once — when it runs normally, under load, and when the network or a dependency breaks. Everything is local-first: one window, one timeline, one experiment file.

This decides what we build and what we leave to others:

- **Build:** multi-protocol flows, waiting for real replies from devices, mock dependencies, fault injection, record/replay of real sessions, virtual devices, one timeline for all of it.
- **Do not rebuild:** Postman-style collection management, a scripting language inside nodes, million-request load generation (k6 does that), IP spoofing, a pentest suite.

## Principles every milestone follows

1. **The direct instrument stays one click away.** Every new capability first works by hand (a screen or a node's *Send now*), then becomes a node. Nothing exists only inside the graph.
2. **One vocabulary.** Targets, templates, match rules and fault profiles are defined once and reused by nodes, direct screens, mocks, replays and devices.
3. **Creation starts from something that worked.** A request that answered, a response field, a captured packet, a recorded session — each has a one-click path into the experiment ("Add to experiment", "Extract", "Mock this", "Replay this").
4. **Bounded and stoppable.** Every wait has a timeout, every repeat a limit, every generator a rate cap, every listener an owner job. Stop ends all of it.
5. **Deterministic when asked.** One experiment seed drives every random choice (generators, faults, mutations, distributions). The same seed reproduces the same run.
6. **Honest reports.** A run records resolved values (secrets masked), which event satisfied each wait, which faults fired, and the exact bytes where they are available.
7. **Safe defaults.** Loopback targets, loopback mock binds, explicit targets and previews for anything that reaches a segment. Guard rails are not permission.

## Where we are

### Delivered

- **Direct instruments:** OSC sender/monitor/generator, MQTT client, broadcast/multicast/sweep and discovery responder, HTTP request and concurrent burst (at a rate, with percentiles), UDP impairment relay, UDP/TCP storm, TCP scanner, Inspector capture bus, Signals library with `Ctrl+K`.
- **Experiment editor:** versioned JSON documents, templates, import/export, grouped undo/redo, autosave, search, arrange/fit, focus and fullscreen modes, 900×600 layout.
- **Node creation:** `A` adds after the selected node and focuses its main field; drag a wire from an output onto a node to connect or onto empty canvas to create the next node — an output may have several wires, which run in parallel; the ＋ on a wire inserts into that wire; saved Signals appear in the add menu as prefilled nodes; OSC and HTTP screens have *Add to experiment*.
- **Nodes:** Start/End, HTTP, OSC, UDP, TCP, MQTT publish, Log, Delay, HTTP status/body/header/latency checks, status branch, parallel branch and join.
- **Feedback:** per-node *Send now* (`Ctrl+Enter`) with formatted response, canvas markers for unwired outputs and unreachable nodes, validation that names and reveals the node, timeline with step highlighting, JSON run reports.
- **Runner:** one job per run; Stop cancels every branch; a Join that never fills fails at that Join.
- **Data and replies** (milestones 3 and 4.1): parameters, profiles, secrets, `{{templates}}`, Extract / Check value / Branch on value, Wait for OSC and Wait for UDP with Matched and Timeout outputs.
- **Server mode** (delivery D1–D4): the same engine and interface in a browser, and a Docker image.
- **Emulators** (milestone 5): HTTP APIs, OSC/UDP/TCP devices and an MQTT broker with templated replies, sequences, faults and outages — on their own screen, as an experiment node with *Wait for HTTP request*, from `signallab emulate` and over MCP.

### Known gaps in what exists

- The impairment relay is UDP only; impairing a TCP stream (latency, throttling, reset, half-open) is still to come.
- The HTTP burst runs at one fixed rate or as fast as its workers go; ramps, steps and spikes, thresholds and an HTTP node under load are milestone 7.
- The Inspector ring truncates payloads at 1 KiB and holds 8192 frames: good for looking, not for replay.
- Remaining editor work: multi-selection and copy/paste.

## Delivery order

| # | Milestone | Why here | Size |
| --- | --- | --- | --- |
| 3 | Data: parameters, templates, extraction, generic checks | Every later feature needs values to flow | M |
| 4 | Reactive flows: wait for replies, retry, repeat | Devices answer; tests must listen | M |
| 5 | Mock services | The biggest missing side: SignalLab as the dependency | M |
| 6 | Faults as nodes and phases | Our strongest differentiator, already half built | M |
| 7 | Load profiles, thresholds, run comparison | Turns runs into measurements | M |
| 8 | Record → replay → mutate | The killer feature for show and installation work | L |
| 9 | Virtual devices | Composes 3–8 into populations | L |
| 10 | More transports, bridge, headless runner | On demand, on the same model | S–L each |

Milestones 3 and 4 are a hard dependency for everything after them. Between 5 and 8 the order can change if a real project needs one sooner.

---

## 3. Data: parameters, templates, extraction, generic checks

**Goal.** Values move through a run: log in, take the token, use it in the next request; send a command with a generated id and check the reply carries it.

Detailed design: [docs/milestone-3-data.md](docs/milestone-3-data.md). **Status:** PR 3.1 delivered — template language and seeded generators, document v2 with parameters and seed, Extract / Check value / Branch on value, validation of names per path, resolved preview, `{{` suggestions, *Send now* with templates and extraction, Extract as variable from a response, seed pinning. PR 3.2 delivered — document v3 with target profiles, a profile switch in the toolbar that marks profiles that would not run, Run with… for one-run overrides and seed, drafts that always save. PR 3.3 delivered — `{{secret.NAME}}` from the Windows Credential Manager, a Secrets section in the Parameters panel, *Send now* executed by the engine so values never reach the interface, masking in the timeline, previews, responses, reports and the Inspector, and runs refused before any traffic when a referenced secret is missing. Milestone 3 is complete.

**What the user gets**

- `{{…}}` works in every text field of every action node: URL, headers, body, OSC address and string arguments, MQTT topic and payload, UDP/TCP text.
- Built-in generators: `{{uuid}}`, `{{now}}` (ms), `{{now.iso}}`, `{{counter}}`, `{{random_int(1,100)}}`, `{{random_float(0,1)}}`, `{{pick(a,b,c)}}`. All seeded.
- **Parameters** of the experiment (name → default), edited in a toolbar panel. **Run with…** overrides them for one run; the report records the values used.
- **Target profiles** (e.g. *Local*, *Stage*, *Venue*) as sets of parameters, switched from the toolbar. Templates keep pointing at `{{params.api}}`; switching the profile retargets the whole experiment.
- **Secrets** as `{{secret.NAME}}`, stored outside the experiment and masked in reports and the timeline.
- **Extract** node: from the last HTTP response (status, header, JSON path such as `$.token` or `$.items[0].id`, regex on the body) into a named variable.
- **Check value** node: `{{var}}` compared with a value (`=`, `≠`, `<`, `>`, contains, matches regex). **Branch on value** is the same with Yes/No outputs. The status-specific nodes stay as shortcuts.

**Convenient paths**

- Typing `{{` in any field opens autocomplete: parameters, variables extracted upstream on the path, generators. Unknown names are underlined before the run.
- In a *Send now* response, clicking a JSON value offers **Extract as variable**: it inserts an Extract node after the request with the path and a suggested name filled in.
- The properties panel shows the **resolved preview** of the selected node with current parameters and the last known variable values.
- Hovering a timeline step shows the variables it read and wrote.

**How (engine)**

- New `engine/template.rs`: parse once into literal/expression parts; resolve against a context `{ params, vars, secrets, generators(seed) }`. Unknown names fail the step with the name, never silently expand to empty. Unit-tested without a runtime.
- `BranchContext` gains `vars: BTreeMap<String, Value>`. A fork copies it; a Join merges branch 1 then branch 2 (later write wins) so results are deterministic.
- Document **version 2** adds `params`, `profiles`, `seed`. `experiment_files::parse` migrates v1 to v2 in memory; saving writes v2. Templates and tests cover the migration.
- JSON paths are a small subset (`$.a.b`, `[n]`, `["key"]`) converted to `serde_json` pointers — no new dependency.
- Secrets: Windows Credential Manager through the `keyring` crate; the experiment stores only names.
- Validation checks that every `{{var}}` is produced on every path before it is used, the same way status checks require an earlier HTTP node today.

**Done when** the login → extract token → authorized request → check field flow can be built from templates, run against two profiles, and its report shows the resolved values with the secret masked.

## 4. Reactive flows: wait for replies, retry, repeat

**Goal.** Send a command to a device, wait for its answer, use it. Retry what flakes. Repeat what pulses.

Detailed design: [docs/milestone-4-reactive.md](docs/milestone-4-reactive.md). **Status:** PR 4.1 delivered — **Wait for OSC** (OSC 1.0 address patterns, argument rules) and **Wait for UDP** (any, contains, regex, hex) with **Matched** and an optional **Timeout** output; listeners armed when the run starts (a taken port stops the run before any traffic, at the wait that needs it); the reply as a variable on the Matched path only (`{{reply.args[0]}}`, `{{reply.from}}`, `{{reply.ms}}`); *Listen now* on a wait; the bundled template *OSC ping → reply*. PR 4.2 delivered — **wait for a reply** on OSC and UDP nodes (sent from the listening port, so a device answering the sender is heard), **Retry** on every action and wait (attempts, fixed or doubling pauses, each failed attempt on the timeline, Stop ends a pause), **Wait for MQTT** on a subscription opened before the first step (retained replays ignored), **Wait for this** from the OSC monitor and the MQTT topic tree, and links from the timeline to the Inspector frame a wait matched; document version 4. PR 4.3 delivered — **Repeat** on every action (count or duration, interval, seeded jitter) and the bounded **Loop** node (Body, Done, optional Limit, an exit condition read after each iteration); document version 5. Milestone 4 is complete. The same PR replaced the engine's English error strings with structured, localized errors (code, values, node, field, technical detail) everywhere the experiment engine reports a problem, classified network failures (refused, timeout, name not found, unreachable, port in use …), split the engine along responsibilities, and made every screen keep what was typed when switching tabs. Alongside it: *Save…* on the sending screens and nested library folders, the Inspector as a tab of the bottom panel, the tool screens' errors and job names as localized codes too, and plural forms, number formats and a language list that make another language a dictionary and one line ([docs/localization.md](docs/localization.md)).

**What the user gets**

- **Wait for OSC / UDP / MQTT** nodes: bind or broker, match rule (OSC address pattern and argument conditions; UDP text/hex contains or regex; MQTT topic filter and payload match), timeout. Outputs **Matched** and **Timeout**, so a missing reply is a branch, not a crash.
- Matched values are extractable: `{{reply.args[0]}}`, `{{reply.payload}}`, `{{reply.from}}`.
- **Expect reply** option on OSC, UDP and MQTT action nodes: one node sends and waits (address/topic, match, timeout). That covers most request/response pairs without a second node.
- **Retry** on every action and wait node: attempts, backoff (fixed/exponential), what counts as failure. The timeline shows each attempt.
- **Repeat** on action nodes: count or duration, interval, optional jitter — heartbeats and polling without graph loops.
- **Loop** node (bounded, later in the milestone): a body branch returning to the loop, with a max iteration count and an optional exit condition.

**Convenient paths**

- The OSC monitor and MQTT tree get **Wait for this** on a received message: it creates a Wait node with the match rule taken from that message.
- In the timeline, a satisfied wait links to the event that satisfied it (and to its Inspector frame).

**How (engine)**

- Listeners are **armed when the run starts**, not when execution reaches the wait. Each Wait node owns a bounded queue of matching events with arrival timestamps; reaching the node consumes the first event that arrived after the previous step started. This removes the race between "send" and "start listening", and binding errors surface before any traffic is sent.
- Listener sockets and MQTT connections belong to the run's task set, so Stop and End release them (the same guard that now cancels branches).
- New output ports `matched`/`timeout` join `yes`/`no` and `branch1`/`branch2` in `validPortsFor` and Rust validation.
- Retry and Repeat are node properties executed by the runner, so the graph stays a DAG and validation stays simple. Loop is a structured node with an explicit body; cycles elsewhere remain invalid.
- Tests use local fixtures: an OSC echo and a UDP responder bound to loopback inside `cargo test`.

**Done when** an experiment sends `/ping`, waits up to 500 ms for `/pong` with the same id, extracts an argument, retries twice on timeout, and the report states which reply or which timeout ended each step.

## 5. Mock services

**Goal.** SignalLab plays the dependency: the service, device or API the system under test calls.

**Status:** delivered as **emulators** — an *Emulators* screen with a library (`emulators.json`, a demo API and a demo OSC, UDP and TCP device on loopback), HTTP routes (method, `:name` path segments, header/query/body/JSON conditions) with responses in sequence, in turn or a seeded weighted mix, delays with seeded jitter and the faults *no answer* and *connection closed*; OSC/UDP responders and a TCP line device (greeting, hang-up); templated replies (`{{request.…}}`); per-rule counts, a live exchange list and Inspector frames; **Mock this** on the HTTP screen; the **Emulator** node (opened before the first step, its counts in the report) and **Wait for HTTP request**; the template *Retry a flaky API* (the "done when" below, also an engine test); `signallab emulate`, MCP tools and the API commands. Since then also: *Mock this* on a *Send now* response (a URL template becomes a path pattern), *Copy URL* per route, the *malformed* fault (JSON that stops halfway), **outages** on every emulator (up so long, down so long — the flapping of milestone 6, as a fixed schedule), and an **MQTT 3.1.1 broker** emulator (QoS 0/1/2, retained, wills, takeover, a login, device rules), so MQTT gear and *Wait for MQTT* need no broker installed. Section 5 is complete.

**What the user gets**

- **Mock HTTP** as a direct screen (always-on endpoints while testing an app by hand) and as a node (started for a run, stopped at its end).
- Routes: method, path pattern (`/users/:id`), optional header/query/body match. Responses: status, headers, body with templates (`{{request.params.id}}`, `{{uuid}}`), delay range.
- **Response sequences** (500, 500, 200) for retry/backoff tests, and a **fault mix** (80 % 200, 10 % 429, 5 % 500, 5 % timeout or connection reset), seeded.
- Every received request goes to the Inspector and can satisfy a **Wait for HTTP request** node, so an experiment can assert that the app called us, how many times, with what body.
- **Responder** rules for OSC/UDP (generalising the discovery auto-reply): on a matching message, reply with a templated one after a delay.

**Convenient paths**

- **Mock this** on the HTTP screen and on a *Send now* response: creates a route that returns exactly that response.
- Route list with hit counters and **Copy URL**; response presets (200 JSON, 404, 500, slow, timeout).

**How (engine)**

- `engine/mock.rs` on `hyper` 1.x (already in the dependency tree through `reqwest`), one job per mock server. Binds to `127.0.0.1` by default; binding to the LAN is an explicit, visible choice.
- Route matching and response rendering are pure functions shared by the screen and the node, and use `template.rs` from milestone 3.
- The fault vocabulary (delay, error status, reset, timeout, malformed body) is shared with milestone 6.

**Done when** an app pointed at the mock retries after two 500 responses, and an experiment proves it: three requests were received, the third got 200, all within the backoff window.

## 6. Faults as nodes and phases

**Goal.** Degrade the network or a dependency on a schedule, from the same graph, and always restore it.

**Status:** delivered — the **Impairment**, **Change impairment** and **Emulator down/up** nodes (document version 7); presets *LAN*, *Busy Wi-Fi*, *4G*, *Satellite*, *Intermittent*, *Offline*; bursts of loss (Gilbert–Elliott), reordering, a bandwidth limit and offline in the relay; profiles changed while a relay runs, on the Impairment screen too; every decision drawn from the run's seed; each phase counted in the run report (version 4) and every applied fault a timeline step; relays closed with the run whatever its outcome; *Route through impairment* on OSC and UDP nodes; the templates *Fault phases* and *Dependency outage*; a running emulator taken down and brought up by hand, over the API and over MCP. The "done when" below is an engine test (`engine/tests/faults.rs`). Still open: TCP impairment, and comparing two runs' reports side by side (milestone 7).

**What the user gets**

- **Impairment** node: starts the UDP relay (listen → target) with a profile; **Change impairment** switches profiles mid-run; the relay stops and is restored at the end of the run, whatever the outcome.
- Profile presets: *LAN*, *Busy Wi-Fi*, *4G*, *Satellite*, *Intermittent*, *Offline*.
- New impairments: bandwidth limit, reordering, burst loss (Gilbert–Elliott), blackhole.
- Dependency faults through the mock: error rate, latency, reset, flapping (up/down on an interval).
- A **fault schedule** template: a parallel branch with Delay and Change-impairment steps next to the traffic branch, so phases read top to bottom (0–20 s clean, 20–40 s +300 ms, 40–50 s 20 % loss, 50–60 s offline, then recover).

**Convenient paths**

- **Route through impairment** on any OSC/UDP node: inserts an Impairment node before it and points the node at the relay port. One click instead of editing ports by hand.
- Preset chips in the node form; the current profile is visible on the canvas node and in the timeline.

**How (engine)**

- `netsim.rs` gets a control channel so a running relay can change its `ImpairProfile` without rebinding; new fields default to "off", so existing configurations keep working.
- All random decisions come from the experiment seed; every applied fault is a timeline event.
- TCP proxy impairment (latency, throttling, reset, half-open) is a separate, later addition.

**Done when** the same seeded fault experiment gives the same drop pattern twice, Stop in the middle restores a clean relay, and a clean run and a faulted run can be compared.

## 7. Load profiles, thresholds, run comparison

**Goal.** Measure, not only pass/fail.

**Status:** started — the HTTP screen's burst runs at a fixed rate (open model) or as fast as its workers go, counts the requests it missed, and reports p50/p90/p95/p99 (`engine/tests/burst.rs`). Still open: profiles, the *Load* setting on a node, thresholds, run history and Compare.

**What the user gets**

- **Load** on an HTTP node (later OSC/UDP/MQTT): constant rate, ramp, step, spike, soak, Poisson arrivals; bounded by rate and duration caps.
- Metrics: RPS, p50/p90/p95/p99, error rate, status distribution, bytes/s; a latency histogram.
- **Thresholds** as checks: `p95 < 300 ms`, `errors < 1 %` — pass/fail in the report.
- **Run history** from the saved reports and **Compare** of two runs (p95 218 → 347 ms, +59 %).

**Convenient paths**

- **Load test this node** in the properties panel: a dialog with the profile drawn as a chart before it starts.
- Compare opens from the timeline: "compare with the previous run of this experiment".

**How (engine)**

- The burst runner already has both models: a fixed rate on an absolute schedule (each request due at n / rate, missed when no worker is free within 50 ms) and the closed model (each worker sends again on its answer); a profile makes the rate a function of time.
- Percentiles come from `latency::LatencyHistogram` (log buckets 1 % wide, atomics, constant memory); progress events stay batched and rate-gated.
- Reports gain a metrics block; comparison is a pure function over two reports.

**Done when** a ramp test with thresholds fails for the right reason and its comparison with a previous run shows the regression.

## 8. Record → replay → mutate

**Goal.** Record a real session with the equipment on site, replay it at home without the equipment, and break it on purpose.

**What the user gets**

- **Record** in the Inspector: a durable recording (separate from the display ring) with exact bytes, monotonic timing, direction, protocol, endpoints and gaps, saved as a `.slrec` file.
- **Replay**: speed 0.5×/1×/2×/10×/max, range selection, host/port remapping, template substitution (replace a token or device id).
- **To experiment**: selected recorded messages become a draft flow of send nodes with the recorded delays.
- **Mutate**: duplicate, drop, reorder, truncate, bit-flip, boundary values for OSC arguments, invalid type tags, oversize payloads — seeded. A failing case is saved as a reproducer (seed, case number, bytes) and can be replayed alone.
- Security simulations stay within this model: **replay attack** (resend a recorded command later), **identity collision** (two senders claim the same device id through templates), **out-of-order/duplicate delivery**.

**Convenient paths**

- **Replay this** / **To experiment** on selected Inspector frames; a speed slider and a remap table in one panel.
- A **Mutations** tab that lists generated variants and marks the ones that made the system misbehave.

**How (engine)**

- A recorder subscriber at the existing `inspect::publish` sites receives full payloads only while recording is armed (same single-atomic check as capture). HTTP is recorded as request/response pairs; truncated or summary-only material is marked non-replayable and never offered as exact replay.
- `.slrec` is JSONL (base64 bytes), streamed to disk with a size cap.
- Mutation operators are pure functions over bytes or decoded OSC messages, each seeded and unit-tested.

**Done when** a recorded OSC/UDP/MQTT session replays without the original sender, and a mutated case that broke the receiver reproduces from its saved file.

## 9. Virtual devices

**Goal.** One device template, many instances, each with its own identity, state, timers and reactions.

**What the user gets**

- **Device template**: state variables (`temperature = 23.4`, `battery = 87`), timers (every 1 s publish telemetry, every 5 s HTTP heartbeat), handlers (on OSC `/reset` set `battery = 100` and reply), built from the same action, wait and check vocabulary.
- **Spawn** 1, 10, 100 instances with `{{device.id}}`, `{{device.index}}` and a per-instance seed; state and timers per instance.
- A **devices table**: online/offline, last message, errors, per instance; aggregate metrics for the population.
- Combined with faults: "which instances recovered after the outage, and how long did it take".

**Convenient paths**

- **Make a device from this flow**: turns a working experiment branch into a template.
- Spawn count and rate limits in one dialog, with a preview of the traffic the population will generate.

**How (engine)**

- A device runtime executes a template per instance as tasks owned by one job; shared sockets where the protocol allows, bounded per-instance queues.
- Resource budgets (instances, messages per second) are enforced and shown before start; scale targets are set from measurements, not promised up front.

**Done when** 100 simulated sensors publish MQTT and answer OSC commands, survive a fault phase, and the report lists which instances recovered and when.

## 10. More transports, bridge, headless runner

Added when a real experiment needs them, each on the same node/wait/template model:

- **WebSocket**: connect, send, wait for message, close — delivered (the WebSocket screen, four nodes, `send ws`, `send_ws`; document version 8). Still to come: load with many clients, and a WebSocket emulator.
- **SSE** listener: events as waitable messages, reconnect detection.
- **Scripted TCP/UDP exchange**: `SEND HEX`, `WAIT`, `EXPECT` with wildcards, `READ n`, `EXTRACT bytes[4:8]` — for proprietary binary protocols.
- **Protocol bridge**: a job that maps OSC ↔ MQTT ↔ HTTP ↔ UDP with templates (`/sensor/temp $1` → topic `sensor/temp`, payload `{"t": $1}`), useful as a temporary integration gateway.
- **Headless runner** for CI: `signal-lab run experiment.json --profile stage --param k=v --report out.json`, exit code by outcome, JUnit output. The engine already runs without Tauri (`engine::Host`, `engine::Service`, delivered with server mode), so this is a thin binary over the same command table.
- **gRPC**, distributed agents and a topology canvas are separate projects, considered only with concrete demand.

## Delivery

Detailed design: [docs/delivery.md](docs/delivery.md).

- **Delivered:** `npm run check` as the single definition of the checks, run by CI on Windows and Linux for every push and pull request; one version source; `CHANGELOG.md` as the release notes; `npm run release` cutting a tag from `main`; a release workflow that builds Windows (NSIS, MSI) and Linux (deb, rpm, AppImage) installers into a draft release with checksums for a person to publish; Linux checks and packages from a Windows machine through Docker.
- **Delivered — server mode (D1–D4):** the engine independent of Tauri (a Cargo workspace: `engine`, `server`, `src-tauri`), `signal-lab-server` with the same commands over HTTP and WebSocket, the interface in a browser, and the image `ghcr.io/proanima/signallab` (amd64 + arm64, smoke-tested in CI, published when a release is published, with SBOM and attested provenance). Loopback by default, a token for anything else; `--network host` on Linux for broadcast, multicast and discovery.
- **Later:** code signing for Windows installers and signed checksums.

## Cross-cutting work

- **Document format.** Each milestone that changes the file bumps the version and ships a migration from the previous one with tests. Templates in `experiments/templates/` are updated with it.
- **Target safety.** A per-experiment allowlist (loopback by default); a preview of hosts, ports and rates before broad traffic (broadcast, sweep, load, device populations).
- **Timeline.** Filters by node, protocol and state; lanes per protocol once runs carry waits, faults and loads together.
- **Time.** Monotonic elapsed time for ordering, delays and replay; wall clock only for display.
- **Performance.** All high-rate events go through the rate gate and report what was not drawn.
- **Tests.** Local protocol fixtures in `cargo test`; editor transformations in `npm test`; every new node kind round-trips through JSON and appears in both locales.

## Next concrete slice (milestone 7 — load profiles and thresholds)

Delivered since milestone 6 (faults as nodes and phases, seeded and counted per phase): the HTTP burst at a fixed rate with p50/p90/p95/p99 (the start of milestone 7), WebSocket (milestone 10's first transport: a screen, connect/send/wait/close nodes, `send ws`), Basic/Bearer/Digest authentication and cookie jars for HTTP, and a parameter matrix for `signallab run`; document version 8. Milestone 5 before it delivered emulators (HTTP, OSC, UDP, TCP, an MQTT broker), with outages and the malformed fault, the *Emulator* node and *Wait for HTTP request* (version 6). Every screen, the WebSocket screen included, is walked end to end — desktop app and server, Windows and Linux, and the published image (`npm run e2e`). Next: the rest of milestone 7 — the *Load* setting on an HTTP node with ramp, step and spike profiles, thresholds as checks (`p95 < 300 ms`), and Compare of two runs' reports.

### PR 4.2 as planned

1. ~~**Expect reply** on OSC and UDP action nodes~~ — delivered.
2. ~~**Wait for this**~~ on a message in the OSC monitor (and an MQTT topic), and the timeline's link to the Inspector frame — delivered.
3. ~~**Wait for MQTT**~~ on a connection owned by the run — delivered.
4. ~~**Retry** on action and wait nodes~~ — delivered, with the tests of item 5 (an echo device in `cargo test`, retry counts and backoff timing, Stop during a pause).

## Explicitly outside the plan

Collection management in the style of Postman, a general scripting language inside nodes, million-request load generation, arbitrary source-IP spoofing, an unrestricted pentest suite, and a topology simulator. Any of these can return only with a concrete experiment that needs it.
