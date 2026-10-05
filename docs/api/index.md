---
title: HTTP API
description: Drive a Signal Lab server from scripts, CI and other tools with the same commands and events its own interface uses.
---

# The HTTP API

To drive Signal Lab from a script, a CI pipeline or another tool, talk to a
Signal Lab server (`signal-lab-server`, or the Docker image) over HTTP. The
interface the server shows in a browser uses exactly this API: every button is a
call to `/api/invoke/<command>`, every live number arrives on `/api/events`. So
anything a person can do on the server's page, a script can do too.

The desktop app has no HTTP API: its window reaches its engine inside the app.
To automate on a desktop, run a server on the same machine (see
[the server](../server/index.md)) or use the command line
[`signallab`](../automation/cli.md).

## Endpoints {#endpoints}

| Method and path | What it does | Token |
| --- | --- | --- |
| `GET /api/health` | Whether the server answers, its version, whether it wants a token | not needed |
| `POST /api/invoke/<command>` | Runs one engine command: JSON arguments in, JSON result out ([commands](commands.md)) | needed |
| `POST /api/run` | Runs an experiment to its end: the result, or its steps as lines ([runs](run.md)) | needed |
| `GET /api/events` | WebSocket of every engine event ([events](events.md)) | needed |
| `GET /api/files?path=…` | A file the engine wrote in the data folder, as a download | needed |
| `GET /api/openapi.json` | This API described in OpenAPI 3.1 | needed |
| `GET /login`, `POST /login` | The sign-in page and form for browsers | not needed |
| `POST /logout` | Ends a browser's session | a session |

Any other path under `/api/` answers `404` with the code `api.not_found`; a
method a path does not take (`GET /api/invoke/…`) is `405`, with an empty body.
Everything else is the interface; on a server with a token, a browser without
a session is sent to `/login` first.

## Base URL {#base-url}

A server listens on `http://127.0.0.1:1430` unless told otherwise with
`--listen` (or `SIGNALLAB_LISTEN`). The Docker image listens on every network
card, `0.0.0.0:1430`. The examples on these pages use:

```bash
SERVER=http://127.0.0.1:1430
```

The server speaks plain HTTP. For HTTPS, put a reverse proxy that terminates
TLS in front of it and start the server with `--secure-cookie`.

## Authentication {#authentication}

A server started without a token listens on loopback only and needs no
authentication: anyone on that machine may use it. A server that others can
reach always has a token, and then every request except `/api/health` and
`/login` must carry it.

**Scripts** send the token in the `Authorization` header:

```bash
TOKEN=$(cat token.txt)
curl -fsS "$SERVER/api/invoke/jobs_list" -X POST \
  -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json"
```

The header must be exactly `Bearer`, one space, and the token. A missing or
wrong token is `401` with the code `auth.required`.

**Browsers** sign in once at `/login` with the token and get a session cookie,
`signallab_session`: `HttpOnly`, `SameSite=Strict`, kept 7 days, and `Secure`
when the server runs with `--secure-cookie`. `POST /logout` ends it. A wrong
token on the sign-in form costs one second before the answer, which makes
guessing slow. Sessions live in the server's memory: a restart signs every
browser out, while scripts with the token are not affected. The server keeps
at most 1024 sessions; past that, the oldest goes.

Where the token comes from is the server's setup (`--token-file`,
`SIGNALLAB_TOKEN`, or the `token` file `--generate-token` makes in the data
folder): see [server security](../server/security.md). A token has at least 24
characters and no spaces; `signal-lab-server token` prints a new one.

::: warning
Anyone with the token can make the server send traffic. Keep the file it is in
readable only by you, and never put it in a URL — the server does not read a
token there anyway.
:::

## Host and Origin {#host-origin}

Two checks run before anything else, on every path, `/api/health` included.

**`Host`.** The `Host` header must name this server:

- Loopback names always pass: `localhost`, names ending in `.localhost`,
  `127.x.x.x` and `[::1]`.
- Names given with `--allowed-host` (or `SIGNALLAB_ALLOWED_HOSTS`) pass.
- A server with a token and no `--allowed-host` answers to any name.

Anything else is `403` with `auth.host`. Address the server by a name it
accepts: `curl` sends the host of the URL you give it.

**`Origin`.** A request that changes something (any method but `GET` and
`HEAD`) and the WebSocket upgrade must come from the server's own page when
they carry an `Origin` header: its host and port must equal `Host`. Otherwise
the answer is `403` with `auth.origin`; `Origin: null` is refused too. Scripts
and `curl` send no `Origin` and pass this check — they still need the token.

**JSON only.** `POST /api/invoke/…` and `POST /api/run` take
`Content-Type: application/json` (parameters such as `; charset=utf-8` are
fine). Anything else is `415` with `command.json_required`. A web page on
another site cannot send that without asking the server first, and the server
never agrees.

## Calling a command {#invoke}

```http
POST /api/invoke/<command>
Content-Type: application/json

{ "argument": "value", … }
```

- The body is one JSON object with the command's arguments. A command without
  arguments takes `{}` or an empty body (the `Content-Type` header is still
  needed).
- Argument names are camelCase, as the interface sends them: `jobId`,
  `nodeId`. Objects passed as an argument (`config`, `request`, `document`,
  `library`…) keep the field names the engine writes, which are mostly
  snake_case: `timeout_ms`, `port_start`.
- An argument a command does not know is an error, never ignored: `422` with
  `command.args_invalid` naming the command, and the parser's words in
  `detail`. So is a required one left out. A command without arguments does
  not read the body at all.
- An optional argument may be left out or sent as `null`.
- The answer is `200` with the command's result as JSON. A command that has
  nothing to return answers `null`.

Every command, with its arguments and result, is in [commands](commands.md).

## Errors {#errors}

| Status | When | Body |
| --- | --- | --- |
| `200` | The command ran; `/api/run` started the run | The result |
| `400` | The body is not JSON; a run request cannot be read | `EngineError`: `command.args_invalid`, `api.run_invalid`, `api.run_source` |
| `400` | `/api/files` without `path` | Plain text from the web server, not an `EngineError` |
| `401` | No token, or a wrong one | `EngineError`: `auth.required` |
| `403` | A `Host` or `Origin` the server refuses | `EngineError`: `auth.host`, `auth.origin` |
| `404` | No such API path; a file that is not in the data folder | `EngineError`: `api.not_found`, `file.not_found` |
| `405` | A method the path does not take | Empty |
| `413` | A request body over 24 MiB | Plain text from the web server, not an `EngineError` |
| `413` | A download over 256 MiB | `EngineError`: `file.too_large` |
| `415` | Not `application/json` | `EngineError`: `command.json_required` |
| `422` | The command failed, or the run could not start | `EngineError`: any code of the engine |
| `500` | A file could not be read | `EngineError`: `file.io` |

An unknown command name is `422` with `command.unknown`.

Every failure the engine reports has one shape, the `EngineError`:

```json
{
  "code": "transport.refused",
  "params": { "target": "http://127.0.0.1:8080/" },
  "node": "request",
  "field": { "key": "url" },
  "detail": "error sending request for url (http://127.0.0.1:8080/): tcp connect error: Connection refused (os error 111)"
}
```

| Field | What it is |
| --- | --- |
| `code` | What went wrong: a stable identifier. Every code and its message are listed in [error messages](../reference/errors.md), grouped by the part before the dot (for example [`transport`](../reference/errors.md#transport)) |
| `params` | The values the message names, all as strings. Left out when there are none |
| `node` | The experiment node it is about. Left out when there is none |
| `field` | The field it is about: `key` (named in [fields](../reference/errors.md#fields)) and `index`, 1-based, for repeated fields such as a header. Left out when there is none |
| `detail` | The operating system's, a parser's or a library's own words, in English. Left out when there are none |

Branch on `code`, never on `detail`. The secret values a run or a single send
uses are masked (`••••`) in every error it reports.

A request that reaches its server but gets an error status, or none at all,
is not a failed command: `http_request` answers `200` with the response, and
`ok`, `error` and `cause` say what happened. See
[`http_request`](commands.md#http_request).

## Limits {#limits}

| What | Limit | At the limit |
| --- | --- | --- |
| A request body | 24 MiB | `413` |
| An experiment document | 4 MiB | `file.too_large` |
| A download from `/api/files` | 256 MiB | `413`, `file.too_large` |
| A run's length | 300 s | The run fails with `run.timeout` |
| The feedback form (`feedback_send`) | 15 MiB in all | `feedback.too_large` |
| Events waiting for one WebSocket | 4096 | It gets [`server://lagged`](events.md#event-server-lagged) with how many it missed |

## Events {#events}

`GET /api/events` upgraded to a WebSocket streams every event the engine
sends — a run's steps, a job's end, a monitor's messages, the Inspector's
frames — to every connected client, as text messages:

```json
{ "event": "job://ended", "payload": { "job_id": 7, "kind": "storm", "error": null } }
```

It takes the same token and `Origin` rules as the rest. Every channel and its
payload is in [events](events.md).

## Files {#files}

`GET /api/files?path=<path>` downloads a file the engine wrote in the server's
data folder: a run report (`report_path` of a run's result), an export of the
experiment or of the Inspector, the signal or emulator library. `path` is the
path the engine gave you, on the server (URL-encoded):

```bash
curl -fsS -G "$SERVER/api/files" --data-urlencode "path=/data/runs/run-1759600000000-3.json" \
  -H "Authorization: Bearer $TOKEN" -o report.json
```

- Only files inside the data folder are served. Anything else, a folder or a
  file that does not exist is `404` with `file.not_found`.
- The answer is `application/octet-stream` with
  `Content-Disposition: attachment`. Characters of the file name other than
  letters, digits, `.`, `_` and `-` become `_`.
- A file over 256 MiB is `413` with `file.too_large`.

What the data folder holds is in [files and folders](../reference/files.md).

## Health {#health}

`GET /api/health` is open: it needs no token, only a `Host` the server accepts.

```bash
curl -fsS "$SERVER/api/health"
```

```json
{ "status": "ok", "version": "[[version]]", "auth": true }
```

`auth` says whether requests need a token. `signal-lab-server healthcheck`
asks the same address the server listens on and exits `0` when it answers;
the Docker image's health check runs it.

## OpenAPI description {#openapi}

`GET /api/openapi.json` describes this API in OpenAPI 3.1: the endpoints, every
command with its arguments, and the run request and result. It needs the token
like the rest of `/api/`. These pages are the full reference; where the two
differ, these pages follow the engine.

## Jobs {#jobs}

Long-running work — a monitor, a generator, a burst, a relay, a connection, an
emulator, a run — is a job. A command that starts one returns its `JobInfo`
as soon as it runs:

```json
{ "id": 4, "kind": "osc-monitor", "label": "OSC monitor 0.0.0.0:9000", "params": { "bind": "0.0.0.0:9000" }, "started_ms": 1759600000000 }
```

| Field | What it is |
| --- | --- |
| `id` | The job's number, unique while the server runs; other commands take it as `jobId` or `id` |
| `kind` | `experiment`, `osc-monitor`, `osc-gen`, `http-burst`, `netsim`, `storm`, `scan`, `beacon`, `discovery`, `mqtt`, `websocket` or `emulator` |
| `label` | One English line, for logs |
| `params` | The values the label is made of (target, bind, host…). Left out when there are none |
| `started_ms` | When it started, milliseconds since 1970 |

These commands start a job: `experiment_start`, `osc_monitor_start`,
`osc_generator_start`, `http_burst_start`, `netsim_start`, `storm_start`,
`scan_start`, `broadcast_beacon_start`, `discovery_start`, `mqtt_connect`,
`ws_connect` and `emulator_start`. The server logs each start with the address
of the client that asked; `POST /api/run` starts a job too.

- [`jobs_list`](commands.md#jobs_list) lists the running jobs,
  [`job_stop`](commands.md#job_stop) stops one,
  [`jobs_stop_all`](commands.md#jobs_stop_all) stops every one.
- A job that ends on its own, or fails, sends
  [`job://ended`](events.md#event-job-ended). A job you stop sends nothing more:
  `job_stop` answering `true` is the confirmation.
- Jobs belong to the server, not to the client that started them. Closing the
  page or ending the script does not stop them, every client sees them and can
  stop them, and a server that shuts down stops them all.

## A complete example {#example}

Ask whether the server is up, start a small HTTP emulator with one command,
run the bundled `http-check` experiment against it and wait for the result,
then stop the emulator. `jq` picks fields out of the answers.

```bash
SERVER=http://127.0.0.1:1430
TOKEN=$(cat token.txt)            # leave out with a loopback server without a token
AUTH="Authorization: Bearer $TOKEN"
JSON="Content-Type: application/json"

# 1. Up? Which version? Does it want a token?
curl -fsS "$SERVER/api/health"
# {"status":"ok","version":"[[version]]","auth":true}

# 2. One command: an HTTP emulator on 127.0.0.1:8080 that answers GET / with 200
EMULATOR=$(curl -fsS -X POST "$SERVER/api/invoke/emulator_start" -H "$AUTH" -H "$JSON" -d '{
  "emulator": { "name": "Example", "bind": "127.0.0.1:8080", "protocol": "http",
                "routes": [ { "method": "GET", "path": "/", "responses": [ { "body": "ok" } ] } ] }
}' | jq .id)

# 3. Run the bundled experiment that expects 200 from http://127.0.0.1:8080/, and wait
curl -sS -X POST "$SERVER/api/run" -H "$AUTH" -H "$JSON" -d '{"template":"http-check"}' \
  | jq '{outcome, error, report_path}'
# {"outcome":"passed","error":null,"report_path":"/data/runs/run-1759600000000-2.json"}

# 4. Stop the emulator
curl -fsS -X POST "$SERVER/api/invoke/job_stop" -H "$AUTH" -H "$JSON" -d "{\"id\":$EMULATOR}"
# true
```

`curl -f` turns an error status into a failed command; leave it out (as in
step 3) to see the `EngineError` in the body.
