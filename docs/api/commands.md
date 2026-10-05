---
title: Commands
description: Every command of Signal Lab's engine, called as POST /api/invoke/<command>, with its arguments, result and errors.
---

# Commands

Every command the engine has, grouped by what it works on. Each is called as
`POST /api/invoke/<command>` with a JSON object of arguments, and answers `200`
with its result or `422` with an [`EngineError`](index.md#errors). How to
authenticate and what the statuses mean is in [the API overview](index.md).

## Conventions {#conventions}

- **Argument names** are camelCase (`jobId`, `nodeId`). An argument a command
  does not know, or a required one left out, is refused with
  `command.args_invalid`; every command that takes arguments can fail with it.
  A command without arguments does not read the body.
- **Objects passed as an argument** — `config`, `request`, `document`,
  `library`, `emulator`, `profile` — use the engine's own field names, mostly
  snake_case (`timeout_ms`). Inside them a field the engine does not know is
  **ignored**, so a misspelled optional field quietly keeps its default. Only
  `feedback_send`'s form refuses unknown fields.
- **Optional** arguments and fields may be left out or sent as `null`; the
  tables give their defaults.
- **Results** are JSON. "null" means the command has nothing to return.
- **Addresses** written `IP:port` take a numeric address and a port
  (`127.0.0.1:9000`, `[::1]:9000`); a host name there is refused. Where a table
  says `IP:port` or `host:port`, a host name works too: it is
  looked up when the command runs, and its IPv4 address is used when it has one
  (so `localhost:9000` is `127.0.0.1:9000`).
- **Jobs**: a command marked *Starts a job* returns a
  [`JobInfo`](#type-jobinfo); the work goes on until it ends or is stopped with
  [`job_stop`](#job_stop). See [jobs](index.md#jobs).
- **Paths** in results are on the machine where the engine runs — on a server,
  inside its data folder; download them with [`/api/files`](index.md#files).

The examples use this shell function:

```bash
SERVER=http://127.0.0.1:1430
TOKEN=$(cat token.txt)
invoke() {
  curl -sS -X POST "$SERVER/api/invoke/$1" \
    -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
    --data "${2:-}"
}
```

## Application {#application}

### app_info {#app_info}

What the engine is and where it runs. No arguments.

**Result**

| Field | Type | Meaning |
| --- | --- | --- |
| `version` | string | Signal Lab's version, `[[version]]` |
| `mode` | string | `desktop` or `server` |
| `secrets_writable` | boolean | Whether [`secret_set`](#secret_set) and [`secret_delete`](#secret_delete) can work here: `false` on a server |
| `data_dir` | string | The data folder, on the machine the engine runs on |
| `os` | string | `windows`, `linux`… |
| `arch` | string | `x86_64`, `aarch64`… |

### get_host_info {#get_host_info}

The machine's name and the address it would send from. No arguments.

**Result**: `{ "local_ip": string, "hostname": string }`. `local_ip` is the
IPv4 address the system picks for traffic to the internet (found without
sending anything), or `127.0.0.1` when there is none. `hostname` is the
computer's name, or `localhost` when the system does not say.

### firewall_status {#firewall_status}

Whether the system's firewall lets other machines reach this program. Only
Windows has a per-program firewall to read; elsewhere `applies` is `false`, and
of the rest only `program` is filled in. No arguments.

**Result**

| Field | Type | Meaning |
| --- | --- | --- |
| `applies` | boolean | There is a per-program firewall here (Windows) |
| `program` | string | The program the rules are about |
| `enabled` | boolean | The firewall is on for the network the machine is on now |
| `networks` | string[] | The kinds of network the machine is on: `domain`, `private`, `public` |
| `allowed` | boolean | An inbound rule lets UDP in for this program on the current network |
| `blocked` | boolean | An inbound rule blocks this program on the current network; it wins over any allow rule |
| `rules` | number | Inbound rules for this program, of any kind |

**Errors**: `firewall.failed`.

### firewall_allow {#firewall_allow}

Lets other machines reach Signal Lab: the system shows its own administrator
prompt, then the program's inbound rules (a block rule included) are replaced
by one allow rule each for Signal Lab and the `signallab` command line next to
it. Desktop app on Windows only; a server refuses, since nobody is at its
screen to answer the prompt.

| Argument | Type | Required | Meaning |
| --- | --- | --- | --- |
| `public` | boolean | yes | Allow on public networks too, not only private and domain ones |

**Result**: the new [`firewall_status`](#firewall_status).

**Errors**: `firewall.server` (on a server), `firewall.unsupported` (not
Windows), `firewall.declined` (the prompt was answered No), `firewall.failed`.

### feedback_send {#feedback_send}

Sends a message to Signal Lab's developers, through the studio's hub, which
mails it to them.

| Argument | Type | Required | Meaning |
| --- | --- | --- | --- |
| `form` | object | yes | The message, below. Unknown fields are refused |

| Field of `form` | Type | Default | Meaning |
| --- | --- | --- | --- |
| `message` | string | — | What happened; required, at most 20 000 characters |
| `email` | string | none | Where an answer may go |
| `meta` | object of strings | `{}` | What the app says about itself (version, os, arch, mode, lang, screen) |
| `screenshots` | `{ name, data }[]` | `[]` | Images, `data` in base64; at most 6, 8 MiB each |
| `logs` | `{ name, text }[]` | `[]` | Text files; at most 4, 2 MiB each |

Everything together is at most 15 MiB.

**Result**: `{ "id": string }`, the reference the developers get.

**Errors**: `feedback.message_required`, `feedback.message_too_long`,
`feedback.too_many_files`, `feedback.file_too_large`, `feedback.too_large`,
`feedback.invalid`, the hub's refusals (`feedback.email_invalid`,
`feedback.file_type`, `feedback.rate_limited`, `feedback.disabled`,
`feedback.send_failed`, `feedback.failed`), and the network's `transport.*`.

```bash
invoke app_info
# {"version":"[[version]]","mode":"server","secrets_writable":false,"data_dir":"/data","os":"linux","arch":"x86_64"}
```

## Jobs {#jobs}

### jobs_list {#jobs_list}

The running jobs, oldest first. No arguments.

**Result**: [`JobInfo`](#type-jobinfo)`[]`.

### job_stop {#job_stop}

Stops one job at once: its sockets close, its relay, server or connection goes.
A stopped job sends no [`job://ended`](events.md#event-job-ended); a stopped
run saves no report.

| Argument | Type | Required | Meaning |
| --- | --- | --- | --- |
| `id` | number | yes | The job's `id` |

**Result**: `true` when a job with that id was running, `false` otherwise.

### jobs_stop_all {#jobs_stop_all}

Stops every running job, whoever started it. No arguments.

**Result**: null.

```bash
invoke jobs_list
# [{"id":3,"kind":"osc-monitor","label":"OSC monitor 0.0.0.0:9000","params":{"bind":"0.0.0.0:9000"},"started_ms":1759600000000}]
invoke job_stop '{"id":3}'
# true
```

## Experiments and runs {#experiments}

These commands take and return an experiment document (`Experiment`): the JSON
the editor saves and exports, with `version`, `name`, `params`, `profiles`,
`profile`, `seed`, `cookies`, `nodes` and `edges`. Its nodes are in
[nodes](../experiments/nodes.md); its parameters, profiles and templates in
[data](../experiments/data.md). A document is at most 4 MiB
(`file.too_large`). To run an experiment and wait for its result, use
[`POST /api/run`](run.md) rather than [`experiment_start`](#experiment_start).

### experiment_load {#experiment_load}

The working experiment: `experiment.json` in the data folder — on a server, the
one its interface shows. When there is none, the starter experiment. Older
document versions are migrated. No arguments.

**Result**: `Experiment`.

**Errors**: `file.io`, `file.json_invalid` (with the file's `path`, `line`
and `column`), `file.too_large`, `doc.version_unsupported` and the other
`doc.*` checks.

### experiment_save {#experiment_save}

Replaces the working experiment, `experiment.json` in the data folder. It is
written to a temporary file first, so a failed write leaves the previous one.

::: warning
On a server this is the document every browser's editor works on.
:::

| Argument | Type | Required | Meaning |
| --- | --- | --- | --- |
| `document` | `Experiment` | yes | The document |

**Result**: string, the path written.

**Errors**: `doc.*`, the document's size checks (`param.*`, `params.too_many`,
`profile.*`, `profiles.too_many`, `seed.range`), `file.too_large`, `file.io`.

### experiment_parse {#experiment_parse}

Reads an experiment from JSON text, as [[ui:exp.importJson]] does. Versions 1
to 8 are migrated to version 9, the current one; a file from before version 8
opens with `cookies` off, so it runs as it did. A byte-order mark is skipped.

| Argument | Type | Required | Meaning |
| --- | --- | --- | --- |
| `text` | string | yes | The file's text |

**Result**: `Experiment`.

**Errors**: `file.json_invalid` (`line`, `column`), `file.too_large`,
`doc.version_unsupported`, `doc.*`, the size checks of
[`experiment_save`](#experiment_save).

### experiment_export {#experiment_export}

Writes a snapshot of a document to `exports/experiment-<ms>-<16 hex digits>.json`
in the data folder. Every export is a new file.

| Argument | Type | Required | Meaning |
| --- | --- | --- | --- |
| `document` | `Experiment` | yes | The document |

**Result**: string, the path written.

**Errors**: those of [`experiment_save`](#experiment_save).

### experiment_validate {#experiment_validate}

Checks that a document would run with its active profile (or its defaults):
the graph, every field, parameters, and that every secret it names is stored.
A blocking problem is the error. On success, it says which of the *other*
profiles would fail, so you know before switching.

| Argument | Type | Required | Meaning |
| --- | --- | --- | --- |
| `document` | `Experiment` | yes | The document |
| `overrides` | object of strings | no | Parameter values for this check only, as [[ui:exp.runWith]] gives them |

**Result**: `{ "profile": string or null, "error": EngineError }[]` — each
other profile that would not validate (`null`: the defaults, without a
profile). An empty list means every profile is fine.

**Errors**: any validation code (`doc.*`, `graph.*`, `node.*`, `param.*`,
`profile.*`, `template.*`, `loop.*`…), `run.override_unknown` (an override
for a parameter the document does not have), `secret.missing`,
`secret.store`, `secret.unsupported`.

### experiment_resolve {#experiment_resolve}

One node with its templates filled in, as the editor's preview shows it: the
active profile's values and the variable values you give. Secrets are shown
as `••••`, never their values. Names that have no value stay as written and
are listed.

| Argument | Type | Required | Meaning |
| --- | --- | --- | --- |
| `document` | `Experiment` | yes | The document |
| `nodeId` | string | yes | The node |
| `vars` | object | yes | Variable values to use, by name; `{}` for none |

**Result**: `{ "node": node, "missing": string[] }`.

**Errors**: `node.not_found`, `template.*`, `secret.store`,
`secret.unsupported`.

### experiment_send_node {#experiment_send_node}

[[ui:exp.sendNow]]: performs one node on its own, through the same code a run
uses. An action is sent; a wait listens from now on until it matches or times
out. A [[ui:exp.node.ws_send]] or [[ui:exp.node.wait_ws]] node opens the
connection its [[ui:exp.node.ws_connect]] node describes. Nothing is sent with
cookies, and the run's [[ui:exp.node.impairment]] and [[ui:exp.node.emulator]]
nodes are not opened.

| Argument | Type | Required | Meaning |
| --- | --- | --- | --- |
| `document` | `Experiment` | yes | The document; its active profile gives the parameter values |
| `nodeId` | string | yes | An action or a wait |
| `vars` | object | yes | Variable values the node's templates read; `{}` for none |

**Result**

| Field | Type | Meaning |
| --- | --- | --- |
| `detail` | string | What happened, in English |
| `response` | [`HttpResponse`](#type-httpresponse) or null | An HTTP node's response |
| `vars` | object | What the step set: a wait's reply, or what the [[ui:exp.node.extract]] nodes after a request take from its response |

Secret values are masked in all of it.

**Errors**: `node.not_found`, `run.not_an_action` (not an action or a
wait), `ws.connection_unknown`, `secret.missing`, `template.*`, and whatever the
step fails with: `transport.*`, `wait.timeout`, `check.*`…

### experiment_start {#experiment_start}

Starts a run, as [[ui:exp.run]] does, and returns at once. Its steps arrive as
[`experiment://step`](events.md#event-experiment-step) events, its end as
[`experiment://ended`](events.md#event-experiment-ended), and its report is
saved under `runs/` in the data folder. Waits, emulators, impairment relays and
MQTT subscriptions open before the first step, so a port that is taken fails
here. A run longer than 300 s fails with `run.timeout`. *Starts a job*
(`experiment`).

| Argument | Type | Required | Meaning |
| --- | --- | --- | --- |
| `document` | `Experiment` | yes | The document |
| `overrides` | object of strings | no | Parameter values for this run only |
| `seed` | number | no | The run's seed, 0 to 9007199254740991; default: the document's, else a new one |

**Result**: [`JobInfo`](#type-jobinfo), `params.name` the experiment's name.

**Errors**: everything [`experiment_validate`](#experiment_validate) reports,
`seed.range`, `transport.address_in_use` and the other bind failures,
`emulator.*`, `impair.*`, `node.params_only` (a listen address or an MQTT
wait's broker or topic that is not fixed when the run starts), and the
[`mqtt_connect`](#mqtt_connect) errors of a broker an MQTT wait cannot reach.

### experiment_runs {#experiment_runs}

Runs read back from their reports in `runs/`, newest first. A report that
cannot be read is left out.

| Argument | Type | Required | Meaning |
| --- | --- | --- | --- |
| `name` | string | no | Only runs of the experiment with this exact name |
| `limit` | number | no | At most this many; default 50, at most 500 |

**Result**: run summaries:

| Field | Type | Meaning |
| --- | --- | --- |
| `name` | string | The report's file name, `run-<ms>-<job>.json`: what [`experiment_compare`](#experiment_compare) takes |
| `experiment` | string | The experiment's name |
| `started_ms`, `ended_ms` | number | Milliseconds since 1970 |
| `outcome` | string | `passed` or `failed` |
| `seed` | number | The run's seed |
| `profile` | string or null | Its profile |
| `loads` | object[] | Each load step: `node`, `sent`, `rps`, `p95_ms`, `error_rate`, `held` (every threshold held) |

**Errors**: `file.io`.

### experiment_compare {#experiment_compare}

Two runs side by side, load step by load step, as the timeline's
[[ui:exp.compare]] shows them. Steps are matched by node id.

| Argument | Type | Required | Meaning |
| --- | --- | --- | --- |
| `a` | string | yes | The earlier run's report file name |
| `b` | string | yes | The later run's report file name |

**Result**: `{ "a": summary, "b": summary, "steps": [...] }`, each step with
`node`, `missing_in` (`a` or `b`, when only one run has it), `metrics`,
`sent` (`[a, b]`), `thresholds_a` and `thresholds_b` (each threshold as
`{ metric, op, value, actual, held }`). `metrics` lists nine metrics, each as
`{ metric, a, b, change, percent, worse }`: `metric` is `p50_ms`, `p90_ms`,
`p95_ms`, `p99_ms`, `mean_ms`, `max_ms`, `error_rate`, `rps` or `missed`;
`change` is `b − a`; `percent` the change in % of `a` (null when `a` is 0);
`worse` that it moved the wrong way — higher, or lower for `rps` — by 5 % or
more, or from 0 to anything. A step only one run has is never `worse`. See
[load](../experiments/load.md).

**Errors**: `runs.name_invalid` (anything but a report's file name, no folders),
`runs.not_found`, `file.json_invalid`, `file.io`.

```bash
invoke experiment_validate "$(jq '{document: .}' experiment.json)"
# []
```

## Secrets {#secrets}

Secret values are used by experiments as `{{secret.NAME}}` and never leave the
engine: no command returns one. Where they are kept depends on where the engine
runs:

| Where | Store | Setting and removing |
| --- | --- | --- |
| Desktop app, Windows | Windows Credential Manager | Yes |
| Desktop app, Linux | None | `secret.unsupported` |
| Server | `SIGNALLAB_SECRET_<NAME>`, or the file `<NAME>` in `--secrets-dir` (default `/run/secrets/signallab`) | No: `secret.read_only` |

A name starts with a Latin letter or `_`, goes on with Latin letters, digits
and `_`, and has at most 128 characters (`secret.name_invalid`).

### secret_status {#secret_status}

Which of the given names have a stored value.

| Argument | Type | Required | Meaning |
| --- | --- | --- | --- |
| `names` | string[] | yes | The names to look up |

**Result**: an object, name → `true` (stored) or `false`.

**Errors**: `secret.name_invalid` (on a server), `secret.store`,
`secret.too_large` (a server's file over 16 KiB), `secret.unsupported`.

### secret_set {#secret_set}

Stores a value under a name, replacing the one there. Desktop app only.

| Argument | Type | Required | Meaning |
| --- | --- | --- | --- |
| `name` | string | yes | The name |
| `value` | string | yes | Not empty; at most 16 KiB |

**Result**: null.

**Errors**: `secret.read_only` (on a server), `secret.unsupported`,
`secret.name_invalid`, `secret.empty`, `secret.too_large`, `secret.store`.

### secret_delete {#secret_delete}

Removes a stored value. Removing one that is not stored is not an error.
Desktop app only.

| Argument | Type | Required | Meaning |
| --- | --- | --- | --- |
| `name` | string | yes | The name |

**Result**: null.

**Errors**: `secret.read_only` (on a server), `secret.unsupported`,
`secret.name_invalid`, `secret.store`.

```bash
invoke secret_status '{"names":["API_TOKEN","MQTT_PASSWORD"]}'
# {"API_TOKEN":true,"MQTT_PASSWORD":false}
```

## OSC {#osc}

See [OSC](../protocols/osc.md) for the screen these commands serve.

### osc_send {#osc_send}

Sends one OSC message in one UDP datagram, from a fresh socket.

| Argument | Type | Required | Meaning |
| --- | --- | --- | --- |
| `target` | string | yes | `IP:port` or `host:port` to send to; a name is looked up, its IPv4 address taken when it has one |
| `address` | string | yes | The OSC address, `/mixer/fader/1`; it starts with `/` |
| `args` | [`OscArg`](#type-oscarg)`[]` | yes | The arguments; `[]` for none |

**Result**: number, the bytes sent.

**Errors**: `node.osc_address` (no leading `/`; field `address`),
`transport.target_invalid` (no port, or neither form), `transport.dns` (the
name does not resolve), `transport.*`.

### osc_monitor_start {#osc_monitor_start}

Listens for OSC on a UDP port and decodes every packet. Each one arrives as an
[`osc://message`](events.md#event-osc-message) event. *Starts a job*
(`osc-monitor`, `params.bind`).

| Argument | Type | Required | Meaning |
| --- | --- | --- | --- |
| `bind` | string | yes | `IP:port` to listen on: `0.0.0.0:9000` every network card, `127.0.0.1:9000` this machine only |

**Result**: [`JobInfo`](#type-jobinfo).

**Errors**: `node.bind_invalid`, `transport.address_in_use`,
`transport.address_unavailable`, `transport.denied`, `wait.bind_failed`. The
job ends with `wait.receive_failed` if the socket can no longer receive.

### osc_generator_start {#osc_generator_start}

Sends a stream of OSC messages whose single argument follows a waveform.
Progress arrives as [`osc://gen-tick`](events.md#event-osc-gen-tick). *Starts
a job* (`osc-gen`, `params.target`, `params.address`).

| Argument | Type | Required | Meaning |
| --- | --- | --- | --- |
| `config` | object | yes | Below |

| Field of `config` | Type | Default | Meaning |
| --- | --- | --- | --- |
| `target` | string | — | `IP:port` or `host:port` to send to; a name is looked up once, when the job starts |
| `address` | string | — | The OSC address; it starts with `/` |
| `rate` | number | — | Messages per second, held between 0.1 and 5000 |
| `waveform` | string | — | `sine`, `triangle`, `saw` (falling: `max` to `min`, then back at once), `ramp` (rising: `min` to `max`, then back at once), `square`, `random` or `constant` (`max`) |
| `freq` | number | — | Cycles of the waveform per second |
| `min`, `max` | number | — | The value's range |
| `as_int` | boolean | `false` | Round and send an int instead of a float |
| `duration_s` | number | `0` | Stop after this many seconds; 0 runs until stopped |

**Result**: [`JobInfo`](#type-jobinfo).

**Errors**: `node.osc_address`, `transport.target_invalid`, `transport.dns`,
`transport.*`. The job ends with a `transport.*` error if a send fails.

```bash
invoke osc_send '{"target":"127.0.0.1:9000","address":"/cue/go","args":[{"type":"int","value":1}]}'
# 16
invoke osc_monitor_start '{"bind":"0.0.0.0:9000"}'
```

## HTTP and cookies {#http}

See [HTTP](../protocols/http.md).

### http_request {#http_request}

Sends one HTTP request and returns the response. A request that gets no
response — refused, timed out, a name that does not resolve, a certificate
that is not trusted — is **not** an error of the command: the response says
so in `error` and `cause`.

| Argument | Type | Required | Meaning |
| --- | --- | --- | --- |
| `request` | [`HttpRequest`](#type-httprequest) | yes | The request |
| `cookies` | boolean | no | Send the [[ui:nav.http]] screen's cookie jar and keep what the answer sets; default `false` |

**Result**: [`HttpResponse`](#type-httpresponse).

**Errors**: `http.client_failed` (the request could not even be prepared).

### http_burst_start {#http_burst_start}

Sends one request many times, several at once, and measures it. Without a
`rate`, each worker sends again as soon as it has an answer; with one,
requests start on a fixed schedule however slow the answers are, and a request
that waited more than 50 ms past its moment for a free worker is skipped and
counted as missed. Progress arrives as
[`http://burst-progress`](events.md#event-http-burst-progress) ten times a
second. *Starts a job* (`http-burst`, `params.method`, `params.url`, and
`params.rate` when paced).

| Argument | Type | Required | Meaning |
| --- | --- | --- | --- |
| `config` | object | yes | The fields of [`HttpRequest`](#type-httprequest) and the ones below, in one object |

| Field of `config` | Type | Default | Meaning |
| --- | --- | --- | --- |
| `concurrency` | number | — | At most this many in flight, held between 1 and 512 |
| `total` | number | `0` | Stop after this many requests; 0: no count |
| `duration_s` | number | `0` | Stop after this many seconds; 0: no time limit |
| `rate` | number | `0` | Requests started per second, 0.1 to 100 000; 0: as fast as answers come |
| `cookies` | boolean | `false` | Use the [[ui:nav.http]] screen's cookie jar |

With neither `total` nor `duration_s`, the burst runs until stopped.

**Result**: [`JobInfo`](#type-jobinfo).

**Errors**: `http.rate_invalid`, `http.duration_invalid`, `http.client_failed`.

### http_cookies {#http_cookies}

The [[ui:nav.http]] screen's cookie jar: every cookie that has not expired. On a server
there is one jar for every page and script. No arguments.

**Result**: cookies, each with `name`, `value`, `domain`, `host_only` (no
Domain attribute: only the host that set it gets it back), `path`, `expires`
(Unix seconds, null for a session cookie), `secure`, `http_only` and
`same_site` (string or null).

### http_cookies_clear {#http_cookies_clear}

Empties the [[ui:nav.http]] screen's cookie jar. No arguments.

**Result**: null.

```bash
invoke http_request '{"request":{"method":"GET","url":"http://127.0.0.1:8080/health","headers":[["Accept","application/json"]],"body":null,"timeout_ms":5000}}' \
  | jq '{status, latency_ms, body}'
```

## WebSocket {#websocket}

See [WebSocket](../protocols/websocket.md). A connection that `ws_connect`
opens is a job; the others name it by `jobId`.

### ws_connect {#ws_connect}

Opens a WebSocket and keeps it open. What arrives and what is sent comes as
[`ws://messages`](events.md#event-ws-messages) every 100 ms; the connection's
state as [`ws://state`](events.md#event-ws-state). *Starts a job*
(`websocket`, `params.url`).

| Argument | Type | Required | Meaning |
| --- | --- | --- | --- |
| `config` | [`WsConfig`](#type-wsconfig) | yes | Where and how to connect |

**Result**: [`JobInfo`](#type-jobinfo).

**Errors**: `ws.url_invalid`, `ws.header_invalid`, `ws.protocol_invalid`,
`ws.handshake_status` (the server answered the upgrade with another status),
`ws.subprotocol_refused`, `ws.handshake_failed`, `transport.*`.

### ws_send {#ws_send}

Sends one message on an open connection.

| Argument | Type | Required | Meaning |
| --- | --- | --- | --- |
| `jobId` | number | yes | The connection's job |
| `message` | object | yes | `{ "text": "…" }` for a text message, or `{ "hex": "de ad be ef" }` for a binary one; exactly one of them |

**Result**: number, the bytes sent.

**Errors**: `ws.not_connected`, `ws.payload_required` (neither or both),
`hex.invalid` (an empty `hex` too), `node.too_long` (over 16 MiB, field
`payload`; nothing is sent and the connection stays open), `ws.closed`, `transport.*` (`transport.timeout` when the server stopped
reading for 10 s).

### ws_close {#ws_close}

Closes a connection with a close handshake and waits up to 2 s for the server's
answer; the job then ends. Once a connection has ended, its job is gone and
closing it is `ws.not_connected`.

| Argument | Type | Required | Meaning |
| --- | --- | --- | --- |
| `jobId` | number | yes | The connection's job |
| `code` | number | no | 1000, or 3000 to 4999 for an application's own; default 1000 |
| `reason` | string | no | At most 123 bytes; default empty |

**Result**: `{ "code", "reason", "by", "error" }` — `by` is `client`,
`server` or `lost`; `code` is 1005 when the close carried none and 1006 when
there was no close frame.

**Errors**: `ws.close_code`, `node.too_long`, `ws.not_connected`.

### ws_exchange {#ws_exchange}

One exchange without a job: connect, send a message if given, wait for an
answer if asked to, close.

| Argument | Type | Required | Meaning |
| --- | --- | --- | --- |
| `config` | [`WsConfig`](#type-wsconfig) | yes | Where and how to connect |
| `message` | object | no | `{ "text" }` or `{ "hex" }`, as for [`ws_send`](#ws_send) |
| `expect` | object | no | What to wait for: `mode` (`any`, `contains`, `regex`, `hex`; default `any`), `pattern` (default empty), `timeout_ms` (default 2000) |

With `expect` and no `message`, the first matching message after connecting
counts — a greeting.

**Result**: `{ "handshake", "sent", "reply", "closed" }` — `handshake` is
`{ url, peer, local, protocol, ms }`; `sent` the bytes sent or null; `reply`
`{ kind, text, hex, bytes, json, ms }` or null (`json`: a text answer parsed,
else null; `ms`: since the send, or since connecting when nothing was sent);
`closed` as [`ws_close`](#ws_close) returns.

**Errors**: those of [`ws_connect`](#ws_connect) and [`ws_send`](#ws_send),
`wait.timeout` (with `ms`, `unmatched` and `target`), `regex.invalid` and
`hex.invalid` (a `pattern` that does not parse).

```bash
invoke ws_exchange '{"config":{"url":"ws://127.0.0.1:9001/"},"message":{"text":"{\"type\":\"ping\"}"},"expect":{"mode":"contains","pattern":"pong"}}' \
  | jq .reply.text
```

## MQTT {#mqtt}

MQTT 3.1.1 over plain TCP, QoS 0, 1 and 2. See [MQTT](../protocols/mqtt.md).

### mqtt_connect {#mqtt_connect}

Connects to a broker and keeps the connection. The command returns once the
broker has accepted the connection (CONNACK), so a wrong password or a closed
port is its error. Messages arrive as
[`mqtt://messages`](events.md#event-mqtt-messages) every 100 ms; state
changes as [`mqtt://state`](events.md#event-mqtt-state); completed QoS 1/2
publishes and unsubscribes as [`mqtt://ack`](events.md#event-mqtt-ack).
*Starts a job* (`mqtt`, `params.broker`, `params.client`).

| Argument | Type | Required | Meaning |
| --- | --- | --- | --- |
| `config` | [`MqttConfig`](#type-mqttconfig) | yes | The broker and how to connect |

**Result**: [`JobInfo`](#type-jobinfo).

**Errors**: `mqtt.client_id_required`, `transport.*` (refused, unreachable,
`dns`, `timeout` after 6 s), `mqtt.no_answer` (no CONNACK within 6 s),
`mqtt.protocol`, `mqtt.refused_protocol`, `mqtt.refused_client_id`,
`mqtt.refused_unavailable`, `mqtt.refused_credentials`,
`mqtt.refused_not_authorized`, `mqtt.refused`.

### mqtt_publish {#mqtt_publish}

Publishes on an open connection.

| Argument | Type | Required | Meaning |
| --- | --- | --- | --- |
| `jobId` | number | yes | The connection's job |
| `topic` | string | yes | The topic: not empty, and no `+` or `#` |
| `payload` | string | yes | The payload, sent as UTF-8 |
| `qos` | number | yes | 0, 1 or 2 (above 2 is sent as 2) |
| `retain` | boolean | yes | Ask the broker to keep it; an empty payload with `retain` clears a retained value |

**Result**: null. A QoS 1 or 2 publish is confirmed later by
[`mqtt://ack`](events.md#event-mqtt-ack).

**Errors**: `node.topic_wildcard` (field `topic`) and `mqtt.topic_required`
(field `topic`), as for [`mqtt_publish_once`](#mqtt_publish_once) — the command
refuses them before it looks for the connection; `mqtt.not_connected`.

### mqtt_subscribe {#mqtt_subscribe}

Subscribes an open connection to filters. What the broker grants arrives as
[`mqtt://state`](events.md#event-mqtt-state) with `state: "subscribed"`.

| Argument | Type | Required | Meaning |
| --- | --- | --- | --- |
| `jobId` | number | yes | The connection's job |
| `filters` | `{ filter, qos }[]` | yes | At least one; `qos` defaults to 0. `+` and `#` are wildcards |

**Result**: null.

**Errors**: `mqtt.filter_required`, `mqtt.not_connected`.

### mqtt_unsubscribe {#mqtt_unsubscribe}

Unsubscribes an open connection from filters.

| Argument | Type | Required | Meaning |
| --- | --- | --- | --- |
| `jobId` | number | yes | The connection's job |
| `filters` | string[] | yes | At least one |

**Result**: null. The broker's answer arrives as
[`mqtt://ack`](events.md#event-mqtt-ack) with `kind: "unsubscribed"`.

**Errors**: `mqtt.filter_required`, `mqtt.not_connected`.

### mqtt_publish_once {#mqtt_publish_once}

Connects, publishes one message, waits for the acknowledgement its QoS asks for
(up to 6 s), disconnects. It brings its own connection, under a client id of
its own — the first 12 characters of `client_id`, `-o` and a number — so it
never knocks a live connection with that id off the broker.

| Argument | Type | Required | Meaning |
| --- | --- | --- | --- |
| `config` | [`MqttConfig`](#type-mqttconfig) | yes | The broker; `subscribe` is not used |
| `topic` | string | yes | Not empty and without `+` or `#` |
| `payload` | string | yes | Sent as UTF-8 |
| `qos` | number | yes | 0, 1 or 2 (above 2 is sent as 2) |
| `retain` | boolean | yes | Ask the broker to keep it |

**Result**: string, a summary written by Signal Lab:
`<topic> → <broker> · <bytes> B · qos<n>`, with ` retained` when retained.

**Errors**: `mqtt.topic_required`, `node.topic_wildcard`, and those of
[`mqtt_connect`](#mqtt_connect) but `mqtt.client_id_required`: an empty
`client_id` is accepted here.

```bash
invoke mqtt_publish_once '{"config":{"host":"127.0.0.1","port":1883,"client_id":"lab"},"topic":"lab/lamp/set","payload":"ON","qos":1,"retain":false}'
# "lab/lamp/set → 127.0.0.1:1883 · 2 B · qos1"
```

## Broadcast, multicast and discovery {#broadcast}

See [broadcast and discovery](../protocols/broadcast.md).

::: danger
Broadcast and a sweep reach every host of a network segment. Send only on
networks you are responsible for.
:::

### broadcast_send {#broadcast_send}

Sends one datagram to each target, once.

| Argument | Type | Required | Meaning |
| --- | --- | --- | --- |
| `config` | object | yes | Below |

| Field of `config` | Type | Default | Meaning |
| --- | --- | --- | --- |
| `mode` | string | — | `list`, `broadcast`, `multicast` or `sweep` |
| `target` | string | — | By mode, below |
| `port` | number | `0` | The port, for `sweep` only |
| `payload` | [`Payload`](#type-payload) | — | What each datagram carries |
| `bind` | string | any | The local `IP:port` it is sent from; empty or null: `0.0.0.0:0` (`[::]:0` when every target is IPv6) |
| `ttl` | number | `1` | IP TTL, or the multicast hop limit; 1 to 255 |
| `multicast_loop` | boolean | `true` | Multicast comes back to this machine too |
| `rate`, `count`, `duration_s` | number | `0` | For [`broadcast_beacon_start`](#broadcast_beacon_start) only |

| `mode` | `target` |
| --- | --- |
| `list` | `IP:port` or `host:port` entries separated by commas, semicolons or new lines (not by spaces); a name is looked up, its IPv4 address taken when it has one |
| `broadcast` | `255.255.255.255:port`, or an address ending in `.255` with its port |
| `multicast` | A group from 224.0.0.0 to 239.255.255.255 with its port |
| `sweep` | A CIDR block, `192.0.2.0/24`: every usable host on `port`; at most 1024 hosts, so `/22` or narrower |

**Result**

| Field | Type | Meaning |
| --- | --- | --- |
| `targets` | number | Destinations |
| `packets`, `bytes` | number | What went out |
| `errors` | number | Datagrams that could not be sent |
| `resolved` | string[] | The first 8 destinations |
| `summary` | string | The payload in one line |
| `error` | `EngineError` | Why the first failed datagram failed; left out when none did |

**Errors**: `broadcast.target_required`, `broadcast.not_broadcast`,
`broadcast.ipv6`, `broadcast.not_multicast`, `broadcast.sweep_port`,
`broadcast.cidr_invalid`, `broadcast.prefix_invalid`,
`broadcast.sweep_too_large`, `node.osc_address` (an OSC address must start
with `/`), `hex.empty`, `hex.invalid`, `node.bind_invalid`,
`socket.option_failed`, `transport.target_invalid`, `transport.dns`, bind
failures.

### broadcast_beacon_start {#broadcast_beacon_start}

Sends the same round — one datagram per target — again and again. Its counters
arrive as [`broadcast://emit-stat`](events.md#event-broadcast-emit-stat) every
250 ms. After more than 32 failed sends with none sent at all, it stops with
the reason. *Starts a job* (`beacon`, `params.mode`, `params.target`,
`params.targets`, `params.rate`).

| Argument | Type | Required | Meaning |
| --- | --- | --- | --- |
| `config` | object | yes | As for [`broadcast_send`](#broadcast_send), with the three below |

| Field of `config` | Type | Default | Meaning |
| --- | --- | --- | --- |
| `rate` | number | — | Rounds per second; above 0, and rounds × targets at most 50 000 datagrams per second |
| `count` | number | `0` | Stop after this many rounds; 0: no count |
| `duration_s` | number | `0` | Stop after this many seconds; 0: until stopped |

**Result**: [`JobInfo`](#type-jobinfo).

**Errors**: those of [`broadcast_send`](#broadcast_send),
`broadcast.rate_invalid`, `broadcast.rate_limit`.

### discovery_start {#discovery_start}

Listens on a UDP port, keeps a list of every peer that sends something, and
can answer probes as a device would. The peers arrive as
[`broadcast://peers`](events.md#event-broadcast-peers) every 400 ms. *Starts
a job* (`discovery`, `params.bind`, `params.groups`, `params.joined`).

| Argument | Type | Required | Meaning |
| --- | --- | --- | --- |
| `config` | object | yes | Below |

| Field of `config` | Type | Default | Meaning |
| --- | --- | --- | --- |
| `bind` | string | — | `IP:port` to listen on |
| `groups` | string[] | `[]` | Multicast groups to join (IPv4) |
| `interface` | string | any | The local IPv4 address to join the groups on |
| `reuse` | boolean | `true` | Share the port with a program already listening on it (`SO_REUSEADDR`) |
| `respond` | boolean | `false` | Answer what arrives |
| `response` | [`Payload`](#type-payload) | none | The answer; needed with `respond` |
| `respond_delay_ms` | number | `0` | Wait this long before answering |
| `match_contains` | string | none | Answer only datagrams whose text contains this |

At most 512 peers are listed; later ones are not added.

**Result**: [`JobInfo`](#type-jobinfo).

**Errors**: `node.bind_invalid`, `broadcast.port_shared` (the port is taken
and `reuse` is off), `broadcast.interface_invalid`,
`broadcast.not_multicast`, `broadcast.join_failed`,
`broadcast.reply_missing`, `node.osc_address`, `hex.*`, bind failures. The
job ends with `wait.receive_failed` if the socket can no longer receive.

```bash
invoke broadcast_send '{"config":{"mode":"list","target":"127.0.0.1:9000, 127.0.0.1:9001","payload":{"kind":"text","text":"PING"}}}' \
  | jq '{packets, errors}'
```

## Impairment {#impairment}

A relay between a client and its server that delays, drops, duplicates,
corrupts, reorders or throttles what passes, over UDP or TCP. See
[impairment](../tools/impairment.md).

### netsim_start {#netsim_start}

Starts a relay: what arrives on `listen` goes on to `target`, and the answers
come back the same way, both impaired by the profile. Its counters arrive as
[`netsim://stat`](events.md#event-netsim-stat) every 250 ms. *Starts a job*
(`netsim`, `params.listen`, `params.target`, and `params.protocol` for TCP).

| Argument | Type | Required | Meaning |
| --- | --- | --- | --- |
| `config` | object | yes | Below |

| Field of `config` | Type | Default | Meaning |
| --- | --- | --- | --- |
| `listen` | string | — | `IP:port` the relay listens on; point the client here |
| `target` | string | — | `IP:port` of the real server, or `host:port` — a host name is looked up once, when the relay starts |
| `profile` | [`ImpairProfile`](#type-impairprofile) | — | What to do to the traffic |
| `seed` | number | new | The draws' seed: the same seed and the same traffic give the same drops |
| `protocol` | string | `udp` | `udp` (datagrams) or `tcp` (streams) |

**Result**: [`JobInfo`](#type-jobinfo).

**Errors**: `node.range` (a value of the profile outside its range, with
`min`, `max` and the field), `node.too_long`, `node.bind_invalid`,
`transport.target_invalid`, `transport.dns` (a target name that cannot be
found), bind failures. The job ends with
`wait.receive_failed` if a socket can no longer receive.

### netsim_set_profile {#netsim_set_profile}

A running relay impairs with another profile from now on, without closing its
sockets.

| Argument | Type | Required | Meaning |
| --- | --- | --- | --- |
| `jobId` | number | yes | The relay's job |
| `profile` | [`ImpairProfile`](#type-impairprofile) | yes | The new profile |

**Result**: null.

**Errors**: `netsim.not_running`, `node.range`, `node.too_long`.

```bash
invoke netsim_start '{"config":{"listen":"127.0.0.1:9010","target":"127.0.0.1:9000","profile":{"latency_ms":80,"jitter_ms":20,"loss":0.02}}}'
```

## Storm and scanner {#storm-scanner}

::: danger
A storm loads a target as hard as you ask, and a scan probes every port of a
range. Aim them only at hosts you are responsible for.
:::

### storm_start {#storm_start}

Sends a steady load of UDP datagrams or TCP connections to one target. Its
counters arrive as [`storm://stat`](events.md#event-storm-stat) every 250 ms.
*Starts a job* (`storm`, `params.protocol`, `params.target`, `params.rate`).

| Argument | Type | Required | Meaning |
| --- | --- | --- | --- |
| `config` | object | yes | Below |

| Field of `config` | Type | Default | Meaning |
| --- | --- | --- | --- |
| `target` | string | — | `IP:port` or `host:port`; a name is looked up once, when the job starts |
| `protocol` | string | — | `udp`: datagrams; `tcp`: a connection per unit that writes the payload and closes (each connect may take 500 ms) |
| `size` | number | — | Payload bytes, held between 1 and 65 507 |
| `rate` | number | — | Units per second, on a schedule: unit *n* is due *n* / `rate` seconds after the start, and each wake sends what is due (at most 256; a schedule further behind skips the older units); 0 sends as fast as it can |
| `duration_s` | number | `0` | Stop after this many seconds; 0: until stopped |

**Result**: [`JobInfo`](#type-jobinfo).

**Errors**: `transport.target_invalid`, `transport.dns`. Failed sends are
counted in the events, not reported as errors.

### scan_start {#scan_start}

Tries a TCP connection to every port of a range and reports the open ones,
with what the service says first when asked. Open ports arrive as
[`scan://open`](events.md#event-scan-open), progress as
[`scan://progress`](events.md#event-scan-progress). *Starts a job* (`scan`,
`params.host`, `params.from`, `params.to`).

| Argument | Type | Required | Meaning |
| --- | --- | --- | --- |
| `config` | object | yes | Below |

| Field of `config` | Type | Default | Meaning |
| --- | --- | --- | --- |
| `host` | string | — | A host name or an address |
| `port_start`, `port_end` | number | — | The range, both included; given the wrong way round, they are swapped |
| `concurrency` | number | `256` | Attempts at once, 1 to 1024 |
| `timeout_ms` | number | `600` | Per port, 50 to 10 000 |
| `grab_banner` | boolean | `false` | Read up to 256 bytes the service sends within 400 ms of connecting |

**Result**: [`JobInfo`](#type-jobinfo).

**Errors**: `scan.host_required`.

```bash
invoke scan_start '{"config":{"host":"127.0.0.1","port_start":8000,"port_end":9100,"grab_banner":true}}'
```

## Inspector {#inspector}

The Inspector records what the tools send and receive, as frames, while
capture is armed. On a server there is one Inspector for every page and
script. See [the Inspector](../tools/inspector.md).

### inspect_set_enabled {#inspect_set_enabled}

Arms or disarms capture. While disarmed nothing is recorded.

| Argument | Type | Required | Meaning |
| --- | --- | --- | --- |
| `enabled` | boolean | yes | Arm (`true`) or disarm |

**Result**: [`CaptureStats`](#type-frame).

### inspect_stats {#inspect_stats}

The capture's counters. No arguments.

**Result**: [`CaptureStats`](#type-frame).

### inspect_snapshot {#inspect_snapshot}

The newest frames, oldest first.

| Argument | Type | Required | Meaning |
| --- | --- | --- | --- |
| `limit` | number | yes | How many, 1 to 8192 |

**Result**: [`Frame`](#type-frame)`[]`, without their bytes (see
[`inspect_payload`](#inspect_payload)).

### inspect_clear {#inspect_clear}

Empties the capture and its counters. No arguments.

**Result**: [`CaptureStats`](#type-frame).

### inspect_export {#inspect_export}

Writes every frame held to `capture-<ms>.jsonl` or `capture-<ms>.txt` in the
data folder. In `jsonl`, each line is a frame with the bytes it keeps in
`data`, base64; `txt` is for reading, with a hex dump of each frame.

| Argument | Type | Required | Meaning |
| --- | --- | --- | --- |
| `format` | string | yes | `txt`; anything else writes `jsonl` |

**Result**: string, the path written.

**Errors**: `inspect.empty`, `file.io`.

### inspect_payload {#inspect_payload}

The bytes a frame keeps, past the 1 KiB preview its batch carried.

| Argument | Type | Required | Meaning |
| --- | --- | --- | --- |
| `seq` | number | yes | The frame's number |

**Result**: `{ "seq", "bytes", "kept", "dump", "hex" }` — `bytes` the frame's
size, `kept` how many of them are kept (up to 256 KiB), `dump` every row as
`offset  hex  |ascii|`, `hex` the plain hex a replay sends.

**Errors**: `inspect.frame_gone` (newer frames took its place),
`inspect.no_payload` (only its size was recorded).

```bash
invoke inspect_set_enabled '{"enabled":true}'
invoke inspect_snapshot '{"limit":20}' | jq '.[] | {seq, proto, dir, summary}'
```

## Signal library {#signals}

The library is `signals.json` in the data folder. It is only storage: a signal
is sent with the command of its transport (`osc_send`, `broadcast_send`,
`http_request`, `mqtt_publish` or `mqtt_publish_once`). See
[signals](../tools/signals.md) and [files](../reference/files.md#signals-json).

### signals_load {#signals_load}

Reads the library. When the file does not exist, the starter set is written
first. No arguments.

**Result**: `{ "path": string, "library": library, "seeded": boolean }` —
`seeded` is true when the starter set was just written. The library is
`{ "version", "signals": [...], "folders": [...] }`: `version` 2 (a version 1
file comes back as it is), `folders` left out when there are none. Each signal
has `id`, `name`, `group` (its folder, `"A/B"`; empty for none), `note` and
`body`.

**Errors**: `signals.json_invalid` (with `path`, `line`, `column`; the file is
never replaced), `file.io`.

### signals_save {#signals_save}

Replaces the whole library file, through a temporary file in the same folder.
A file that exists and does not read as a library is left as it is.

| Argument | Type | Required | Meaning |
| --- | --- | --- | --- |
| `library` | object | yes | `{ version, signals, folders }` as `signals_load` returns it |

**Result**: string, the path written.

**Errors**: `signals.json_invalid` (the file now on disk does not read, with
`path`, `line`, `column`; nothing is written), `signals.encode`, `file.io`.

A signal's `body` by its `transport`:

| `transport` | Fields |
| --- | --- |
| `osc` | `target`, `address`, `args` ([`OscArg`](#type-oscarg)`[]`) |
| `udp` | `target`, `payload`: `{ "kind": "text", "text" }` or `{ "kind": "hex", "hex" }` |
| `http` | `request` ([`HttpRequest`](#type-httprequest)) |
| `mqtt` | `broker` (`host:port`), `topic`, `payload`, `qos`, `retain` |

```bash
invoke signals_load | jq '.library.signals[] | {name, transport: .body.transport}'
```

## Emulators {#emulators}

An emulator is Signal Lab playing the other side: an HTTP API, an OSC, UDP or
TCP device, an MQTT broker. Its document — `name`, `bind`, `protocol`, the
protocol's rules and an optional `outage` — is described in
[emulators](../tools/emulators.md). The library is `emulators.json` in the
data folder.

### emulators_load {#emulators_load}

Reads the emulator library. When the file does not exist, the starter set is
written first. No arguments.

**Result**: `{ "path", "library": { "version": 1, "emulators": [{ "id", "note", "emulator" }] }, "seeded" }`.

**Errors**: `emulators.json_invalid` (with `path`, `line`, `column`; never
replaced), `file.io`.

### emulators_save {#emulators_save}

Replaces the whole emulator library, through a temporary file.

| Argument | Type | Required | Meaning |
| --- | --- | --- | --- |
| `library` | object | yes | `{ version, emulators }` as `emulators_load` returns it |

**Result**: string, the path written.

**Errors**: `emulators.encode`, `file.io`.

### emulator_check {#emulator_check}

Whether an emulator would start: everything [`emulator_start`](#emulator_start)
checks before it binds.

| Argument | Type | Required | Meaning |
| --- | --- | --- | --- |
| `emulator` | object | yes | The emulator document |
| `params` | object of strings | no | Values its templates read as parameters |

**Result**: null when it would start.

**Errors**: `emulator.*`; `node.*` for a field missing, out of range, too long
or malformed (`node.required`, `node.range`, `node.too_long`,
`node.bind_invalid`, `node.target_invalid`, `node.method_invalid`…);
`param.unknown`, `template.*`, `osc.pattern_*`, `regex.invalid`,
`hex.invalid`. Each has `rule`, `retained` or `response` in `params` when the
problem is in one of them.

### emulator_start {#emulator_start}

Starts an emulator as a job of its own. Its socket is open when the command
returns. What it receives and answers arrives as
[`emulator://activity`](events.md#event-emulator-activity) every 200 ms when
something changed. *Starts a job* (`emulator`, `params.name`,
`params.protocol`, `params.local`, and `params.source` when `source` is
given).

| Argument | Type | Required | Meaning |
| --- | --- | --- | --- |
| `emulator` | object | yes | The emulator document |
| `params` | object of strings | no | Values its templates read as parameters |
| `seed` | number | no | Its seed, 0 to 9007199254740991; default: a new one |
| `source` | string | no | The library entry it comes from, kept on the job as `params.source` |

**Result**: [`JobInfo`](#type-jobinfo).

**Errors**: those of [`emulator_check`](#emulator_check), `seed.range`,
`transport.address_in_use` and the other bind failures.

### emulator_exchanges {#emulator_exchanges}

What a running emulator received and answered. It keeps the last 500
exchanges.

| Argument | Type | Required | Meaning |
| --- | --- | --- | --- |
| `jobId` | number | yes | The emulator's job |
| `after` | number | no | Only exchanges numbered above this; default 0 |
| `limit` | number | no | At most this many, 1 to 500; default 500 |

**Result**

| Field | Type | Meaning |
| --- | --- | --- |
| `job_id` | number | The job |
| `name`, `protocol`, `local` | string | The emulator, its protocol and the address it listens on |
| `counts` | object | `total`, `unmatched` (no rule took it), `failed`, `down` (arrived while it was down: its outage or [`emulator_down`](#emulator_down)), `hits` (per rule), and `missed` (MQTT: messages a client too far behind did not get; left out while 0) |
| `forced` | string | `unavailable`, `reset` or `timeout` while it is taken down; left out otherwise |
| `exchanges` | object[] | Each: `seq`, `ts`, `from`, `request`, `rule` (1-based; left out when none took it), `reply`, `status`, `fault`, `ms`, `error`, `frame`, `down`, and `data` (the request as templates read it) |

**Errors**: `emulator.not_running`.

### emulator_down {#emulator_down}

Takes a running emulator down until it is brought up, whatever its outage
schedule says, or brings it back. While down, an HTTP emulator meets each
request with `fault`, a TCP device and an MQTT broker drop their connections
and refuse new ones, and OSC and UDP devices answer nothing.

| Argument | Type | Required | Meaning |
| --- | --- | --- | --- |
| `jobId` | number | yes | The emulator's job |
| `down` | boolean | yes | Down (`true`) or up |
| `fault` | string | no | What HTTP requests meet: `unavailable` (503, without `Retry-After`: when it comes back is not known), `reset` (the connection closes), `timeout` (no answer); default `unavailable` |

**Result**: null.

**Errors**: `emulator.not_running`.

```bash
invoke emulator_exchanges '{"jobId":5,"after":0}' | jq '.counts, (.exchanges[] | {request, rule, status})'
```

## Shared types {#types}

### JobInfo {#type-jobinfo}

What a command that starts a job returns, and what [`jobs_list`](#jobs_list)
lists: `id`, `kind`, `label` (English, for logs), `params` (the values the
label names; left out when none) and `started_ms`. See
[jobs](index.md#jobs).

### OscArg {#type-oscarg}

One OSC argument, its type and its value:

| `type` | `value` | OSC tag |
| --- | --- | --- |
| `int` | 32-bit integer | `i` |
| `float` | number, sent as 32-bit float | `f` |
| `str` | string | `s` |
| `long` | 64-bit integer | `h` |
| `double` | number, 64-bit | `d` |
| `bool` | `true` or `false` | `T` or `F` |
| `blob` | array of bytes, `[222, 173]` | `b` |
| `nil` | none: `{ "type": "nil" }` | `N` |

### HttpRequest {#type-httprequest}

| Field | Type | Default | Meaning |
| --- | --- | --- | --- |
| `method` | string | — | `GET`, `POST`… |
| `url` | string | — | `http://` or `https://` |
| `headers` | `[name, value][]` | `[]` | Request headers |
| `body` | string or null | null | The body |
| `timeout_ms` | number | `10000` | For the whole exchange |
| `auth` | object | none | `{ "scheme": "basic", "username", "password" }`, `{ "scheme": "digest", "username", "password" }` or `{ "scheme": "bearer", "token" }` |

Up to 10 redirects are followed. Credentials and cookies typed for one host
never go on to another. A Digest request answers the server's 401 challenge
and sends again.

### HttpResponse {#type-httpresponse}

| Field | Type | Meaning |
| --- | --- | --- |
| `ok` | boolean | A 2xx status |
| `status`, `status_text` | number, string | The status; 0 and empty without a response |
| `latency_ms` | number | Until the whole body arrived |
| `headers` | `[name, value][]` | Response headers |
| `body` | string | The body as text, at most 256 KiB |
| `body_bytes` | number | The body's full size |
| `truncated` | boolean | `body` was cut at 256 KiB |
| `error` | string or null | Why there was no response, every layer of the cause |
| `cause` | string or null | What kind of failure: `refused`, `timeout`, `dns`, `unreachable`, `reset`, `address_in_use`, `address_unavailable`, `denied`, `tls`, `target_invalid`, `failed` — the same as the `transport.*` codes |
| `digest` | object | A Digest request that met a 401 only; left out otherwise. `challenged`: the challenge was answered and the request sent again. `error`: why it could not be, an `EngineError` (`http.digest_not_offered`, `http.digest_unsupported`, `http.digest_invalid`, `http.digest_other_origin`) or null |

### Payload {#type-payload}

What a broadcast or discovery datagram carries:
`{ "kind": "osc", "address", "args" }`, `{ "kind": "text", "text" }` (sent as
is, no terminating zero) or `{ "kind": "hex", "hex" }` (`de ad be ef`,
`deadbeef`, `0xDE,0xAD` — anything but hex digits is ignored).

### MqttConfig {#type-mqttconfig}

| Field | Type | Default | Meaning |
| --- | --- | --- | --- |
| `host` | string | — | The broker's name or address |
| `port` | number | — | Usually 1883 |
| `client_id` | string | — | Not empty; another connection with the same id is knocked off by the broker |
| `username`, `password` | string | empty | `username` is sent when not empty; `password` only along with a `username` |
| `keep_alive_s` | number | `60` | Pings go at half of it; 0: none |
| `clean_session` | boolean | `true` | The CONNECT flag |
| `will` | object or null | null | `{ topic, payload, qos, retain }`, published by the broker if the connection is lost |
| `subscribe` | `{ filter, qos }[]` | `[]` | Subscribed as soon as the connection is up |

### WsConfig {#type-wsconfig}

| Field | Type | Default | Meaning |
| --- | --- | --- | --- |
| `url` | string | — | `ws://` or `wss://` (`wss://` trusts what the system trusts for HTTPS) |
| `headers` | `[name, value][]` | `[]` | Sent with the upgrade request |
| `protocols` | string[] | `[]` | Subprotocols to offer, in order of preference |
| `timeout_ms` | number | `10000` | For the connection, TLS and the upgrade together |

Messages are at most 16 MiB either way.

### ImpairProfile {#type-impairprofile}

Every field is optional; what is left out does nothing. Probabilities are 0
to 1.

| Field | Range | Meaning | UDP | TCP |
| --- | --- | --- | --- | --- |
| `name` | at most 60 characters | A label for the timeline and the report | yes | yes |
| `latency_ms` | 0 to 60 000 | Delay added to everything | yes | yes |
| `jitter_ms` | 0 to 60 000 | Up to this much more, drawn each time | yes | yes |
| `loss` | 0 to 1 | A datagram is dropped | yes | — |
| `duplicate` | 0 to 1 | A datagram is sent twice | yes | — |
| `corrupt` | 0 to 1 | One bit of a datagram is flipped | yes | — |
| `reorder` | 0 to 1 | A datagram is held back so later ones overtake it | yes | — |
| `rate_kbps` | 0, or 8 to 10 000 000 | Bandwidth limit, kilobits per second; 0: none | yes | yes |
| `burst_start` | 0 to 1 | A datagram starts a burst of losses | yes | — |
| `burst_length` | 1 to 1000 | Datagrams a burst lasts on average (needed with `burst_start`) | yes | — |
| `offline` | `true` or `false` | Nothing gets through | yes | yes |
| `reset` | 0 to 1 | A chunk of a stream resets its connection | — | yes |
| `stall` | 0 to 1 | A chunk of a stream leaves its connection half-open | — | yes |

### Frame and CaptureStats {#type-frame}

A `Frame` is one captured packet, request or message:

| Field | Meaning |
| --- | --- |
| `seq` | Its number, rising |
| `ts` | When, milliseconds since 1970 |
| `proto` | `osc`, `udp`, `tcp`, `http`, `mqtt`, `ws`… |
| `dir` | `tx` (sent) or `rx` (received) |
| `source` | The tool that captured it: `osc-monitor`, `broadcast`, `netsim`… |
| `job_id` | Its job, or null |
| `local`, `remote` | The addresses: `IP:port` of this side and of the other (a URL or a broker for HTTP, WebSocket and MQTT). A relayed frame's `local` is the address the relay listens on and its `remote` where the frame was going; the leg ends its `verdict` (`· client→target`, `· target→client`) |
| `bytes` | Its size |
| `summary` | One line |
| `detail` | A decode over several lines, or null |
| `hex` | A hex dump of the first 1 KiB, or null |
| `verdict` | What became of it — `dropped`, `sampled`, a status — or null |
| `kept` | Of `bytes`, how many are kept (up to 256 KiB); 0 when only the size was recorded |
| `publish` | An MQTT publish only: `{ broker, topic, qos, retain, text }` — the broker as `host:port`, and whether the kept bytes, which are the message's payload, are UTF-8 text. Absent for every other frame |

`CaptureStats`: `enabled`, `total` (frames recorded), `bytes`, `skipped`
(recorded but never sent to the interface), `buffered` (frames held),
`capacity` (8192), `held` (payload bytes held) and `held_limit` (64 MiB). The
oldest frames give way past either limit.
