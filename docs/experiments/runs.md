---
title: Runs and results
description: Starting a run with the experiment's values or others, the timeline, stopping, what passed and failed mean, the run report, seeds, trying one node, and experiment files with their versions.
---

# Runs and results

## Starting a run {#start}

Press [[ui:exp.run]] in the editor's toolbar. Before anything is sent:

1. The experiment is checked as the editor checks it — its graph, fields,
   names and values ([what is checked](flow.md#validation)) — and every secret
   it uses must be stored ([secrets](data.md#secret-check)).
2. It is saved.
3. The run opens what it needs for its whole length: its emulators, its
   impairment relays, the sockets its waits listen on and its MQTT
   subscriptions.

If any of this fails, nothing runs: the problem is shown, and the node it is
about is selected. Otherwise the timeline opens under the canvas and the steps
appear in it as they happen. While the run goes, [[ui:exp.run]] becomes
[[ui:common.stop]] and the experiment cannot be edited.

The run uses the active profile's values, and the seed pinned in the
experiment or a new one. To run once with others, use [[ui:exp.runWith]].

### Running with other values {#run-with}

The ▾ next to [[ui:exp.run]] opens [[ui:exp.runWith]]: other values for one
run, without changing the experiment.

| Field | What | Empty |
| --- | --- | --- |
| [[ui:exp.profile]] | the profile for this run; shown when the experiment has profiles | the active one |
| each parameter | a value for this run only | the value of the chosen profile, shown greyed |
| [[ui:exp.seed]] | the seed for this run, 0–9 007 199 254 740 991; the button beside it fills in the last run's seed | the pinned seed, or a new one |

[[ui:exp.run]] in the form starts the run; [[ui:exp.resetOverrides]] empties the
form. What you typed stays in the form for the session, so the same change is
one click the next time. A profile that would not run is marked ⚠. The
precedence of values is in [data](data.md#precedence).

When the experiment has profiles, or a run had values typed for it, the
timeline says which profile the run used — or [[ui:exp.runDefaults]] — and,
when values were typed, [[ui:exp.overridden]].

## The timeline {#timeline}

[[ui:exp.timeline]] lies under the canvas; ▸ and ▾ fold it, its edge resizes
it. It holds one row per step event, oldest first: the time, the node, its
state and what happened — `HTTP 200 · 41 ms`, `token = abc123`,
`/pong 42 ← 127.0.0.1:9000 · 12 ms`. A failure says why, its technical detail
in the tooltip. A click on a row selects its node on the canvas.

| State | The step |
| --- | --- |
| [[ui:exp.running]] | has started |
| [[ui:exp.passed]] | ended well, and chose its output |
| [[ui:exp.failed]] | failed; the first failure is the run's |
| [[ui:exp.retry]] | failed an attempt and will try again ([Retry](flow.md#retry)) |
| [[ui:exp.repeating]] | is sending again and again, at most one row a second ([Repeat](flow.md#repeat)) |
| [[ui:exp.load]] | is under load, one row a second ([load](load.md#progress)) |

On the canvas, each node carries a badge with its latest state.

The timeline's header holds:

- the result: [[ui:exp.running]], [[ui:exp.passed]], [[ui:exp.failed]] with the
  reason, or [[ui:exp.stopped]];
- [[ui:exp.reportSaved]] once the report is written — in a browser a link that
  downloads it, in the desktop app its path in the tooltip;
- [[ui:exp.compare]], to put this run beside an earlier one
  ([comparing runs](load.md#compare));
- the profile and changed values, as above;
- the run's seed with [[ui:exp.pinSeed]], or, when the experiment has one
  pinned, that seed with [[ui:exp.unpinSeed]] ([seeds](#seeds)).

**Frames.** While the [[ui:dock.inspector]] is capturing, a wait — or a send
that waits for its reply — that matched a message keeps the number of that
message's frame. A button under the rows names the node and the frame; it
opens the Inspector in the bottom panel with that frame selected. See the
[Inspector](../tools/inspector.md).

The timeline shows the last run of the experiment in this session; it is
emptied when another experiment is opened.

## Stop {#stop}

Press [[ui:common.stop]], or [[ui:app.stopAll]] in the header for every job at
once. The run ends at once ([what stops](flow.md#stop)), the timeline shows
[[ui:exp.stopped]], and **no report is saved**. A run started from the command
line or the API on a server is a job like any other: [[ui:app.stopAll]] on that
server stops it too, and its caller learns it was stopped.

## The result {#result}

| Result | Means | Report |
| --- | --- | --- |
| [[ui:exp.passed]] | every branch finished, no step failed, and End was reached | saved |
| [[ui:exp.failed]] | a step failed — a check, a wait without a [[ui:exp.portTimeout]] wire, a network error, a threshold — or the run ran out of time (`run.timeout`), a Join waited in vain (`run.join_waiting`), or no branch reached End (`run.no_end`) | saved, with the first failure |
| [[ui:exp.stopped]] | someone stopped it | none |
| did not start | the experiment is invalid, a secret is missing, or a port could not be opened | none |

A failed run names the node and the field of its first failure; the
[error reference](../reference/errors.md) lists every code. The command line
says the same with its exit code: `0` passed, `1` failed, `2` the experiment or
the call was invalid (a missing secret counts), `3` something outside the
experiment kept it from running, such as a port that could not be opened. See
[`signallab run`](../automation/cli.md#cli-run).

## The run report {#report}

Every run that ends by itself — passed or failed — writes a JSON report in the
`runs` folder of the data folder: `Documents/SignalLab/runs` on a desktop, the
server's own data folder on a server ([files](../reference/files.md)). The file
is `run-<start time in ms>-<job number>.json`; a report is never written over
another. If it cannot be written, the editor says why.

| Key | What |
| --- | --- |
| `version` | the report's format, now 5 |
| `experiment` | the experiment's name |
| `document_version` | the experiment's version, now 9 |
| `seed` | the seed the run used |
| `profile` | the profile it ran with, or `null` for the defaults |
| `overrides` | the values typed in [[ui:exp.runWith]] |
| `params` | every parameter value the run used |
| `started_ms`, `ended_ms` | Unix milliseconds |
| `outcome` | `passed` or `failed` |
| `error` | the first failure, or `null` |
| `steps` | every step event, in order (below) |
| `emulators` | each Emulator node's counts — present when there is one ([emulators](faults.md#emulator)) |
| `impairments` | each Impairment node's counts and phases — present when there is one ([phases](faults.md#change-impairment)) |

Each step event has:

| Key | What |
| --- | --- |
| `job_id`, `node_id` | the run and the node |
| `ts` | Unix milliseconds |
| `state` | `running`, `passed`, `failed`, `retry`, `repeating`, `load` |
| `detail` | what happened, in English |
| `message_key`, `message_params` | the same as the interface's text and its values, so the step can be shown in any language |
| `vars` | the variables the step wrote, if any |
| `error` | why it failed: `code`, `params`, `node`, `field`, `detail` |
| `frame` | the Inspector frame a wait matched, if capture was on |
| `load` | what a load measured ([metrics](load.md#metrics)), on its last event |

Secret values never appear in a report: they are masked as `••••`
([masking](data.md#masking)).

The report's format grew with the features: version 3 added the emulators'
counts, version 4 the impairments' phases, version 5 a load's measurements.

The reports are the run history: [[ui:exp.compare]] reads them, and so does
[`experiment_runs`](../api/commands.md#experiment_runs). The command line's
`--report` copies a run's report where you want it.

## Seeds {#seeds}

Every run has a seed, a whole number from 0 to 9 007 199 254 740 991. It is,
in order:

1. the seed given to this run in [[ui:exp.runWith]], on the command line
   (`--seed`) or to the API;
2. the seed pinned in the experiment;
3. a new random seed.

The run's first row in the timeline gives it, and the report keeps it.

The seed decides everything random a run does: the
[generators](data.md#generators) in templates, the jitter of
[Repeat](flow.md#repeat), the arrivals of a [Random load](load.md#schedule), the
fate of every packet in an [impairment relay](faults.md#seed) and an emulator's
random choices. Each draws from a stream of its own, so parallel branches
never shift each other's values.

To repeat a run:

1. Press [[ui:exp.pinSeed]] next to its seed in the timeline. The seed is stored
   in the experiment, and every run uses it until you press
   [[ui:exp.unpinSeed]]. In [[ui:exp.params]], [[ui:exp.seed]] shows and edits
   the pinned seed; empty, it is [[ui:exp.seedRandom]].
2. Run with the same profile and values; the report lists them.

What a seed cannot repeat: the time (`{{now}}`), `{{run.id}}`, and when devices
and the network answer.

## Trying one node {#send-now}

To try one node without running the experiment, select it and press
[[ui:exp.sendNow]] — or <kbd>Ctrl</kbd>+<kbd>Enter</kbd> in its properties —
on an [[ui:exp.node.http]], [[ui:exp.node.tcp]], [[ui:exp.node.osc]],
[[ui:exp.node.udp]], [[ui:exp.node.mqtt]], [[ui:exp.node.ws_connect]] or
[[ui:exp.node.ws_send]] node. On a wait it is [[ui:exp.listenNow]]: it listens from now until a message
matches or its timeout ends.

The engine performs the node with the code a run uses, once:

- with the active profile's values, the variable values known in this session
  (from the last run and earlier tries) and the stored secrets;
- with the pinned seed, or a new one; `{{run.id}}` is `0` and `{{counter}}`
  is `1`;
- without Retry, Repeat or load — one send;
- **without cookies**: one request, nothing set before it to send back;
- without the run's relays and emulators. A [[ui:exp.node.wait_http]] listens
  on a listener of its own, and a WebSocket send or wait opens the connection
  its [[ui:exp.node.ws_connect]] describes, for that one try.

If a name the node uses has no value yet, nothing is sent and the result says
which names are missing — run the experiment, or use [[ui:exp.sendNow]] on the
node that sets them, first.

The result shows ✓ or ✕ and what happened. For an HTTP request it also shows
the status, the time, the size and the [[ui:http.response]]; in a JSON response
each value can be clicked to [extract it](data.md#extract), and
[[ui:http.mockThis]] turns the response into a route of an emulator. The
values a wait received, or that the [[ui:exp.node.extract]] nodes right after a
request would take from its response, become known to the preview and the
next [[ui:exp.sendNow]]. While a run goes, [[ui:exp.sendNow]] is not
available.

The preview of a templated node — what it [will send](data.md#preview) — is
resolved by the engine too, without sending anything.

## Experiment files {#files}

### The working experiment {#working-file}

The editor holds one experiment, saved by itself 0.7 s after each change to
`experiment.json` in the data folder; the toolbar says [[ui:exp.saving]],
[[ui:exp.saved]] or [[ui:exp.saveError]]. An unfinished graph saves too. A file
that cannot be read is reported with its path, never replaced. An experiment
file is at most 4 MiB.

On a server the file is in the server's data folder, so every browser that
opens the editor there works on the same experiment.

### Opening, templates and export {#open-export}

The ☰ button in the toolbar opens [[ui:exp.documents]]:

- [[ui:exp.templates]]: [[ui:exp.templateEmpty]], [[ui:exp.templateHttp]],
  [[ui:exp.templateBranch]], [[ui:exp.templateParallel]],
  [[ui:exp.templatePingReply]], [[ui:exp.templatePoll]],
  [[ui:exp.templateFlaky]], [[ui:exp.templateFaults]],
  [[ui:exp.templateOutage]], [[ui:exp.templateWsEcho]]. Their targets are on
  `127.0.0.1`.
- [[ui:exp.importJson]] reads a file of up to 4 MiB — of this version of the
  experiment format or an older one, which is brought up to date as it opens —
  and checks it before showing its name and how many nodes and connections it
  has. The file must also fit in 4 MiB as the editor writes it, indented, so a
  compact file close to the limit can be refused. A broken file is refused with
  the line and column of the problem, a file from a newer Signal Lab with
  `doc.version_unsupported`, and the current experiment stays.
- [[ui:exp.openDocument]] replaces the current experiment with the chosen one.
  <kbd>Ctrl</kbd>+<kbd>Z</kbd> brings the previous one back during this
  session. Opening an experiment does not run it.
- [[ui:exp.exportJson]] writes a copy to the `exports` folder of the data
  folder, as `experiment-<time in ms>-<random>.json`, never over another copy;
  in a browser, [[ui:common.download]] fetches it.

The command line and the API take the same files, and the templates by name:
`empty`, `http-check`, `status-branch`, `parallel-flows`, `osc-ping-reply`,
`poll-until-ready`, `flaky-api`, `fault-phases`, `dependency-outage`,
`websocket-echo`.

### Document versions {#versions}

An experiment file has a `version`; this Signal Lab writes version 9 and opens
every earlier one, filling in what the older file could not hold. A file of a
version newer than 9 is refused (`doc.version_unsupported`) rather than opened
without what it holds.

| Version | Added |
| --- | --- |
| 2 | parameters and the seed |
| 3 | profiles |
| 4 | Retry, and a reply awaited by an OSC or UDP send |
| 5 | Repeat, and [[ui:exp.node.loop]] |
| 6 | [[ui:exp.node.emulator]] and [[ui:exp.node.wait_http]] |
| 7 | [[ui:exp.node.impairment]], [[ui:exp.node.impairment_change]] and [[ui:exp.node.emulator_state]] |
| 8 | the WebSocket nodes, HTTP authentication and the cookie jar |
| 9 | load on an HTTP request, and impairment over TCP |

A file from before version 8 opens with [[ui:exp.cookies]] off, so it runs as it
did; a newer file keeps its own setting. Saved again, any file becomes version
9.

## From the command line or a server {#automation}

A run is the same everywhere: the command line and the server's API start the
same run the editor does, with the same steps, result and report.

```bash
signallab run checkout.json --profile Stage -p api=http://192.0.2.10:8080 --seed 42 --report report.json
```

- [`signallab run`](../automation/cli.md#cli-run) runs experiment files or
  templates in this process or on a server, prints the steps like the timeline
  and exits with the result's code.
- [`POST /api/run`](../api/run.md) runs one on a server and answers with the
  result, or streams its steps as they happen. A client that goes away does not
  stop the run; it runs to its end and keeps its report.
- In CI: [GitHub Actions and others](../automation/ci.md).
