---
title: Concepts
description: The ideas behind Signal Lab — screens and experiments, signals, jobs, capture, emulators, impairment relays, parameters, templates, secrets, seeds, reports and the data folder.
---

# Concepts

This page explains the ideas Signal Lab is built on, so that the rest of the
documentation reads easily. Each section links to the page that covers its subject in
full.

## Screens and experiments {#screens-and-experiments}

Signal Lab has two ways of working, and you will use both.

- **Screens** are tools for work you do now, by hand: send this message, listen on that
  port, start this emulator, see what the broker holds. You try, look, change something
  and try again. Each protocol and tool has a screen; see [The window](interface.md).
- **Experiments** are flows you build once and run again, the same way every time: send
  a request, wait for the reply, check it, carry on or branch. Each run is reported step
  by step and saved. See [Experiments](../experiments/index.md).

The two meet in several places. On the HTTP and OSC screens, [[ui:common.toExperiment]]
turns what you just sent into the next step of the experiment. On the OSC monitor and
the MQTT screen, [[ui:osc.waitForThis]] turns a message you received into a step that
waits for it. And a signal from the library can become a step — any but raw UDP
bytes written as hex.

## Signals and the library {#signals}

A **signal** is a message you keep: a name, a folder, a note on what it should make
happen, and what it sends — an OSC message, raw UDP bytes, an HTTP request or an MQTT
publish. The **signal library** holds them in folders you can nest, rename and drag
around.

- You save a signal from the OSC, HTTP and MQTT screens ([[ui:sig.saveNew]]), make one on
  the [[ui:nav.signals]] screen, or save a frame the Inspector caught
  ([[ui:sig.fromFrame]]), which then replays it byte for byte.
- You send it from the [[ui:nav.signals]] screen, from anywhere with
  <kbd>Ctrl</kbd>+<kbd>K</kbd>, as a step of an experiment, or with `signallab fire` from a
  terminal.
- A signal sends exactly what its screen would: the same bytes, through the same path,
  shown in the Inspector under its real protocol.

The library is one file, `signals.json`, in the [data folder](#data-folder): plain JSON
you can read, edit, copy to another machine or keep in a repository. It starts with a
set of example signals, all aimed at `127.0.0.1`. See [Signals](../tools/signals.md).

## Jobs {#jobs}

Anything that keeps running after you press its button is a **job**: an OSC monitor or
generator, a broker or WebSocket connection, a beacon or discovery listener, an HTTP
load burst, an emulator, an impairment relay, a storm, a scan, an experiment run.

- Each job has a pill in the bottom panel's strip, with its number, what it is and a
  button to stop it. The sidebar shows how many jobs each screen has running.
- [[ui:app.stopAll]] in the header stops every job at once.
- A job that ends on its own — a scan that finished, a run that passed, a monitor whose
  port failed — leaves its pill, and the console says how it ended.
- A job carries on while you work on other screens.
- Installing an update stops every job first.

On a server, jobs belong to the server: every page signed in to it sees the same jobs
and can stop them.

## Capture and the Inspector {#capture}

Every tool — senders, monitors, listeners, emulators, relays, experiment runs — hands
each frame it sends or receives to one **capture**, and the [Inspector](../tools/inspector.md)
shows it in one timeline.

- Capture is **off until you arm it** with [[ui:ins.arm]], and costs nothing while it is
  off. It stays on, whichever screen you are on, until you disarm it.
- It keeps up to 8192 frames and 64 MiB of their bytes; the oldest frames make room for
  new ones. Each frame keeps up to 256 KiB of its bytes, and the list shows its first
  KiB.
- [[ui:ins.pause]] stops the list from moving so you can read it; capture goes on
  underneath.
- Secret values used by a run are masked in every frame.
- The whole capture can be exported, every byte kept, to a `.jsonl` or `.txt` file.

## Emulators {#emulators}

An **emulator** plays the other side: the API, device or service your system talks to.
Each one is a document with a protocol, the address it listens on, and rules saying what
to answer:

| Protocol | What it emulates |
| --- | --- |
| HTTP | An API: routes by method and path, responses in sequence, in turn or at random, with delays and faults |
| OSC | A device that answers OSC messages by address and arguments |
| UDP | A device that answers datagrams by their payload |
| TCP | A device that answers lines on a TCP connection, with a greeting |
| MQTT | A broker that routes what clients publish, and answers by rules like a device |

An emulator can answer slowly, fail, close the connection, send a malformed body, or go
down on a schedule. Every exchange is counted, listed on its screen and captured for
the Inspector.

You start one from the [[ui:nav.emulators]] screen, where it runs as a job; from an
experiment's [[ui:exp.node.emulator]] node, where it answers for the whole run; or with
`signallab emulate`. The **emulator library** is `emulators.json` in the data folder. It
starts with one emulator of each kind, all on `127.0.0.1`:

| Emulator | Listens on | What it does |
| --- | --- | --- |
| [[ui:seed.emu.demo-api.name]] | `127.0.0.1:8080` (HTTP) | A health check, a user by id, a create, a slow answer, and a route that fails twice before it works |
| [[ui:seed.emu.osc-device.name]] | `127.0.0.1:9100` (OSC) | Answers `/ping` with `/pong` and a count, acknowledges `/fader/…` with `/ack`, takes `/cue/…` without a word |
| [[ui:seed.emu.udp-device.name]] | `127.0.0.1:7100` (UDP) | Answers `PING` with `PONG` and a count, anything else with how many bytes it got |
| [[ui:seed.emu.tcp-device.name]] | `127.0.0.1:7200` (TCP) | A line protocol like a projector's: greets with `READY`, reports and switches power, says `BYE` and hangs up on `QUIT` |
| [[ui:seed.emu.mqtt-broker.name]] | `127.0.0.1:1883` (MQTT) | A retained `lab/status`, and a lamp: `ON` or `OFF` published to `lab/<name>/set` is answered on `lab/<name>/state` |

See [Emulators](../tools/emulators.md).

## Impairment relays {#impairment}

An **impairment relay** sits between a client and its target. You point the client at
the relay's listening address instead of the real target; the relay forwards both ways
and degrades what passes through, by a **profile**:

- over **UDP**, each datagram meets its own fate: latency and jitter, loss and bursts of
  loss, duplication, corruption, reordering, a bandwidth limit, or nothing at all
  (offline);
- over **TCP**, each connection is joined to one of its own to the target, and both
  streams are delayed, held to a bandwidth limit, reset, or left half-open.

Presets set a profile in one click, from a cable to a satellite link. A change applies
while the relay runs, without dropping its port. Every decision is drawn from a seed, so
the same traffic meets the same fate again.

On the [[ui:nav.netsim]] screen a relay runs as a job. In an experiment, an
[[ui:exp.node.impairment]] node opens one for the run and [[ui:exp.node.impairment_change]]
switches its profile mid-run. See [Impairment](../tools/impairment.md) and
[Faults](../experiments/faults.md).

## Experiments {#experiments}

### Nodes and wires {#nodes-and-wires}

An experiment is a graph of **nodes** joined by **wires**. Each node is one step: it
sends something, waits for something, checks a value, extracts one, changes the flow,
or sets up the run — an emulator, an impairment relay. Every experiment has exactly one
[[ui:exp.node.start]] and one [[ui:exp.node.end]], and holds up to 64 nodes. The
experiment open in the editor is saved as you edit. See [Nodes](../experiments/nodes.md).

### Outputs {#outputs}

A wire runs from a node's **output** to another node's input. Most nodes have one
output; others choose between several: [[ui:exp.yes]] and [[ui:exp.no]] for a branch,
[[ui:exp.portMatched]] and [[ui:exp.portTimeout]] for a wait, [[ui:exp.portBody]],
[[ui:exp.portDone]] and [[ui:exp.portLimit]] for a loop, [[ui:exp.branch1]] and
[[ui:exp.branch2]] for a parallel branch.

An output may have several wires: each runs as a branch of its own, in parallel, and a
[[ui:exp.node.join]] waits for all the wires that lead into it. Only a
[[ui:exp.node.loop]]'s body may lead back; any other cycle is an error. See
[Flow](../experiments/flow.md).

### Parameters and profiles {#parameters}

A **parameter** is a named value — a host, a port, a user name — written once under
[[ui:exp.params]] and used in any field as `{{name}}`. A **profile** changes some
parameters at once: one for the laptop, one for the stage, one for the venue. You pick
the profile runs use, or [[ui:exp.runWith]] a profile, other values or a seed for one
run only, without changing the experiment. An experiment holds up to 64 parameters and
32 profiles. See [Data](../experiments/data.md).

### Templates {#templates}

Most text fields of nodes are **templates**: plain text with expressions in double
braces, filled in as the step runs.

- `{{host}}` — a parameter, or a variable set earlier in the run, such as a value an
  [[ui:exp.node.extract]] node took from a response, or a reply a wait received
  (`{{reply.args[0]}}`).
- `{{secret.API_TOKEN}}` — a secret.
- `{{run.id}}`, `{{run.seed}}`, `{{now}}`, `{{now.iso}}`, `{{counter}}` — the run and
  the moment.
- `{{uuid}}`, `{{random_int(1, 10)}}`, `{{random_float(0, 1, 2)}}`, `{{pick("a", "b")}}`
  — generated values.

Only the engine fills templates, so a field means the same thing in a run, in the
editor's preview and in [[ui:exp.sendNow]]. An unknown name is an error, never an empty
string. See [Data](../experiments/data.md).

### Secrets {#secrets}

A **secret** is a value an experiment uses but never stores — a token, a password. The
experiment holds only its name; fields use it as `{{secret.NAME}}`; and every text a run
reports, every step, the report and every Inspector frame, shows it masked. No command
ever hands a secret's value back.

Where the values live depends on where Signal Lab runs:

- **The desktop app on Windows** keeps them in the Windows Credential Manager. You set
  them under [[ui:exp.params]] → [[ui:exp.secrets]].
- **The desktop app on Linux** has no credential store to keep them in, so experiments
  that use secrets run from the command line or a server there.
- **A server** reads them, read-only, from its environment (`SIGNALLAB_SECRET_<NAME>`) or
  from a file per name in its secrets folder (`/run/secrets/signallab/<NAME>` by
  default); they cannot be set from a browser.
- **The command line** reads them the same way as a server, or from the system's
  credential store when asked. See [The command line](../automation/cli.md).

### Seeds {#seeds}

Every run has a **seed**, a number that decides everything random in it: generated
values, a repeat's jitter, an emulator's random choice of response, an impairment
relay's every decision. The same seed and the same traffic give the same run. A new
seed is drawn for each run unless the experiment pins one — [[ui:exp.pinSeed]] in the
run timeline pins the seed of the last run, and [[ui:exp.seed]] under
[[ui:exp.params]] sets one.

### Runs and reports {#reports}

A **run** starts at [[ui:exp.node.start]], follows the wires and passes when it reaches
[[ui:exp.node.end]] with no step failed. It is stopped if it takes longer than 300
seconds. Each step appears in the run timeline as it starts and ends.

A run that ends, passed or failed, writes a **report** into the `runs` folder of the data
folder: the experiment's
name, the seed, the profile and the values used, when it started and ended, the outcome
and its error, every step, and what its emulators and relays counted. Two runs of the
same experiment can be compared. See [Runs and reports](../experiments/runs.md).

## The data folder {#data-folder}

Everything Signal Lab keeps is a file in one folder: `Documents/SignalLab` in your home
folder, on Windows and Linux alike. A server keeps its own, which you choose when you
start it (`/data` in the Docker image).

| File or folder | What it holds |
| --- | --- |
| `experiment.json` | The experiment open in the editor |
| `signals.json` | The signal library |
| `emulators.json` | The emulator library |
| `runs/` | A report for each run |
| `exports/` | Experiments exported from the experiments dialog |
| `capture-….jsonl`, `capture-….txt` | Inspector exports |

The files are JSON, written whole. If one cannot be read, Signal Lab says which file
and where the error is, and leaves it as it is rather than starting over. See
[Files and folders](../reference/files.md).

## Desktop and server {#desktop-and-server}

The desktop app and a server run the same engine behind the same interface. What
differs:

| | Desktop app | Server, in a browser |
| --- | --- | --- |
| Where traffic starts, where monitors listen | This computer | The server |
| Data folder | `Documents/SignalLab` | The server's; hover [[ui:app.server]] in the header to see it |
| Secrets | Windows Credential Manager, set in the app; none on Linux | Read-only, from the server's environment or secret files |
| Reports, exports, captures | Written to the data folder; the path is shown | Downloaded by the browser |
| Signing in | — | With the server's access token, when it has one |
| Jobs, the HTTP screen's cookie jar | This app's | The server's, shared by every page signed in to it |
| Firewall | A notice offers to allow Signal Lab (Windows) | Never changed by Signal Lab |
| Updates | Installs signed releases when you click | Updated with its image |

See [The server](../server/index.md) and [Server security](../server/security.md).

## What Signal Lab does not do on its own {#on-its-own}

- **It sends only when you act**, and only to the addresses you type. Starting the app
  sends nothing — except, in the desktop app, the daily update check, which you can turn
  off. Feedback goes out only when you send the form.
- **Its examples stay on this computer.** The starter signals, the starter emulators,
  new emulators and the experiment templates all use `127.0.0.1`. Listeners you start —
  an OSC monitor, the discovery listener, an impairment relay — default to `0.0.0.0`, every
  network card, so that other machines can reach them; type `127.0.0.1` to keep one on
  this computer.
- **It changes the firewall only when you click** [[ui:fw.allow]] and confirm Windows'
  administrator prompt, or run `signallab firewall allow`. A server never changes its
  host's firewall.
- **A server without an access token** listens only on `127.0.0.1`, and refuses to start
  on any other address.
- **It keeps to guard rails**: a broadcast sweep reaches at most 1024 hosts, and a beacon
  sends at most 50 000 packets per second across all its targets.

The guard rails are not permission: [Storm](../tools/storm.md), the
[Scanner](../tools/scanner.md) and [Broadcast](../protocols/broadcast.md) send real
traffic. Use them only on networks and hosts you own or are authorized to test.
