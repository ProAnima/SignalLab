---
title: Runs
description: POST /api/run runs an experiment on a Signal Lab server and answers with its result, or streams its steps as NDJSON lines.
---

# Running an experiment

To run an experiment on a server from a script or a pipeline and know how it
went, send it to `POST /api/run`. The server runs it to its end and answers
with the result — or, if you ask for it, with each step as it happens. This
is what [`signallab run --server`](../automation/cli.md#cli-run) uses.

It is the same run as the editor's [[ui:exp.run]] and
[`experiment_start`](commands.md#experiment_start): a job that every open page
sees and can stop, the same events, the same report in the data folder.

## The request {#request}

```http
POST /api/run
Authorization: Bearer <token>
Content-Type: application/json

{ "template": "osc-ping-reply", "overrides": { "device": "192.0.2.20:9000" }, "seed": 42, "timeout": 30 }
```

The body names one experiment — a `document` or a `template`, not both — and
what to run it with:

| Field | Type | Default | Meaning |
| --- | --- | --- | --- |
| `document` | object | — | An experiment as the editor saves and exports it. Older versions are migrated, as when a file is opened |
| `template` | string | — | A bundled template by file name, with or without `.json` (below) |
| `overrides` | object | `{}` | Parameter values for this run only. Values may be strings, numbers or booleans; each must be a parameter of the experiment |
| `profile` | string | the document's | Run with this profile; `""` runs with the defaults |
| `seed` | number | the document's, else a new one | 0 to 9007199254740991; the same seed draws the same random values |
| `timeout` | number | `300` | Seconds before the run fails with `run.timeout`; 1 to 300 |

A field the server does not know is refused (`400`, `api.run_invalid`). The
document itself is read as an opened file is, so it is at most 4 MiB.

The bundled templates — the experiments the editor offers under [[ui:exp.templates]]:

| `template` | What it is |
| --- | --- |
| `empty` | Start and End |
| `http-check` | A GET of `http://127.0.0.1:8080/`, then a check for status 200 |
| `status-branch` | A GET of `http://127.0.0.1:8080/`; on 200 an OSC message, otherwise a 500 ms delay |
| `parallel-flows` | A [[ui:exp.node.fork]] into a GET of `http://127.0.0.1:8080/` and a log entry, side by side, then [[ui:exp.node.join]] |
| `osc-ping-reply` | An OSC `/ping` to the parameter `device` (`127.0.0.1:9000`), then a wait of 2 s for `/pong` on `127.0.0.1:9001` |
| `poll-until-ready` | A [[ui:exp.node.loop]] that asks `device` for `/status` over OSC until it answers `ready`, at most 10 times |
| `flaky-api` | An emulated API (parameter `api`) that fails before it works, asked in a [[ui:exp.node.loop]] until it answers 200 |
| `fault-phases` | A UDP device behind an impairment relay, sent to for 8 s while the relay goes clean, lossy, offline and clean again |
| `dependency-outage` | An emulated API (parameter `api`) taken down for 2 s while a [[ui:exp.node.loop]] asks it until it answers 200 again |
| `websocket-echo` | A connection to the parameter `service` (`ws://127.0.0.1:9001/echo`), a message, a wait for its echo, a check, a close |

Open one under [[ui:exp.templates]] to see its nodes and parameters; see
[experiments](../experiments/index.md).

## The result {#result}

By default the answer is `200` with `Content-Type: application/json`, sent
when the run has ended: one JSON object, the run's result.

```json
{
  "job_id": 12,
  "experiment": "OSC ping → reply",
  "outcome": "passed",
  "seed": 42,
  "profile": null,
  "overridden": true,
  "params": { "device": "192.0.2.20:9000" },
  "started_ms": 1759600000000,
  "ended_ms": 1759600000310,
  "steps": [ { "job_id": 12, "ts": 1759600000001, "node_id": "start", "state": "running", "detail": "", "message_key": null, "message_params": null }, … ],
  "report_path": "/data/runs/run-1759600000000-12.json"
}
```

| Field | Type | Meaning |
| --- | --- | --- |
| `job_id` | number | The run's job |
| `experiment` | string | The experiment's name |
| `outcome` | string | `passed`, `failed` or `stopped` |
| `seed` | number | The seed it ran with: give it back as `seed` to draw the same values |
| `profile` | string or null | The profile it ran with |
| `overridden` | boolean | Some values came from `overrides` |
| `params` | object | Every parameter value the run used |
| `started_ms`, `ended_ms` | number | Milliseconds since 1970 |
| `error` | `EngineError` | Why it failed: its first failure. Left out when it passed |
| `steps` | object[] | Every step in the order it happened, as [`experiment://step`](events.md#event-experiment-step) |
| `emulators` | object[] | What each [[ui:exp.node.emulator]] node received and answered: `node`, `name`, `protocol`, `local`, `counts`. Left out when there are none |
| `impairments` | object[] | What each [[ui:exp.node.impairment]] node's relay did, phase by phase. Left out when there are none |
| `report_path` | string | The run's report on the server; download it with [`/api/files`](index.md#files). Left out when none was written |
| `report_error` | `EngineError` | Why the report could not be written. Left out otherwise |

Secret values are masked in all of it. The report file holds the same steps;
see [runs and reports](../experiments/runs.md).

While the run goes on, the server sends a space every 15 s. JSON ignores
whitespace before a value, so the result still parses, and a proxy does not
take a long, quiet run for a dead connection.

## Following the steps {#lines}

To see the steps as they happen, ask for NDJSON:

```http
Accept: application/x-ndjson
```

The answer is `200` with `Content-Type: application/x-ndjson`: one JSON object
per line, each with a `type`.

| `type` | When | The rest of the line |
| --- | --- | --- |
| `started` | First, once | `job_id`, `experiment`, `seed`, `profile`, `overridden`, `started_ms` |
| `step` | Each step | The step, as [`experiment://step`](events.md#event-experiment-step) |
| `heartbeat` | Every 15 s | Nothing |
| `ended` | Last, once | The result, as above |

```text
{"job_id":12,"experiment":"OSC ping → reply","seed":42,"profile":null,"overridden":true,"started_ms":1759600000000,"type":"started"}
{"job_id":12,"ts":1759600000001,"node_id":"start","state":"running","detail":"","message_key":null,"message_params":null,"type":"step"}
…
{"job_id":12,"experiment":"OSC ping → reply","outcome":"passed",…,"type":"ended"}
```

Read the lines until `ended`; ignore a `type` you do not know. The server
sends `X-Accel-Buffering: no`, so an nginx proxy passes each line on at once.

## Status and outcome {#status}

An HTTP error status means **no run started**; the body is an
[`EngineError`](index.md#errors):

| Status | Code | Why |
| --- | --- | --- |
| `400` | `api.run_invalid` | The body is not a run request: not JSON, an unknown field, an override that is not a string, number or boolean |
| `400` | `api.run_source` | Neither `document` nor `template`, or both |
| `415` | `command.json_required` | Not `Content-Type: application/json` |
| `422` | `api.template_unknown` | No bundled template of that name |
| `422` | `file.json_invalid`, `file.too_large`, `doc.*` | The document cannot be read |
| `422` | any validation code, `run.override_unknown`, `profile.active_missing`, `run.limit_range`, `seed.range`, `secret.missing`, `transport.address_in_use`… | The experiment cannot start: it does not validate, a value is out of range, a secret is not stored, a port it listens on is taken |

Once the run has started the status is `200`, whatever happens: read
`outcome` in the result.

| `outcome` | Meaning |
| --- | --- |
| `passed` | Every step passed and [[ui:exp.node.end]] was reached |
| `failed` | A step failed, or the run took longer than its `timeout` (`run.timeout`); `error` says which and why |
| `stopped` | It was stopped before it ended: by `job_stop`, [[ui:app.stopAll]], or the server shutting down. `steps` has the steps it got to; no report is saved |

## A client that goes away {#disconnect}

Closing the connection does not stop the run. It is a job on the server: it
runs to its end and saves its report, as a run started in a browser does when
the tab is closed. Find it with [`jobs_list`](commands.md#jobs_list), stop it
with [`job_stop`](commands.md#job_stop), and read its report afterwards with
[`experiment_runs`](commands.md#experiment_runs). When the server shuts down,
the run is stopped and a client still connected gets `"outcome": "stopped"`.

## Examples {#examples}

Run a bundled template and wait for the outcome:

```bash
SERVER=http://127.0.0.1:1430
TOKEN=$(cat token.txt)
curl -sS -X POST "$SERVER/api/run" \
  -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
  -d '{"template":"osc-ping-reply","overrides":{"device":"192.0.2.20:9000"},"timeout":30}' \
  | jq -r .outcome
```

Send your own experiment with a parameter changed, and print each step as it
happens:

```bash
jq '{document: ., overrides: {api: "http://192.0.2.10:8080"}, profile: ""}' smoke.json |
  curl -sSN -X POST "$SERVER/api/run" \
    -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
    -H "Accept: application/x-ndjson" --data @- |
  jq -r 'select(.type == "step") | "\(.node_id)  \(.state)  \(.detail)"'
```

`-N` keeps `curl` from holding the lines back. To fail a pipeline on a failed
run, check `outcome`:

```bash
outcome=$(curl -sS -X POST "$SERVER/api/run" -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" -d '{"template":"http-check"}' | jq -r .outcome)
[ "$outcome" = "passed" ]
```

## From the command line {#cli}

[`signallab run`](../automation/cli.md#cli-run) with `--server <url>` runs on a
server through this endpoint: it sends the experiment it read, with
`overrides`, `seed` and `timeout`, asks for NDJSON, and prints each step as its
line arrives. The token comes from `--token-file`, else `SIGNALLAB_TOKEN`.
With `--report` it downloads the report through `/api/files`. A server that
sends nothing for 60 s — not even a heartbeat — counts as gone.
