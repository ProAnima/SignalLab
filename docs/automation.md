# Automation: the API, the command line and CI

Status: 2026-10-02.

The experiments that bring an installation up are also its regression tests.
Signal Lab runs them without a person in two ways: the **command line**
`signallab` (in this process, or on a lab server) and the **HTTP API** of
`signal-lab-server`. Both go through the engine's own command table, so a run
in a pipeline is the run the app would make — same steps, same report, same
messages in English or Russian.

## 1. The command line

`signallab` ships next to the installers in every release
(`signallab-<version>-windows-x64.zip`, `signallab-<version>-linux-x64.tar.gz`)
and in the server image (`/usr/local/bin/signallab`). From a checkout:
`cargo run -p signal-lab-cli -- <command>`.

```bash
signallab run tests/smoke.json --param api=http://127.0.0.1:8080 --junit junit.xml
signallab run http-check poll-until-ready          # bundled templates by name
signallab validate tests/*.json                    # the editor's check, nothing sent
signallab send osc 127.0.0.1:9000 /cue/go f:0.75 s:main
signallab send http GET http://127.0.0.1:8080/health --expect-status 200
signallab fire "Fader value" --library signals.json
signallab templates
```

### `run` and `validate`

| Option | |
| --- | --- |
| `<FILE>…` | experiment files (any version the app reads; older ones are migrated) or bundled template names |
| `--param NAME=VALUE` | a parameter value for this run, repeatable; it applies to the experiments that have that parameter and must belong to at least one |
| `--profile NAME` | run with this profile (`""` = the defaults) |
| `--seed N` | the seed of the random values; a failed run prints the one it used |
| `--timeout SECONDS` | fail a run that takes longer (1–300, default 300) |
| `--junit PATH` | a JUnit report: a suite per experiment, a case per node that ran, skipped cases for nodes the run never reached, the failure in words with its technical detail |
| `--report PATH` | copy the run report (the app's JSON): a file for one experiment, a folder for several |
| `--data-dir PATH` | keep this process's runs there; by default a temporary folder, removed at exit |
| `--server URL` | run on that server instead (§3) |
| `--token-file PATH` | the server's token; else `SIGNALLAB_TOKEN` |
| `--secrets files\|system` | where secrets come from in this process (below) |
| `--secrets-dir PATH` | a folder of secret files, one per name |

Every experiment is checked before the first one runs, so a broken third file
stops the first one too. Steps are printed as they happen, like the app's
timeline; each experiment ends with one line on stdout, and several with a
summary.

```
▶ HTTP check (http-check) · seed 2057534441848683
     0.000  Start         Passed · Started · seed 2057534441848683
     0.002  HTTP request  Passed · HTTP 404 · 2 ms
     0.002  HTTP status   Failed · Expected HTTP 200, received 404
✖ HTTP check failed after 2 ms: HTTP status — Expected HTTP 200, received 404
  To run it again with the same random values: --seed 2057534441848683
```

On GitHub Actions a failure is also an `::error` annotation on the run page.

### Exit codes

| Code | Meaning |
| --- | --- |
| `0` | every experiment passed; the send succeeded |
| `1` | an experiment ran and failed (or ran out of time); a send failed (refused, no route, unexpected status) |
| `2` | the invocation or a document is wrong: an argument, a file that cannot be read, a validation error, an unknown parameter, a missing secret |
| `3` | nothing could run for a reason outside the experiment: the server cannot be reached or refuses the token, a port that cannot be opened |

With several experiments the most serious code wins.

### Output for machines

`--json` (anywhere on the line) prints one JSON object per line on stdout and
nothing else: `started`, every `step`, `ended` (the run
result: outcome, seed, params, error, steps, report path), then `summary`
(`total`, `passed`, `failed`, `exit_code`). A failure that kept an experiment
from starting is `{"type":"error","error":{code,params,…},"exit_code":2}`.
Errors are always the engine's `EngineError` — a stable `code` and its values —
so a script can branch on `error.code` whatever the language.

### Secrets

`{{secret.NAME}}` in an experiment reads `SIGNALLAB_SECRET_NAME`, or a file
named `NAME` in `--secrets-dir` (default `/run/secrets/signallab` when it
exists — the Docker secrets layout). `--secrets system` reads the Windows
Credential Manager or the macOS Keychain instead, as the desktop app does. A
value is never printed: steps, errors and reports show `••••`. A missing secret
is exit code 2, before any traffic.

### Language

`--lang en|ru`, else `SIGNALLAB_LANG`, else the locale (`LC_ALL`,
`LC_MESSAGES`, `LANG`), else English. The texts are the interface's own
(`src/lib/locales/*.ts`, embedded at build time), plural forms included.

### `send` and `fire`

| Command | |
| --- | --- |
| `send osc <host:port> <address> [ARG]…` | one OSC message; arguments typed as `i:3 f:0.5 d:1.5 h:64 s:text b:de ad T F N`, a plain integer is `i`, a plain decimal `f`, anything else `s` |
| `send udp <host:port> --text T \| --hex "de ad"` | one datagram |
| `send http <METHOD> <URL> [-H 'Name: value'] [--body TEXT\|@file] [--expect-status N] [--timeout MS]` | prints the status line on stderr and the body on stdout |
| `send mqtt <host:port> <topic> [payload] [--qos 0\|1\|2] [--retain]` | one publish, MQTT 3.1.1; an empty payload with `--retain` clears a retained value |
| `fire <id or name> [--library PATH]` | a signal of a library (default: the app's `Documents/SignalLab/signals.json`), by id, else by name |

They use the commands the app's screens use, so a fired signal is
byte-identical to one fired in the app.

**Git Bash on Windows** rewrites arguments that start with `/` into Windows
paths (`/cue/go` becomes `C:/Program Files/Git/cue/go`). Write `//cue/go`, or
run with `MSYS_NO_PATHCONV=1`; PowerShell and `cmd` are not affected.

## 2. The HTTP API

`signal-lab-server` serves the API next to the interface. Without a token it
listens on loopback only and needs no authentication; with one, every endpoint
but `/api/health` needs `Authorization: Bearer <token>` (or a browser's session
cookie). Requests take JSON only and must not carry another site's `Origin`. The
full description is `GET /api/openapi.json` (OpenAPI 3.1, `docs/api/openapi.json`).

| Endpoint | |
| --- | --- |
| `GET /api/health` | `{status, version, auth}`, open to all |
| `POST /api/run` | run an experiment to its end (below) |
| `POST /api/invoke/<command>` | any command of the engine's table, with the app's arguments (`osc_send`, `http_request`, `experiment_validate`, `jobs_list`, `job_stop`, …); `200` with the result, `422` with an `EngineError` |
| `GET /api/events` | WebSocket of every event the app gets (`experiment://step`, `job://ended`, …) |
| `GET /api/files?path=` | a file the engine wrote, inside its data folder (run reports, exports) |

### `POST /api/run`

```json
{ "document": { … } , "overrides": { "api": "http://10.0.0.5" }, "profile": "Stage", "seed": 42, "timeout": 60 }
```

`document` is an experiment as the app saves it, or `"template": "http-check"`
names a bundled one; everything else is optional. By default the answer comes
when the run has ended: `200` with `{ outcome, seed, profile, params, error?,
steps, report_path?, … }` — a failed run is still `200` with
`"outcome": "failed"`. An HTTP error means no run started: `400` the request
could not be read, `422` the experiment could not start (validation, an unknown
parameter, a missing secret, a taken port), with the `EngineError`. While a run
is quiet the body carries a space every 15 s, which JSON ignores and which
keeps proxies from closing the connection.

With `Accept: application/x-ndjson` the answer is one JSON object per line as
the run goes: `{"type":"started"}`, a `{"type":"step"}` per step,
`{"type":"heartbeat"}` every 15 s, and last `{"type":"ended", …the result}`.

The run is a job like one started in the interface — listed, stoppable, shown
to every open page — and it is logged with the client's address. A client that
goes away does not stop it: it runs to its end and keeps its report. When the
server shuts down, the run is stopped and the answer ends with
`"outcome": "stopped"`.

```bash
TOKEN=$(cat signallab_token.txt)
curl -fsS -X POST http://lab-pc:1430/api/run \
  -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
  -d '{"template":"http-check"}' | jq .outcome

jq '{document: ., overrides: {api: "http://10.0.0.5"}}' tests/smoke.json |
  curl -sN -X POST http://lab-pc:1430/api/run -H "Accept: application/x-ndjson" \
    -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" --data @-
```

## 3. Runs on a lab server

Gear on an installation's network is reachable from the lab PC, not from a
cloud runner. Run the server there (`deploy/install.sh`, docs/delivery.md §7),
keep its token as a CI secret, and send the runs to it:

```bash
signallab run tests/stage.json --server http://lab-pc:1430 --token-file token.txt --junit junit.xml
```

The experiment comes from the pipeline's checkout; the server runs it with its
own network, secrets and data folder, streams the steps back (`/api/run` with
NDJSON) and keeps the report, which `--report` downloads. The exit codes are the
same; a server that cannot be reached or refuses the token is `3`.

## 4. CI recipes

**GitHub Actions**, with the action from this repository (Linux runners; it runs
`signallab` from the image with host networking and the workspace mounted):

```yaml
jobs:
  signallab:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: ProAnima/SignalLab@v0.4.0
        with:
          experiments: tests/signallab/*.json
          params: |
            api=http://127.0.0.1:8080
        env:
          SIGNALLAB_SECRET_API_TOKEN: ${{ secrets.API_TOKEN }}
      - uses: actions/upload-artifact@v4
        if: always()
        with: { name: signallab-junit, path: signallab-junit.xml }
```

Inputs: `experiments`, `params`, `profile`, `server`, `token`, `junit`,
`timeout`, `version` (the image tag, default `latest`), `image`, `lang`;
outputs `junit` and `exit-code`. On a lab server instead:
`server: http://lab-pc:1430` and `token: ${{ secrets.SIGNALLAB_TOKEN }}`.

**With the binary** (any runner, Windows included):

```yaml
      - run: |
          curl -fsSL -o signallab.tgz https://github.com/ProAnima/SignalLab/releases/download/v0.4.0/signallab-0.4.0-linux-x64.tar.gz
          tar -xzf signallab.tgz && ./signallab run tests/signallab/smoke.json --junit junit.xml
```

**GitLab CI:**

```yaml
signallab:
  image:
    name: ghcr.io/proanima/signallab:0.4.0
    entrypoint: [""]
  script:
    - signallab run tests/signallab/*.json --junit signallab-junit.xml
  artifacts:
    when: always
    reports:
      junit: signallab-junit.xml
```

**Any shell or scheduler** (cron, Jenkins, a deploy script): `signallab run …`
and branch on its exit code; `--json` for a machine to read.

## 5. What is checked

| | Where |
| --- | --- |
| The run followed to its end, its steps live, a stop, the report | `engine/tests/run_to_end.rs` |
| `/api/run`: waited for, as lines, refused, a client that goes away, a shutdown; the token and `Origin` rules; `openapi.json` | `server/tests/run.rs` |
| The command line as a pipeline runs it: exit codes 0/1/2/3, JUnit, reports, `--json`, secrets never shown, Russian, sends that arrive on a socket, a library signal, a run on a server started in the test | `cli/tests/cli.rs` |
| Its messages: the dictionaries extracted whole (CRLF too), plural rules, number formats, the wording of failures | `cli/src/i18n.rs`, `cli/src/extract.rs` (unit tests) |
| `signallab` in the image: version, validate, run read-only, and against the running server | `scripts/image.mjs smoke` |
