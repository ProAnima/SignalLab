# Automation: the API, the command line and CI

Status: 2026-10-02.

The experiments that bring an installation up are also its regression tests.
Signal Lab runs them without a person in three ways: the **command line**
`signallab` (in this process, or on a lab server), **an assistant** (an LLM in
Claude Code, Claude Desktop, Cursor or VS Code, through `signallab mcp`) and the
**HTTP API** of `signal-lab-server`. Both go through the engine's own command table, so a run
in a pipeline is the run the app would make — same steps, same report, same
messages in English or Russian.

## 1. The command line

`signallab` comes with the desktop app: the Windows setup and the MSI put it
next to the app and that folder on `PATH`, the `.deb` and `.rpm` in `/usr/bin` —
so after installing the app it works in any new terminal. It is also in every
release on its own (`signallab-<version>-windows-x64.zip`,
`signallab-<version>-linux-x64.tar.gz`) and in the server image
(`/usr/local/bin/signallab`). From a checkout: `cargo run -p signal-lab-cli -- <command>`.

```bash
signallab run tests/smoke.json --param api=http://127.0.0.1:8080 --junit junit.xml
signallab run http-check poll-until-ready          # bundled templates by name
signallab validate tests/*.json                    # the editor's check, nothing sent
signallab send osc 127.0.0.1:9000 /cue/go f:0.75 s:main
signallab send http GET http://127.0.0.1:8080/health --expect-status 200
signallab fire "Fader value" --library signals.json
signallab emulate tests/orders-api.json --for 120  # play a dependency for two minutes
signallab emulators                                # the app's emulator library
signallab templates
signallab nodes                                    # what an experiment is made of (JSON)
signallab doctor                                   # what stands between Signal Lab and the gear
```

### `run` and `validate`

| Option | |
| --- | --- |
| `<FILE>…` | experiment files (any version the app reads; older ones are migrated) or bundled template names |
| `--param NAME=VALUE` | a parameter value for this run, repeatable; it applies to the experiments that have that parameter and must belong to at least one |
| `--profile NAME` | run with this profile (`""` = the defaults) |
| `--matrix NAME=V1,V2` (`-m`) | run once per value; repeat for more names — every combination runs, the first name varying slowest — or the same name again for more values; spaces around names and values are dropped |
| `--matrix-file PATH` | combinations from JSON: `{"NAME": [values], …}` (every combination; its names after `--matrix`'s, alphabetically) or `[{"NAME": value, …}, …]` (these, each crossed with `--matrix`) |
| `--fail-fast` | stop at the first run that does not pass; the rest are not started (said, counted as `not_started` in `--json`'s summary, and skipped suites of the JUnit report) |
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

**A matrix** runs one experiment against every target, every user, every
payload size. Each combination is a run of its own — named `file [host=a,
user=admin]`, a test suite of the JUnit report (with `param.NAME` properties),
a report of its own under `--report` (a folder) and, with `--json`, a `matrix`
object next to `file` (the file as given) on its `started`, `ended` and `error`
lines, and on `validate`'s. Every combination is checked before the first one
sends anything; an experiment without one of the matrix's parameters runs once,
not once per value it would ignore, and a value or combination given twice runs
once. At most 256 runs from one command.

```
signallab run smoke.json -m host=10.0.0.20:9000,10.0.0.21:9000 -m user=admin,guest --fail-fast --junit junit.xml
```

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
(`total`, `passed`, `failed`, `not_started`, `exit_code`). A failure that kept an
experiment from starting is `{"type":"error","file":…,"error":{code,params,…},"exit_code":2}`.
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
| `send http <METHOD> <URL> [-H 'Name: value'] [--body TEXT\|@file] [--expect-status N] [--timeout MS] [-u NAME:PASSWORD [--digest] \| --bearer TOKEN]` | prints the status line on stderr and the body on stdout; `-u` is Basic, with `--digest` the server's 401 challenge is answered (MD5, SHA-256); exit 1 when a Digest could not be answered, and why |
| `send mqtt <host:port> <topic> [payload] [--qos 0\|1\|2] [--retain]` | one publish, MQTT 3.1.1; an empty payload with `--retain` clears a retained value |
| `send ws <ws://…> [--text T \| --hex "de ad"] [-H 'Name: value'] [--protocol P]… [--expect TEXT \| --expect-regex RE \| --wait] [--timeout MS]` | connect, send, wait for the answer, close: the answer on stdout, the handshake on stderr; exit 1 when the expected answer does not come (`--wait` with nothing sent: the first message, a greeting) |
| `fire <id or name> [--library PATH]` | a signal of a library (default: the app's `Documents/SignalLab/signals.json`), by id, else by name |

They use the commands the app's screens use, so a fired signal is
byte-identical to one fired in the app.

**Git Bash on Windows** rewrites arguments that start with `/` into Windows
paths (`/cue/go` becomes `C:/Program Files/Git/cue/go`). Write `//cue/go`, or
run with `MSYS_NO_PATHCONV=1`; PowerShell and `cmd` are not affected.

### `emulate` and `emulators`

`signallab emulate <FILE|NAME>…` plays the other side — an HTTP API, an OSC, UDP or
TCP device, an MQTT broker — until Ctrl+C or `--for SECONDS`. A file holds one emulator (as a node's
`emulator` field and `signallab nodes` describe it), a list of them, or a library as
the app writes it; a name is the id or the name of one in the app's library
(`Documents/SignalLab/emulators.json`, or `--library PATH`).

| Option | |
| --- | --- |
| `--param NAME=VALUE` | a value its templates read as `{{NAME}}`, repeatable |
| `--bind IP:PORT` | listen there instead, when one emulator is given |
| `--for SECONDS` | stop after this long |
| `--seed N` | the seed of its random choices (a random order, jitter, generators) |
| `--check` | check the emulators and exit, without opening a port |
| `--server URL`, `--token-file PATH` | start them on a lab server, follow them through its API, stop them at the end |

Every request is printed as it is answered — `+   1.204 s Orders API  #2  GET
/orders/42 → 503 Service Unavailable · 0 B  0 ms  ← 127.0.0.1:53114` — and each
emulator ends with its counts: requests, how many no rule took, how many failed, how
many met an outage (`down`, when it has one), and the hits per rule. `--json` prints
`started`, every `exchange` (with what arrived: method, path, headers, body, JSON;
address and arguments; text; topic, levels and payload) and a `summary` per emulator. A port that is taken is exit code 3, an emulator that would not start 2.

A run with *Impairment* nodes ends with a line per relay too — what it received,
dropped and throttled, and each phase as forwarded / received:
`127.0.0.1:9010 → 127.0.0.1:9000: 160 datagrams, 41 dropped, 0 throttled · lan
0.0–2.0 s 40/40, wifi 2.0–4.0 s 39/40, offline 4.0–6.0 s 0/40, lan 6.0–8.0 s 40/40` —
and the report and `--json` carry them (`impairments`).

In a pipeline, the dependency runs in the background while the system under test is
tested against it:

```bash
signallab emulate tests/payments-mock.json --for 300 --json > mock.ndjson &
npm test                                   # the app, configured for http://127.0.0.1:18080
wait                                       # the emulator's counts are the last lines of mock.ndjson
```

Inside an experiment the *Emulator* node does the same for one run, and *Wait for
HTTP request* checks what arrived; `signallab run` prints each emulator's counts
after the run, and the report and `--json` carry them (`emulators`).

An HTTP node under **load** prints its progress once a second and, under its last
line, each threshold held or not (`✕ p95 < 100 ms · 152.58 ms`); a threshold that
does not hold fails the run (exit 1) and its JUnit case (`type="load.threshold"`,
the message the metric and the values). The step's `load` in `--json` and in the
report has every number: planned, sent, ok, failed, missed, rps, error_rate,
p50_ms…p99_ms, statuses, each second, the histogram and the verdicts.

`signallab emulators` lists the library: id, name, protocol, address and rules
(`--json` for the whole documents).

### `doctor` and `firewall allow`

`signallab doctor [--server URL --token-file PATH]` says what could stand
between Signal Lab and the gear, and exits 1 when something does: the network
this machine is on; the data folder; the firewall — on Windows, for
`signallab` and the desktop app each, whether a rule lets other machines in on
the current kind of network (private, domain, public) or one blocks them (a
*Cancel* at the system's prompt); on Linux, whether ufw or firewalld is on
and how to open a port; and with `--server`, whether that server answers and
takes the token.

`signallab firewall allow [--public]` fixes the Windows part: Windows asks for
administrator rights, then the inbound rules of `signallab` and the app (the
blocking ones included) are replaced by one allow rule each, on private and
domain networks — and public ones with `--public` (a venue's Wi-Fi often is
one). On Linux it prints the ufw or firewalld command for the ports you listen on.

## 2. For an assistant: `signallab mcp`

`signallab mcp` is a [Model Context Protocol](https://modelcontextprotocol.io)
server on stdio. An assistant can then build, check and run experiments and
talk to gear directly:

| Tool | |
| --- | --- |
| `describe_nodes` | the document format, every kind of node with its fields, outputs and an example, the `{{template}}` language |
| `list_templates`, `get_template` | the bundled experiments, to run or adapt |
| `validate_experiment` | the editor's check, nothing sent |
| `run_experiment` | a run to its end: every step, what came back, why it failed; progress while it runs; cancelling stops the run |
| `send_osc`, `send_udp`, `send_http`, `send_mqtt`, `send_ws` | one message (a WebSocket exchange: connect, send, the answer, close), as the app sends it |
| `listen` | what arrives on a UDP port for a while — OSC decoded, other datagrams as text and hex |
| `list_signals`, `fire_signal` | the user's signal library |
| `list_emulators` | the user's emulator library |
| `start_emulator` | an emulator (a document, or a library entry by id or name, `bind` to move it) answers until `stop_job` |
| `emulator_exchanges` | what a running emulator received and answered, rule by rule, with what each request carried (`after` for only the new ones) |
| `list_runs`, `compare_runs` | earlier runs read back from their reports (each load step's numbers and thresholds), and two of them side by side, a regression marked |
| `list_jobs`, `stop_job` | what is running |

Results are text for the model and the same as structured data; failures are
the engine's codes with the interface's words (`--lang ru` for Russian). Tools
that only read are marked read-only, so a client can let them run without
asking; everything that sends is marked as reaching the outside world.

`signallab mcp --print-config <client>` prints what a client needs, with this
executable's own path:

```bash
signallab mcp --print-config claude-code      # claude mcp add signallab -- "…\signallab.exe" mcp
signallab mcp --print-config claude-desktop   # the mcpServers entry for claude_desktop_config.json
signallab mcp --print-config cursor           # the same for .cursor/mcp.json
signallab mcp --print-config vscode           # the servers entry for .vscode/mcp.json
```

With `--server http://lab-pc:1430` (and `SIGNALLAB_TOKEN` in the client's
environment) the runs and sends happen on the lab server, through its API —
the gear only the lab can reach; `listen` stays on this machine. From the image:
`docker run -i --rm --network host --entrypoint signallab ghcr.io/proanima/signallab mcp`.
`--library PATH` names a signal library other than the app's, `--emulators PATH`
an emulator library. Runs keep their
reports in the app's data folder (`Documents/SignalLab/runs`, or `--data-dir`),
so they stay after the session and are where the app keeps its own.

## 3. The HTTP API

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

## 4. Runs on a lab server

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

## 5. CI recipes

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

Inputs: `experiments`, `params`, `profile`, `matrix` (`NAME=V1,V2`, one per
line), `matrix-file`, `fail-fast`, `server`, `token`, `junit`,
`timeout`, `version` (the image tag, default `latest`), `image`, `lang`,
`fail-on-error` (`"false"` to go on after a failed run and branch on `exit-code` —
GitHub hands on no outputs of an action that failed); outputs `junit` and
`exit-code`. On a lab server instead:
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

## 6. What is checked

| | Where |
| --- | --- |
| The run followed to its end, its steps live, a stop, the report | `engine/tests/run_to_end.rs` |
| `/api/run`: waited for, as lines, refused, a client that goes away, a shutdown; the token and `Origin` rules; `openapi.json` | `server/tests/run.rs` |
| The command line as a pipeline runs it: exit codes 0/1/2/3, JUnit, reports, `--json`, secrets never shown, Russian, sends that arrive on a socket, a library signal, a run on a server started in the test | `cli/tests/cli.rs` |
| Its messages: the dictionaries extracted whole (CRLF too), plural rules, number formats, the wording of failures | `cli/src/i18n.rs`, `cli/src/extract.rs` (unit tests) |
| `signallab` in the image: version, validate, run read-only, and against the running server | `scripts/image.mjs smoke` |
| Every kind of node in one experiment, passing here and on a server against an HTTP API, a TCP sink, an OSC and a UDP device that answer, and an MQTT broker | `cli/tests/nodes.rs` |
| `signallab mcp` as a client drives it: every tool, progress, a cancelled run that stops, protocol errors, `--server`, `--print-config` | `cli/tests/mcp.rs` |
| The node catalogue the assistant reads: every kind the engine has, each example accepted with its outputs | `cli/src/catalog.rs` (unit tests) |
| `signallab doctor` with a server whose token is right and one whose token is not | `cli/tests/doctor.rs` |
| Emulators: routes, sequences, a seeded mix, faults, OSC/UDP/TCP rules, the library | `engine/src/emulator*.rs` (unit tests) |
| An Emulator node retried against until it answers, a webhook a wait reads, a port shared with waits, ports checked before the first step, the commands | `engine/tests/emulators.rs` |
| `signallab emulate` in the background, here and on a server; `--check`, a taken port, library names | `cli/tests/emulate.rs` |
| `start_emulator`, `emulator_exchanges`, `list_emulators` as a client calls them | `cli/tests/mcp.rs` |
