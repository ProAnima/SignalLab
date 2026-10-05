---
title: Architecture
description: How Signal Lab is put together — one Rust engine behind the desktop app, the server and the command line — and the rules that keep it that way.
---

# Architecture

Signal Lab is one networking engine, written in Rust, with several ways in: a
desktop app, a server that hands the same interface to a browser, and a command
line. The interface is React 19 and TypeScript. This page is the map for someone
about to change the code: the pieces, how they talk, where things are, and the
rules worth not breaking. The full, current list of those rules is
[CLAUDE.md](https://github.com/ProAnima/SignalLab/blob/main/CLAUDE.md) at the
root of the repository; read it before a larger change.

## The pieces {#pieces}

| Piece | Where | What it is |
| --- | --- | --- |
| Interface | `src/` | React 19 + TypeScript, built by Vite into `dist/`. The same build runs in the desktop window and in a browser. |
| Engine | `engine/` (`signal-lab-engine`) | Every protocol, the capture bus, jobs, the experiment runner, emulators, the impairment relay. No Tauri, no window. |
| Desktop app | `src-tauri/` (`signal-lab`) | Tauri 2: one process, the interface in the system's webview (WebView2 on Windows, WebKitGTK on Linux), the engine in-process. |
| Server | `server/` (`signal-lab-server`) | axum: the engine's commands over HTTP, its events over a WebSocket, the built interface as static files, sign-in. Also the Docker image `ghcr.io/proanima/signallab`. |
| Command line | `cli/` (`signal-lab-cli`, the binary `signallab`) | Runs experiments, sends single messages and serves emulators without a window, here or on a server; `signallab mcp` for an LLM. |
| Documentation | `docs/` | This site (VitePress), built into the app and the server at `/docs/`. |

The four Rust crates are one Cargo workspace (`Cargo.toml`): one version, one
`Cargo.lock`, one `target/`. The version is written only in `package.json`;
`scripts/version.mjs` copies it into the Rust manifests, and Tauri reads it from
`package.json` directly.

## How they talk {#how-they-talk}

```text
                    interface (src/) — src/lib/transport.ts
                 ┌──────────────────────┴──────────────────────┐
   desktop app:  invoke("engine", …)               browser:  POST /api/invoke/<command>
                 Tauri events                                WebSocket /api/events
                 ▼                                             ▼
            src-tauri/                                    server/ (axum)  ◀── signallab --server
                 └──────────────────────┬──────────────────────┘             (/api/run, /api/invoke)
                                        ▼
                     engine::Service::invoke(name, json)   ◀── signallab (in its own process)
                     engine::Host::emit(event, payload)
                                        │
          protocols · capture bus · jobs · experiment runner · emulators · impairment relay
```

### Commands: one table {#commands}

`Service::invoke(name, json)` in `engine/src/service.rs` is **the** command
table: a `match` on the command's name, each arm reading its arguments through
the `args!` macro (camelCase, as the interface writes them; an unknown field is
an error, `command.args_invalid`) and answering JSON or a `Failure` — always an
`EngineError`. Both front doors only forward to it:

- **Desktop:** `src-tauri/src/lib.rs` registers one Tauri command for engine
  work, `engine(command, args)`, which calls `Service::invoke`. The few other
  Tauri commands there are about the window (whether to look for updates, the
  documentation's window), not about the engine.
- **Server:** `POST /api/invoke/<command>` (`server/src/routes.rs`) hands the
  JSON body to the same `Service`: `200` with the result, `422` with the same
  `EngineError` the desktop app gets. Commands in `JOB_COMMANDS` are logged with
  the client's address.

In the interface, `src/lib/api.ts` wraps every command in a typed function that
mirrors the Rust types, and `src/lib/transport.ts` is **the one place** that
knows whether it runs in the desktop app (Tauri's `invoke` and events) or in a
browser (`fetch` and one shared WebSocket that reconnects). Nothing else in the
interface talks to Tauri or to the server for engine work. What differs between
the two otherwise — fullscreen, downloads, links out, the documentation, sign-out,
updates — is in `src/lib/platform.ts`, and what the engine can do there comes
from `app_info` (`mode`, whether secrets can be written, the data folder, the
system), never guessed.

Adding a command is three edits: the function in `engine/src/*.rs`, its arm in
`Service::invoke`, and its wrapper in `src/lib/api.ts` (plus `JOB_COMMANDS` if it
starts a job). The command line reaches the same table: in its own process it
makes a `Service` of its own; with `--server` it calls the server's
`/api/invoke/<command>` and `/api/run`. The commands are listed for users in
[Commands](../api/commands.md).

### Events {#events}

The engine reports through `engine::Host` (`engine/src/host.rs`): an
`EventSink` plus the capture bus. `Host::emit(event, payload)` is all a module
calls; the sink decides where it goes:

- the desktop shell turns each one into a Tauri event of the same name;
- the server serializes it once and fans it out to every connected page over
  `/api/events` (`server/src/events.rs`); a page that falls behind is told how
  many it missed (`server://lagged`);
- tests keep them in a `Recorder`.

Channels are named `<area>://<what>` — `job://ended`, `inspect://batch`,
`experiment://step`, `osc://message`, `mqtt://messages`, … — and their payloads
carry codes and values, never finished sentences. High-rate sources are batched
(the MQTT and WebSocket screens every 100 ms, the Inspector's pump every 120 ms)
and say what they had to shed instead of dropping it silently. Users' view of
them: [Events](../api/events.md).

## Jobs {#jobs}

Anything that runs for a while — a monitor, a generator, a burst, a relay, a
storm, a scan, a beacon, discovery, an MQTT or WebSocket connection, an emulator,
an experiment run — is a **job** in `JobRegistry` (`engine/src/jobs.rs`): started
by a command, listed by `jobs_list`, stopped by `job_stop` or `jobs_stop_all`,
and ended with `job://ended` and its reason. A job that ends on its own calls
`finish(id)`. The console strip at the bottom of every screen lists them; a job's
name there is `job.<kind>` from the dictionary, filled in from its `params`, while
its English `label` is for the server's log.

## The capture bus {#capture}

Modules never call the Inspector. They publish a normalized `Frame` to
`engine::inspect` (`engine/src/inspect.rs`), which keeps a bounded ring and pumps
batches to the interface as `inspect://batch`:

- **Armed explicitly.** While capture is off, publishing is a single atomic load,
  so the hot paths cost nothing.
- **Bounded.** The ring holds 8192 frames and 64 MiB of payload, whichever fills
  first; frames are numbered under the ring's lock, so each is shipped once and
  in order.
- **Bytes kept, batches light.** A frame keeps its payload (secrets masked) up to
  256 KiB; a batch carries only a 1 KiB hex preview, and the rest is asked for by
  number (`inspect_payload`).
- **Sampled, never silently dropped.** High-rate sources go through a rate gate
  (`inspect::Gate`, `inspect::Budget`) and report what was not drawn as
  *"N not shown"*; the ring still holds it for export.

## Experiments {#experiments}

An experiment is a JSON document (`engine/src/experiment.rs`: nodes, wires,
outputs, a format version with migrations in `experiment_files.rs`). The runner
is split by responsibility:

| Module | Does |
| --- | --- |
| `experiment_validate.rs`, `experiment_fields.rs` | the document, the graph and each node's fields checked before anything is sent |
| `template.rs`, `experiment_data.rs` | the `{{template}}` language, parameters, extraction, value checks — templates are resolved only here, so a preview (`experiment_resolve`) and *Send now* (`experiment_send_node`) mean what a run means |
| `experiment_steps.rs`, `experiment_actions.rs` | what one step does; one network action, its failure classified |
| `experiment_flow.rs` | how a run moves: branches, joins, retry, repeat, the loop |
| `experiment_run.rs` | a run started and followed: listeners, subscriptions, emulators and relays opened **before the first step**, events, the `RunHandle` |
| `experiment_report.rs`, `experiment_compare.rs` | the report file; run history and two runs compared |
| `load.rs`, `latency.rs` | an HTTP node under load: the schedule, the metrics, the thresholds |

A run is one job and one report however it was started: the app's *Run*
(`experiment_start`), the server's `POST /api/run` and `signallab run` all go
through `experiment_run::start_followed`, which hands each step and then the
result to whoever follows it; a client that goes away does not stop the run.
Secrets live behind `SecretStore` (`engine/src/secrets.rs`): the OS credential
store on a desktop, read-only files or environment variables on a server, a
`MemoryStore` in tests. No command returns a secret's value, and everything a run
reports passes through `secrets::mask`.

For users: [Experiments](../experiments/index.md).

## Emulators and impairment {#emulators-impairment}

**Emulators** stand in for the other side: an HTTP API, an OSC or UDP device, a
TCP device, an MQTT broker. An emulator is one document everywhere
(`emulator::Emulator`): the Emulators screen, the *Emulator* node,
`signallab emulate`, MCP and the API all hand it to `emulator_rules::compile`,
and the protocol modules serve it (`emulator_http.rs` on hyper, `emulator_net.rs`
for OSC/UDP/TCP, `emulator_mqtt.rs` on the same MQTT codec as the client). On its
own an emulator is a job (`emulator_job.rs`); in a run it is opened by
`emulator_run::arm_run` before the first step, and an HTTP emulator's server is
also what *Wait for HTTP request* reads.

**Impairment** is a relay between a client and its target that delays, drops,
duplicates, corrupts, reorders or throttles traffic, or cuts it off: `netsim.rs`
for UDP datagrams, `netsim_tcp.rs` for TCP streams. It runs as a job from the
Impairment screen or, inside a run, as an *Impairment* node opened by
`netsim_run::arm_run`; *Change impairment* changes it in place. Every decision
draws from a seeded stream per direction — the run's seed in a run — so the same
seed and the same traffic drop the same packets.

For users: [Emulators](../tools/emulators.md),
[Network impairment](../tools/impairment.md), [Faults](../experiments/faults.md).

## Errors and languages {#i18n}

**The engine never builds a sentence.** Every failure is an `EngineError`
(`engine/src/error.rs`): a `code`, its values in `params`, the `node` and `field`
it is about, and the system's own wording as `detail`. The code is a key of the
interface's dictionary, `err.<code>`; a field is `field.<key>`. Codes are written
as literals (`EngineError::new("wait.timeout")`), because a test scans the engine,
the server and the command line and fails when `en.ts` has no text for a code or
keeps one nobody uses. Network failures reuse the `transport.*` causes (refused,
timeout, name not found, …). The interface shows every failure through
`describeError` (`src/lib/errors.ts`) and `ErrorMessage`, and stores the failure,
not its text, so switching the language re-renders what is on screen, the console
included.

`src/lib/locales/en.ts` is the source of every text and defines the `Dict` type;
every other language is typed against it, so a missing key is a compile error.
The command line speaks the same texts (`cli/build.rs` embeds the dictionaries),
and the server's sign-in page has its own small table. How to write a text, what
is checked and how to add a language: [Localization](localization.md).

## The documentation {#documentation}

`docs/` is a VitePress site: the user pages in every language of the interface,
and these pages in English. `npm run build` builds it into `dist/docs`, so the
desktop app and the server carry it and it works offline; <kbd>F1</kbd> opens the
page of the screen in view (`src/lib/docs.ts`), in the interface's language — in
a window of its own on the desktop, in a tab in a browser. Pages name buttons and
fields by their dictionary keys (`[[ui:key]]`), so a renamed label breaks the
build instead of leaving a page wrong. How to write them:
[Writing the documentation](writing-docs.md).

## The repository {#repository}

| Path | What |
| --- | --- |
| `src/views/` | one screen per module; `ExperimentView.tsx` composes the node editor |
| `src/components/` | shared parts: `TooltipLayer`, `Splitter`, `Scope`, `OscArgs`, `Palette`, `ErrorMessage`, the experiment editor's pieces (`Experiment*.tsx`), `EmulatorEditor`, … |
| `src/lib/` | `transport.ts`, `platform.ts`, `api.ts`, `store.tsx` (jobs, console, connection), `i18n.tsx` and `translate.ts`, `locales/`, `errors.ts`, and the pure logic the tests cover (`experimentGraph.ts`, `library.ts`, `load.ts`, `emulators.ts`, …) |
| `src/styles/` | the stylesheet by area, `@import`ed in cascade order by `src/styles.css`; `rtl.css` for right to left |
| `engine/src/` | the engine: `service.rs`, `host.rs`, `paths.rs`, `error.rs`, `jobs.rs`, `inspect.rs`; protocols (`osc*`, `mqtt*`, `http*`, `ws.rs`, `broadcast.rs`, `discovery.rs`, `storm.rs`, `scan.rs`); `experiment_*.rs`; `emulator_*.rs`; `netsim*.rs` |
| `engine/tests/` | experiments run end to end over loopback, bursts, load, authentication, WebSocket, emulators, faults |
| `src-tauri/` | the desktop shell (`src/lib.rs`), `tauri.conf.json`, the icons, `installer/` (artwork, NSIS hooks, the MSI fragment, installer languages) |
| `server/src/`, `server/tests/` | `config.rs` (options, checked once), `auth.rs` (token, sessions, Host/Origin), `routes.rs`, `events.rs`, `run.rs` (`/api/run`); the server end to end |
| `cli/src/`, `cli/tests/` | `run.rs` and `remote.rs`, `send.rs`, `emulate.rs`, `mcp*.rs`, `catalog.rs` (every kind of node), `i18n.rs`, `junit.rs`; the binary as a pipeline runs it |
| `experiments/templates/` | the bundled experiments: the app's templates, also runnable by name (`signallab run empty`) |
| `docs/` | the documentation; `docs/api/openapi.json` is the API's description, built into the server |
| `tests/` | `*.test.mjs` (`npm test`), `e2e/` (the end-to-end tour), `webkit/` (the tooltip harness) |
| `scripts/` | `check.mjs` (the one list of checks), `version.mjs`, `release.mjs`, `linux.mjs`, `image.mjs`, `e2e.mjs` and `e2e/fixtures.mjs`, `webkit.mjs`, `docs.mjs`, `cli-bundle.mjs`, the icon and installer-art scripts |
| `docker/` | `linux-builder` (Linux checks, packages and the tour from any machine), `webkit` (tooltips in WebKitGTK) |
| `Dockerfile`, `deploy/` | the server image; `compose.yaml` and `install.sh` |
| `action.yml` | the GitHub Action around `signallab` |
| `.github/` | `ci.yml`, `release.yml`, `image.yml`, Dependabot |

## Invariants worth not breaking {#invariants}

The short version; [CLAUDE.md](https://github.com/ProAnima/SignalLab/blob/main/CLAUDE.md)
has every rule with its reasons.

- **One command table.** Both front doors only forward to `Service::invoke`;
  neither changes when a command is added.
- **The engine never depends on Tauri.** Events go through `Host::emit`, capture
  through `host.capture()`, files under `paths::data_dir()`. Anything desktop-only
  lives in `src/lib/platform.ts` with a browser equivalent.
- **The engine never builds sentences.** Failures are codes with values; console
  lines are keys with values; jobs are `job.<kind>`.
- **Long-running work is a job**, registered so it can be listed and stopped.
- **Modules publish frames**; they never call the Inspector.
- **One socket, one owner.** An MQTT connection and a WebSocket each belong to a
  single task; everything else reaches it by channel.
- **Templates are resolved only by the engine**, and **secret values never leave
  it**: no command returns one, every string a run reports is masked, frames are
  redacted.
- **A run is followed, not polled**: one runner and one report for the app, the
  server and the command line.
- **Runs are reproducible.** Seeded generators, seeded jitter, seeded relay
  decisions per direction — no `thread_rng` in the relay.
- **The server is safe by default.** Without a token it listens only on loopback;
  any other address without one is a startup error. Commands take JSON only,
  state changes and the WebSocket must match `Origin`, sessions are `HttpOnly` +
  `SameSite=Strict`.
- **Every kind of node is in two lists**: `cli/src/catalog.rs` and
  `cli/tests/nodes.rs`; both fail on a kind they lack.
- **`signallab mcp` owns stdout**: one JSON-RPC message per line, nothing else.
- **Screens stay mounted.** A view must not assume it is visible and must not
  grab global keys.
- **The interface is accessible and translatable.** Every clickable thing is a
  real `<button>`, every field has a name, help is a `data-tip` tooltip and never
  a caption or a native `title`, and sides are inline-start/end so Arabic mirrors.
- **The firewall changes only when a person says so**, and **updates come only
  from published, signed releases.**
- **No secret in the app**: feedback goes through the studio's
  [hub](hub.md), which alone holds the mailbox's password.
- **Never a default target at a host we do not own.** Storm, Scanner and
  Broadcast send real traffic; the engine's caps (a sweep of 1024 hosts, a beacon
  of 50 000 packets a second) are guard rails, not permission.
