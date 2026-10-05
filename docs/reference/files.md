---
title: Files and folders
description: Where Signal Lab keeps its files — the data folder, the experiment, the signal and emulator libraries, run reports and exports — their formats, and what is safe to edit, back up and move.
---

# Files and folders

Everything Signal Lab keeps is plain JSON (or text) in one folder, the data
folder. Secret values are never in it.

## The data folder {#data-folder}

| Where Signal Lab runs | The data folder |
| --- | --- |
| Desktop app, Windows | `Documents\SignalLab` in your user folder: `C:\Users\<you>\Documents\SignalLab` |
| Desktop app, Linux | `~/Documents/SignalLab` |
| Server | `--data-dir`, or `SIGNALLAB_DATA_DIR`; without either, `Documents/SignalLab` in the home folder of the user it runs as |
| Server, Docker image | `/data`, a volume (`signallab-data` in the compose file) |
| `signallab run` | A temporary folder, removed when it exits — unless `--data-dir` names one |

The desktop app also takes `SIGNALLAB_DATA_DIR` from its environment when it
is set. The folder is made when something is first written to it.

::: tip
On Windows the app uses the `Documents` folder directly inside your user
folder, even when Windows keeps your documents somewhere else (OneDrive).
:::

On a server, files are written on the server's machine, not on yours. The
[[ui:app.server]] badge in the header says where, in its tip; the API gives it
as `data_dir` of [`app_info`](../api/commands.md#app_info), and
[`/api/files`](../api/index.md#files) downloads what is in it.

The command line's `signallab emulate`, `signallab send` and `signallab mcp`
read the app's libraries from the same folder as the desktop app.

## What is in it {#contents}

| File | What it is | Written |
| --- | --- | --- |
| `experiment.json` | The experiment open in the editor | Shortly after every change |
| `signals.json` | The signal library ([[ui:nav.signals]]) | Shortly after every change |
| `emulators.json` | The emulator library ([[ui:nav.emulators]]) | Shortly after every change |
| `runs/run-<ms>-<job>.json` | One report per run that ended on its own | When the run ends |
| `exports/experiment-<ms>-<16 hex digits>.json` | A snapshot of the experiment | [[ui:exp.exportJson]] |
| `capture-<ms>.jsonl`, `capture-<ms>.txt` | The Inspector's frames | [[ui:ins.exportJsonl]], [[ui:ins.exportTxt]] |
| `token` | A server's access token, readable only by its user | `--generate-token`, on the first start |
| `.experiment-<hex>.tmp`, `.signals-<hex>.tmp`, `.emulators-<hex>.tmp` | A save on its way | For a moment, then renamed |

`<ms>` is a time in milliseconds since 1970; `<job>` is the run's job number.
On a server every browser works on the same `experiment.json`, the same
libraries and the same reports.

## Formats {#formats}

All are JSON in UTF-8, written with indentation so they read and diff well.
Each carries a `version`; a file of an older version is read and migrated
when it is opened, and written back in the current version the next time it is
saved — after which an older Signal Lab cannot open it.

### experiment.json {#experiment-json}

The experiment document, version 9 — the same JSON that
[[ui:exp.exportJson]] writes and [[ui:exp.importJson]] reads:

```json
{
  "version": 9,
  "name": "HTTP check",
  "params": [],
  "profiles": [],
  "profile": null,
  "seed": null,
  "cookies": true,
  "nodes": [ { "id": "start", "type": "start", "x": 40, "y": 80 }, … ],
  "edges": [ { "from": "start", "to": "request", "port": "next" }, … ]
}
```

- At most 4 MiB, and 1 to 64 nodes (`doc.node_count`).
- Versions 1 to 8 are migrated on opening. A file from before version 8 opens
  with `cookies` off, so it runs as it did; the other settings each version
  added (parameters in 2, profiles in 3, retries in 4, repeats and loops in 5,
  emulators in 6, impairments in 7, WebSocket and HTTP authentication in 8,
  load in 9) start empty.
- A newer version than this Signal Lab knows is refused
  (`doc.version_unsupported`) rather than opened without what it cannot read.
- A file that does not parse is reported with its path, line and column, and
  never replaced.
- It is written to a temporary file and renamed, so a failed write leaves the
  previous one.

What the nodes, parameters and profiles are: [experiments](../experiments/index.md),
[nodes](../experiments/nodes.md), [data](../experiments/data.md).

### signals.json {#signals-json}

The signal library, version 2:

```json
{
  "version": 2,
  "signals": [
    {
      "id": "…",
      "name": "Go cue",
      "group": "Stage/Cues",
      "note": "",
      "body": { "transport": "osc", "target": "127.0.0.1:9000", "address": "/cue/go", "args": [ { "type": "int", "value": 1 } ] }
    }
  ],
  "folders": [ "Stage", "Stage/Cues" ]
}
```

- `group` is the signal's folder as a `/` path; empty is the top level.
  `folders` (added in version 2) lists every folder, empty ones included,
  and is left out when there are none. A version 1 file reads the same,
  without empty folders.
- `body` is one of `osc`, `udp`, `http` or `mqtt`; their fields are in
  [`signals_save`](../api/commands.md#signals_save).
- When the file does not exist, the starter set is written — every target on
  `127.0.0.1` — and renamed into the interface's language.
- A file that does not parse is reported with its path, line and column
  (`signals.json_invalid`) and never replaced with the starter set: fix or
  delete it. Nothing writes the library while it does not read — a save is
  refused with the same error and the file stays as it is — until
  [[ui:sig.reload]] reads it again. A save goes through a temporary file in the
  same folder, so a write cut short leaves the previous file.

### emulators.json {#emulators-json}

The emulator library, version 1:

```json
{
  "version": 1,
  "emulators": [
    { "id": "demo-api", "note": "…", "emulator": { "name": "Demo API", "bind": "127.0.0.1:8080", "protocol": "http", "routes": [ … ] } }
  ]
}
```

Each entry is an emulator document with an `id` and a `note`; the document is
described in [emulators](../tools/emulators.md). As with the signals, a missing
file gets the starter set (every one bound to `127.0.0.1`), and a broken one
is reported (`emulators.json_invalid`), never replaced. It is written through
a temporary file. `signallab emulate` also reads a file of its own that holds
one emulator, a list of them, or a library like this one.

### Run reports {#run-reports}

`runs/run-<started ms>-<job>.json`, report version 5: one file per run that
passed or failed, never written over (a second run with the same name gets
`-2`, `-3`… added). A run that was stopped saves none.

| Field | What it is |
| --- | --- |
| `version` | 5 |
| `experiment` | The experiment's name |
| `document_version` | The version of the document that ran |
| `seed`, `profile` | What it ran with |
| `overrides` | Values given for this run only |
| `params` | Every parameter value it used |
| `started_ms`, `ended_ms` | Milliseconds since 1970 |
| `outcome` | `passed` or `failed` |
| `error` | Its first failure, or null |
| `steps` | Every step, as [`experiment://step`](../api/events.md#event-experiment-step) |
| `emulators` | What each [[ui:exp.node.emulator]] node received and answered (since version 3); left out when none |
| `impairments` | What each [[ui:exp.node.impairment]] node's relay did, phase by phase (since version 4); left out when none |

A load step's measurements are on its last step (since version 5). The
timeline's run history and [[ui:exp.compare]] read these files; a report that
cannot be read is left out of the list. See
[runs and reports](../experiments/runs.md).

### Exports {#exports}

- `exports/experiment-…json`: the experiment document, as above. Every export
  is a new file.
- `capture-….jsonl`: one Inspector frame per line, with the bytes it keeps in
  `data`, base64.
- `capture-….txt`: the frames for reading, each with a hex dump.

On a server, the Inspector's export downloads to your computer as it is made;
an experiment's export offers [[ui:common.download]], and a run's
[[ui:exp.reportSaved]] in the timeline is a link that downloads its report.

## Secrets are not in these files {#secrets}

An experiment names a secret — `{{secret.API_TOKEN}}` — and only the name is
written. The value is kept:

| Where Signal Lab runs | Where secret values are |
| --- | --- |
| Desktop app, Windows | Windows Credential Manager, under `SignalLab` ([[ui:exp.secrets]] in the editor) |
| Desktop app, Linux | Nowhere: secrets cannot be stored (`secret.unsupported`) |
| Server | Read-only: the environment variable `SIGNALLAB_SECRET_<NAME>`, or the file `<NAME>` in `--secrets-dir` (default `/run/secrets/signallab`) |
| `signallab` | The same files and variables, or the system's store with `--secrets system` |

::: warning
What you type straight into a field is kept as you typed it. A password in an
HTTP signal's credentials, an MQTT broker emulator's password, a token pasted
into a header — all are plain text in `signals.json`, `emulators.json` or
`experiment.json`, and in their exports. Use `{{secret.NAME}}` in an
experiment for anything you would not put in a shared folder.
:::

## Interface settings {#settings}

What the interface remembers — its language, the values last typed on each
screen, which pane is open and how large, [[ui:http.keepCookies]], when updates
were last looked for, and the install's random number for updates — is kept by
the interface itself, not in the data folder: in the app's own storage on the
desktop, in the browser's site storage for a server's page (per browser). The
[[ui:nav.http]] screen's credentials are not kept there.

Signal Lab writes no log files; see
[troubleshooting](troubleshooting.md#logs).

## Backing up, editing, moving {#backup}

- **Back up** by copying the whole folder. Everything in it is self-contained
  JSON; secret values are not in it, so set them again on a new machine.
- **Edit** `signals.json`, `emulators.json` and `experiment.json` by hand while
  Signal Lab is closed (or, on a server, while no page is open): the app writes
  the whole file from what it holds, so a change made while it runs is
  overwritten by its next save. A mistake is reported with the line and column
  when the file is next read, never silently replaced — for `signals.json`,
  also when the app saves into it: that save is refused and the file stays as
  you left it.
- **Delete** `runs/`, `exports/` and `capture-*` files at any time. Deleting
  `signals.json` or `emulators.json` brings the starter set back; deleting
  `experiment.json` brings the starter experiment back.
- **Move** the folder by copying it and pointing Signal Lab at the new place:
  `--data-dir` for a server, `SIGNALLAB_DATA_DIR` for the desktop app.
- **Share** an experiment by exporting it, or by committing its JSON next to the
  project it tests; [`signallab run`](../automation/cli.md#cli-run) runs it
  from there.
