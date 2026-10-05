---
title: Command line
description: signallab runs experiments without a window, sends single messages, plays emulators and checks the network, from a terminal, a script or a CI job.
---

# The command line: `signallab`

`signallab` is Signal Lab without a window. It runs experiments to their end and
exits with a code a script understands, sends one OSC message, datagram, HTTP
request, WebSocket message or MQTT publish, fires a signal from your library,
plays an emulator until you stop it, and says what stands between this machine
and the gear.

It is the same engine as the app: a run from the command line takes the same
steps, writes the same report and says the same things, in the interface's
languages. With `--server` the runs happen on a [Signal Lab server](../server/index.md)
instead, with its network and its secrets.

```bash
signallab run tests/smoke.json --param api=http://127.0.0.1:8080 --junit junit.xml
signallab run flaky-api                                # a bundled template, by name
signallab validate tests/*.json                        # the editor's check, nothing sent
signallab send osc 127.0.0.1:9000 /cue/go f:0.75 s:main
signallab send http GET http://127.0.0.1:8080/health --expect-status 200
signallab fire "Fader value"
signallab emulate tests/orders-api.json --for 120      # play a dependency for two minutes
signallab doctor                                       # firewall, network, data folder
```

For pipelines, see [Signal Lab in CI](ci.md); for an AI assistant,
[`signallab mcp`](mcp.md).

## Installing {#install}

| Where | How you get `signallab` |
| --- | --- |
| Windows, the setup (`.exe`) | Installed next to the app, and that folder is added to `PATH` — yours for an install "for me", the machine's for "for everyone". Open a new terminal after installing. The setup's `/NOPATH` switch leaves `PATH` alone. |
| Windows, the `.msi` | Installed next to the app; the install folder is on the machine's `PATH` while the app is installed. |
| Linux, `.deb` and `.rpm` | `/usr/bin/signallab`. |
| Linux, AppImage | Not included: use the archive below. |
| Any machine, without the app | Every release carries `signallab-<version>-windows-x64.zip` and `signallab-<version>-linux-x64.tar.gz`, each with the program and its license; `SHA256SUMS.txt` on the same page lists their checksums. |
| The server image | `/usr/local/bin/signallab` in `ghcr.io/proanima/signallab` (see [CI](ci.md#docker)). |

Check it with:

```bash
signallab version
```

## Commands {#commands}

| Command | What it does |
| --- | --- |
| [`run`](#cli-run) | Runs experiments one after another; exits 0 only when every one passed. |
| [`validate`](#cli-validate) | Checks experiments as the editor does before a run; sends nothing. |
| [`send`](#cli-send) | Sends one message: `osc`, `udp`, `http`, `ws` or `mqtt`. |
| [`fire`](#cli-fire) | Sends a signal of a signal library, by its id or name. |
| [`emulate`](#cli-emulate) | Plays an HTTP API, an OSC, UDP or TCP device, or an MQTT broker until <kbd>Ctrl</kbd>+<kbd>C</kbd> or `--for`. |
| [`emulators`](#cli-emulators) | Lists the emulators of the app's library. |
| [`templates`](#cli-templates) | Lists the bundled experiment templates. |
| [`nodes`](#cli-nodes) | Describes every kind of node, as JSON. |
| [`mcp`](#cli-mcp) | Serves Signal Lab to an AI assistant over the Model Context Protocol. |
| [`doctor`](#cli-doctor) | Checks the firewall, the network, the data folder and a server. |
| [`firewall`](#cli-firewall) | `firewall allow`: lets other machines reach Signal Lab through the Windows Firewall. |
| [`version`](#cli-version) | Prints the version. |

`signallab help <command>` or `signallab <command> --help` prints a command's
options.

## Options of every command {#global-options}

| Option | What it does | Default |
| --- | --- | --- |
| `--lang <code>` | The language of messages: `en`, `ru`, `es`, `fr`, `de`, `pt`, `zh`, `ja`, `ko`, `hi` or `ar`. | `SIGNALLAB_LANG`, else the locale, else `en` |
| `--json` | Output for machines on stdout instead of text (see [Output](#output)). | off |
| `-h`, `--help` | The command's help. | |
| `-V`, `--version` | The version; before any command, as `signallab --version`. | |

Both `--lang` and `--json` may go anywhere on the line:
`signallab --json run smoke.json` and `signallab run smoke.json --json` are the same.

### Language {#language}

Messages, step texts, failures and the JUnit report use the interface's own
texts, plural forms and number style. The language is the first of:

1. `--lang`;
2. `SIGNALLAB_LANG` (`ru`, `ru-RU` and `ru_RU.UTF-8` all mean Russian);
3. the locale: the first of `LC_ALL`, `LC_MESSAGES` and `LANG` that is set;
4. English.

::: tip
Windows terminals usually set none of the locale variables, so `signallab`
speaks English there unless you set `SIGNALLAB_LANG` or pass `--lang`.
:::

### Output for people and for machines {#output}

Without `--json`, results go to **stdout** and everything a person reads along
the way goes to **stderr**: the steps of a run as they happen, why something
failed, where the report is. `signallab run … 2>/dev/null` leaves one verdict
line per run.

With `--json`, stdout carries JSON and nothing else, and stderr stays quiet:

| Command | What `--json` prints on stdout |
| --- | --- |
| `run` | One object per line: `started`, a `step` per step, `ended` (the run's result), then `summary`; `error` for a run that could not start. See [What `run` prints](#run-output). |
| `validate` | One object per line, per experiment: `valid`, and `profile_issues` or `error`. |
| `send`, `fire` | `{"type": "sent", "result": …}`; `send http` prints `{"type": "response", "response": …}`, `send ws` `{"type": "exchange", "result": …}`. |
| `emulate` | One object per line: `started`, an `exchange` per request, a `summary` per emulator; `valid` with `--check`. |
| `emulators` | `{"path": …, "emulators": [{id, name, protocol, bind, rules, note}, …]}`. |
| `templates` | `[{"name": …, "experiment": …}, …]`. |
| `doctor` | One object: `version`, `network`, `data_dir`, `firewall`, `server`, `problems`. |
| `version` | `{"version": "…"}`. |
| `nodes` | Always JSON, with or without `--json`. |

A failure is `{"type": "error", "error": {…}, "exit_code": N}`. `error` is the
engine's error: a stable `code` (such as `transport.refused` or
`secret.missing`), its `params`, and the `node` and `field` it is about. A
script can branch on `error.code` in any language; the texts for every code are
listed in [Error messages](../reference/errors.md).

### Exit codes {#exit-codes}

| Code | Meaning |
| --- | --- |
| `0` | Every experiment passed; the send succeeded; nothing is in the way. |
| `1` | An experiment ran and failed, ran out of time or was stopped; a send failed (refused, no answer, an unexpected status); `doctor` found something in the way. |
| `2` | The command or a document is wrong: an argument, a file that cannot be read, a validation error, an unknown parameter, a missing secret. |
| `3` | Nothing could run for a reason outside the experiment: the server cannot be reached or refuses the token, a port cannot be opened, the credential store fails. |

With several experiments the most serious result decides, in this order: `2`,
then `3`, then `1`, then `0`.

### Environment variables {#environment}

| Variable | What it does |
| --- | --- |
| `SIGNALLAB_LANG` | The language, when `--lang` is not given. |
| `LC_ALL`, `LC_MESSAGES`, `LANG` | The language, when neither of the above is set. |
| `SIGNALLAB_SERVER` | The server for `run`, `validate`, `emulate`, `mcp` and `doctor`, as `--server` gives it. |
| `SIGNALLAB_TOKEN` | The server's access token, when no token file is given. |
| `SIGNALLAB_TOKEN_FILE` | A file holding the server's token, as `--token-file` gives it. |
| `SIGNALLAB_SECRET_<NAME>` | The value of the secret `NAME` for runs in this process (see [Secrets](#secrets)). |
| `SIGNALLAB_DATA_DIR` | The app's data folder, where `fire`, `emulators`, `emulate`, `mcp` and `doctor` look by default; otherwise `Documents/SignalLab` in your home folder. |
| `GITHUB_ACTIONS` | When it is `true`, a run that does not pass is also printed as an `::error` annotation, which GitHub shows on the run's page. |

## `run` {#cli-run}

```text
signallab run [OPTIONS] <FILE>...
```

Runs experiments one after another and exits `0` only when every one passed.

| Option | What it does | Default |
| --- | --- | --- |
| `<FILE>...` | Experiment files, or names of [bundled templates](#cli-templates). | required |
| `-p`, `--param NAME=VALUE` | A parameter value for this run; repeat for more. | the document's values |
| `--profile NAME` | Run with this profile; every experiment given must have it. `""` runs with the defaults. | the document's profile |
| `-m`, `--matrix NAME=V1,V2` | Run once per value; repeat for more names (see [A matrix of runs](#matrix)). | |
| `--matrix-file PATH` | Combinations from a JSON file. | |
| `--seed N` | The seed of the random values, 0 to 9007199254740991 (2⁵³ − 1). | the document's seed, else a new one per run |
| `--timeout SECONDS` | Fail a run that takes longer, 1–300. | `300` |
| `--fail-fast` | Stop at the first run that does not pass; the rest are not started. | off |
| `--junit PATH` | Write a JUnit XML report there (see [Reports](#reports)). | |
| `--report PATH` | Copy the run report there: a file for one run, a folder for several. | |
| `--data-dir PATH` | The data folder for this process's runs; their reports stay there. | a temporary folder, removed at exit |
| `--server URL` | Run on this server instead of in this process (see [On a server](#run-on-server)). | `SIGNALLAB_SERVER` |
| `--token-file PATH` | A file holding the server's token. | `SIGNALLAB_TOKEN_FILE`, else `SIGNALLAB_TOKEN` |
| `--secrets files\|system` | Where secret values come from in this process. | `files` |
| `--secrets-dir PATH` | A folder of secret files, one per name. | `/run/secrets/signallab` when it exists |

`--data-dir`, `--secrets` and `--secrets-dir` are about runs in this process;
they cannot be combined with `--server`.

### Files and templates {#run-inputs}

A `FILE` is an experiment as the app saves and exports it
([[ui:exp.exportJson]] on the [[ui:nav.experiment]] screen). Any document
version the app opens works; an older one is brought up to date as it is read,
as the app does when it opens it. A name that is not a file is looked up among
the [bundled templates](#cli-templates), with or without `.json`:

```bash
signallab run tests/stage-cues.json tests/api.json
signallab run osc-ping-reply --param device=192.0.2.20:9000
```

Every experiment — and every combination of a matrix — is checked as the editor
checks it before the first one runs. A broken third file stops the first one
too, before anything is sent, with exit code `2`.

### Parameters and profiles {#run-params}

`--param NAME=VALUE` sets a parameter for this run only; the file is not
changed. The value is everything after the first `=`, so
`--param url=http://127.0.0.1/?q=1` works, and `--param note=` sets an empty
value. A value applies to every experiment given that has that parameter, and
must name a parameter of at least one of them — a misspelt name is refused
with exit code `2`.

`--profile NAME` runs with one of the experiment's profiles, as choosing it
under [[ui:exp.profile]] in the editor does. See
[Data and templates](../experiments/data.md).

### A matrix of runs {#matrix}

A matrix runs the same experiment against every target, every user, every
payload size. Each combination is a run of its own, with its own result, its
own report and its own suite in the JUnit report.

```bash
signallab run smoke.json \
  -m device=192.0.2.20:9000,192.0.2.21:9000 \
  -m user=admin,guest \
  --fail-fast --junit junit.xml
```

That is four runs: `device` varies slowest, `user` fastest. They are named
after the file and their values: `smoke.json [device=192.0.2.20:9000, user=admin]`.

- `--matrix NAME=V1,V2` adds an axis. The same name again adds values to it.
  Spaces around names and values are dropped; a value given twice runs once.
- `--matrix-file PATH` reads JSON in one of two shapes:

  ```json
  { "device": ["192.0.2.20:9000", "192.0.2.21:9000"], "retries": [1, 3] }
  ```

  adds axes — every combination runs, these names after `--matrix`'s, in
  alphabetical order;

  ```json
  [
    { "device": "192.0.2.20:9000", "user": "admin" },
    { "device": "192.0.2.21:9000", "user": "guest" }
  ]
  ```

  lists the combinations themselves, each crossed with the `--matrix` axes.
  Values are text, numbers or `true`/`false`; a comma inside a value in a file
  stays part of it.
- Every matrix name must be a parameter of at least one experiment given. An
  experiment without one of them runs once, not once per value it would
  ignore.
- A name set by both `--param` and the matrix, or by both `--matrix` and the
  file, is refused.
- At most **256** runs from one command; more is refused before anything
  runs.

### Running on a server {#run-on-server}

```bash
signallab run tests/stage.json --server http://192.0.2.10:1430 --token-file token.txt
```

The experiment comes from this machine; the server runs it with its own
network, its own [secrets](../server/index.md#secrets) and its own data folder,
and streams the steps back as they happen. The report stays on the server —
its path is printed — and `--report` downloads a copy. A run on a server is a
job there like one started in its interface: every page signed in to it shows
it. If `signallab` goes away mid-run, the run still finishes on the server and
keeps its report.

The token is read from `--token-file` (or `SIGNALLAB_TOKEN_FILE`), else from
`SIGNALLAB_TOKEN`, and sent as `Authorization: Bearer`. A server that cannot be
reached, refuses the token or stops answering mid-run is exit code `3`.
`signallab doctor --server URL` checks the address and the token on their own.

### Reports {#reports}

Each run writes the same report the app writes. Without `--data-dir` the runs
use a temporary folder that is removed when `signallab` exits, so keep what you
need:

- `--report PATH` copies the report: to `PATH` itself for one run, or into the
  folder `PATH` for several, as `01-<file>.json`, `02-<file>.json`, … in the
  order they ran.
- `--data-dir PATH` keeps every run's report in that folder (under `runs/`),
  and prints where each one is.

`--junit PATH` writes a JUnit XML report, the format every CI system reads:

- a `<testsuite>` per run (per combination of a matrix), with the file, the
  seed, the outcome, the profile, the report's path and each matrix value
  (`param.NAME`) as properties;
- a `<testcase>` per node that ran, named after the node's type and its id,
  with its own time;
- a `<failure>` on the node that failed: the message in the chosen language,
  the error code as its `type`, and the node's steps with the technical detail;
- a `<skipped>` case for each node the run never reached (the other side of a
  branch);
- a suite with one `<error>` case for an experiment that could not start, and
  one with a `<skipped>` case for each run `--fail-fast` did not start.

```xml
<testsuites name="Signal Lab" tests="6" failures="1" errors="0" time="2.006">
  <testsuite name="HTTP to OSC" tests="6" failures="1" errors="0" skipped="4" time="2.006" timestamp="2026-10-04T16:48:03">
    <properties>
      <property name="file" value="status-branch" />
      <property name="seed" value="42" />
      <property name="outcome" value="failed" />
      <property name="report" value="out/report.json" />
    </properties>
    <testcase name="Start (start)" classname="HTTP to OSC" time="0.001">
      <system-out>Started · seed 42</system-out>
    </testcase>
    <testcase name="HTTP request (request)" classname="HTTP to OSC" time="2.004">
      <failure message="http://127.0.0.1:8080/ refused the connection — nothing is listening on that port" type="transport.refused">…</failure>
    </testcase>
    <testcase name="Status branch (branch)" classname="HTTP to OSC" time="0.000">
      <skipped message="not reached in this run" />
    </testcase>
    …
  </testsuite>
</testsuites>
```

An HTTP node under [load](../experiments/load.md) adds a line under its last
step for each threshold, held or not (`✕ p95 < 100 ms · 152.58 ms`); a
threshold that does not hold fails the run and its JUnit case (`type="load.threshold"`).

### Secrets {#secrets}

An experiment reads a secret as `{{secret.NAME}}`. For runs in this process the
value comes from:

| `--secrets` | Where the value of `NAME` comes from |
| --- | --- |
| `files` (the default) | The environment variable `SIGNALLAB_SECRET_NAME`; else a file named `NAME` in `--secrets-dir` (by default `/run/secrets/signallab`, when that folder exists — the Docker secrets layout). |
| `system` | The Windows Credential Manager — where the app keeps the values of its [[ui:exp.secrets]]. Linux has no credential store `signallab` reads: there `--secrets system` fails with exit code `3`. |

A file's trailing line break is not part of the value, and an empty value
counts as not set. Names are letters, digits and `_`, not starting with a
digit, up to 128 characters; a value is at most 16 KiB.

```bash
SIGNALLAB_SECRET_API_TOKEN="$API_TOKEN" signallab run tests/api.json
```

A secret that is not set stops the run before any traffic, with exit code `2`
and the name that is missing. A value is never printed: steps, errors, reports
and the JUnit report show `••••` in its place. With `--server`, secrets are the
server's.

### What `run` prints {#run-output}

As a run goes, each step is a line on stderr — its time since the start, the
node, its state and what it did — like the app's timeline. When it ends, one
line on stdout says how it went:

```text
▶ HTTP check (http-check) · seed 1185927457137919
     0.000  Start         Running
     0.000  Start         Passed · Started · seed 1185927457137919
     0.000  HTTP request  Running
     2.004  HTTP request  Failed · URL — http://127.0.0.1:8080/ refused the connection — nothing is listening on that port
✖ HTTP check failed after 2 s: HTTP request · URL — http://127.0.0.1:8080/ refused the connection — nothing is listening on that port
  Technical details: error sending request for url (http://127.0.0.1:8080/): … (os error 10061)
  To run it again with the same random values: --seed 1185927457137919
```

A failed run names the seed it used: `--seed` with that number runs it again
with the same random values. After the verdict come, on stderr, what each
emulator of the run was asked — requests, how many no rule took, how many
failed, and the hits per rule:

```text
✔ Retry a flaky API passed in 7 ms
  Flaky API: 3 requests, 0 without a rule, 0 failed · #1 3
```

and what each impairment relay did: what it received, dropped and throttled,
and each phase as forwarded / received:

```text
  127.0.0.1:19110 → 127.0.0.1:19100: 155 datagrams, 38 dropped, 0 throttled · lan 0.0–2.0 s 39/39, wifi 2.0–4.0 s 39/39, offline 4.0–6.0 s 0/38, lan 6.0–8.0 s 38/38
```

A relay over TCP moves chunks of streams, not datagrams, and drops nothing, so
its line counts chunks, connections, resets, half-open connections and the
times a stream was held back by the bandwidth limit:

```text
  127.0.0.1:19120 → 127.0.0.1:19101: 14 chunks, 2 connections, 1 reset, 0 half-open, 3 held back
```

An HTTP node under [load](../experiments/load.md) reports its progress as a
step at most once a second.

With several runs, a line per run and a count end the output:
`3 runs: 2 passed, 1 failed`. With `--fail-fast`, the runs it did not start are
counted on stderr.

With `--json` every line is an object with a `type`:

```json
{"type":"started","experiment":"Empty experiment","file":"empty","job_id":1,"overridden":false,"profile":null,"seed":1,"started_ms":1791132204585}
{"type":"step","node_id":"start","state":"passed","detail":"Started","message_key":"exp.step.started","message_params":{"seed":1},"job_id":1,"ts":1791132204585}
{"type":"ended","experiment":"Empty experiment","file":"empty","outcome":"passed","seed":1,"params":{},"steps":[…],"report_path":"…","started_ms":1791132204585,"ended_ms":1791132204585,…}
{"type":"summary","total":1,"passed":1,"failed":0,"not_started":0,"exit_code":0}
```

`started`, `ended` and `error` lines carry `file` (the argument as given) and,
in a matrix, `matrix` (the combination's values). `ended` is the run's whole
result: `outcome` (`passed`, `failed` or `stopped`), `seed`, `profile`,
`params`, `error`, every step, `emulators` and `impairments` when the run had
them, and `report_path`. The last step of a load carries `load` with every
number it measured: `planned`, `sent`, `ok`, `failed`, `missed`, `rps`,
`error_rate`, `min_ms`, `mean_ms`, `max_ms`, `p50_ms` to `p99_ms`, `statuses`,
each second (`seconds`), the `histogram`, and each threshold's verdict
(`thresholds`).

## `validate` {#cli-validate}

```text
signallab validate [OPTIONS] <FILE>...
```

Checks experiments the way the editor does before a run — the graph, every
field, templates, parameters and secrets — and sends nothing. Exits `0` when
every one would start.

It takes the inputs of [`run`](#cli-run): files and templates, `--param`,
`--profile`, `--matrix`, `--matrix-file`, and `--server`, `--token-file`,
`--secrets`, `--secrets-dir`. With `--server` the server checks them, against
its own secrets.

```text
✔ tests/stage.json: Stage cues would run
  Rehearsal: Would not run: …
```

A problem that only another profile of the document has is listed under it,
but does not fail the check. With `--json`, one line per experiment (per
combination): `{"experiment", "file", "valid": true, "profile_issues": […]}`,
or `"valid": false` with `error` and `exit_code`.

## `send` {#cli-send}

```text
signallab send <osc|udp|http|ws|mqtt> …
```

Sends one message through the same commands the app's screens use, so it is
the same bytes on the wire. A send reads no library and no secrets.

Exit codes: `0` sent, `1` the send failed, `2` an argument is wrong.

A `<host:port>` is an IP address or a host name, and a port: `127.0.0.1:9000`,
`[::1]:9000` (an IPv6 address in brackets) or `device.local:9000`. A name is
looked up when the command runs, its IPv4 address taken when it has one, so
`localhost:9000` reaches a receiver on `127.0.0.1`. A name that does not
resolve fails the send (`1`); a target with no port is an invalid argument
(`2`).

### `send osc` {#cli-send-osc}

```text
signallab send osc <host:port> <address> [ARG]...
```

One OSC message. Arguments are typed with a prefix, or inferred:

| Argument | OSC type |
| --- | --- |
| `i:3` | int32 |
| `f:0.5` | float32 |
| `d:1.5` | float64 (double) |
| `h:64` | int64 |
| `s:text` | string |
| `b:de ad be ef` | blob, as hex bytes |
| `T`, `F` | true, false |
| `N` | nil |
| `3`, `-3` | a plain integer is int32 |
| `2.5` | a plain decimal is float32 |
| anything else | string |

```bash
signallab send osc 127.0.0.1:9000 /cue/go f:0.75 s:main 3
# ✔ sent /cue/go (32 bytes) → 127.0.0.1:9000
```

Quote an argument with spaces: `"s:hello world"`. `s:7` sends the text `7`. The
address must start with `/`; one that does not is an invalid argument (`2`).

::: warning Git Bash on Windows
Git Bash rewrites arguments that start with `/` into Windows paths, so
`/cue/go` arrives as `C:/Program Files/Git/cue/go`. Write `//cue/go`, or run
with `MSYS_NO_PATHCONV=1`. PowerShell and `cmd` are not affected.
:::

See [OSC](../protocols/osc.md).

### `send udp` {#cli-send-udp}

```text
signallab send udp <host:port> (--text TEXT | --hex HEX)
```

One UDP datagram. Give the payload as `--text`, or as `--hex` bytes:
`"de ad be ef"`, `deadbeef` or `0xDE,0xAD`.

```bash
signallab send udp 127.0.0.1:7000 --text "PLAY 1"
signallab send udp 127.0.0.1:7000 --hex "de ad be ef"
```

### `send http` {#cli-send-http}

```text
signallab send http <METHOD> <URL> [OPTIONS]
```

One HTTP request. The status line goes to stderr — `HTTP 200 OK · 3 ms · 1,234 B`
— and the response body to stdout, so it can be piped on. A body longer than
256 KiB is cut there, and stderr says so.

| Option | What it does | Default |
| --- | --- | --- |
| `-H`, `--header "Name: value"` | A request header; repeat for more. | |
| `--body TEXT` | The request body. `@FILE` sends a text file's contents. | none |
| `--expect-status STATUS` | Exit `1` unless the response has this status. | any status is `0` |
| `--timeout MS` | Milliseconds to wait for the response. | `10000` |
| `-u`, `--user NAME:PASSWORD` | Credentials, sent as Basic. | |
| `--digest` | With `--user`: answer the server's Digest challenge instead (MD5 or SHA-256). | off |
| `--bearer TOKEN` | Send `Authorization: Bearer TOKEN`. Not with `--user`. | |

```bash
signallab send http GET http://127.0.0.1:8080/health --expect-status 200
signallab send http POST http://127.0.0.1:8080/cue -H "Content-Type: application/json" --body '{"cue": 1}'
signallab send http GET http://127.0.0.1:8080/admin -u admin:secret --digest
```

Without `--expect-status`, any response is a success, a `500` included. A
request that gets no response (refused, timed out, a name that does not
resolve) is exit code `1`, and so is a Digest challenge that could not be
answered — the reason is printed after the response.

::: tip
Arguments are visible to other users of the same machine. Keep real passwords
for experiments, where they are [secrets](#secrets).
:::

See [HTTP](../protocols/http.md).

### `send ws` {#cli-send-ws}

```text
signallab send ws <URL> [OPTIONS]
```

One WebSocket exchange: connect to `ws://…` or `wss://…`, send a message,
optionally wait for the answer, close. The handshake and what was sent go to
stderr, the answer to stdout (binary messages as hex).

| Option | What it does | Default |
| --- | --- | --- |
| `--text TEXT` | Send this text message. | nothing sent |
| `--hex HEX` | Send these bytes as a binary message. Not with `--text`. | |
| `-H`, `--header "Name: value"` | A header for the upgrade request; repeat for more. | |
| `--protocol NAME` | A subprotocol to offer; repeat for more, in order of preference. | |
| `--expect TEXT` | Wait for a message containing this text. | |
| `--expect-regex REGEX` | Wait for a message matching this regular expression. | |
| `--wait` | Wait for any message. | |
| `--timeout MS` | Milliseconds to wait for the answer. | `2000` |

```bash
signallab send ws ws://127.0.0.1:9001/echo --text '{"ping": 1}' --expect '"ping"'
signallab send ws ws://127.0.0.1:9001/feed --wait        # nothing sent: the server's first message
```

Without `--expect`, `--expect-regex` or `--wait` it connects, sends and closes
without waiting. When the expected answer does not come in time, the exit code
is `1`. See [WebSocket](../protocols/websocket.md).

### `send mqtt` {#cli-send-mqtt}

```text
signallab send mqtt <host:port> <topic> [payload] [--qos 0|1|2] [--retain]
```

One MQTT 3.1.1 publish over plain TCP, without credentials, as a client of its
own with a fresh client id, so it never knocks off a connection that is already
there. Without a port the broker is on `1883`. A topic is one topic: with a `+`
or `#` in it, or empty, the command is refused before it connects (`2`).

| Option | What it does | Default |
| --- | --- | --- |
| `[payload]` | The payload. | empty |
| `--qos 0\|1\|2` | The quality of service. | `0` |
| `--retain` | Keep it as the topic's retained value. An empty payload with `--retain` clears it. | off |

```bash
signallab send mqtt 127.0.0.1:1883 lab/light/1/set on --qos 1
signallab send mqtt 127.0.0.1 lab/light/1/state "" --retain     # clear the retained value
```

See [MQTT](../protocols/mqtt.md).

## `fire` {#cli-fire}

```text
signallab fire <signal> [--library PATH]
```

Sends a signal of a signal library exactly as the app's [[ui:nav.signals]]
screen fires it: OSC, UDP, HTTP and MQTT signals. The signal is found by its
id, else by its name, ignoring case; a name several signals share is refused
with their ids.

| Option | What it does | Default |
| --- | --- | --- |
| `<signal>` | The signal's id or name. | required |
| `--library PATH` | The library file. | `signals.json` in the app's data folder |

```bash
signallab fire "Fader value"
signallab fire go --library show/signals.json
```

The library is only read, never created or changed. See
[Signals](../tools/signals.md).

## `emulate` {#cli-emulate}

```text
signallab emulate [OPTIONS] <FILE|NAME>...
```

Plays the other side — an HTTP API, an OSC, UDP or TCP device, an MQTT broker —
until <kbd>Ctrl</kbd>+<kbd>C</kbd> or `--for`, and prints every request as it is
answered. A `FILE` holds one emulator, a list of them, or a whole library as the
app writes it; a `NAME` is the id or name of one in the app's library (the
[[ui:nav.emulators]] screen's).

| Option | What it does | Default |
| --- | --- | --- |
| `-p`, `--param NAME=VALUE` | A value its templates read as `{{NAME}}`; repeat for more. | |
| `--bind IP:PORT` | Listen there instead. Only with one emulator. | the emulator's own |
| `--for SECONDS` | Stop after this long. | until <kbd>Ctrl</kbd>+<kbd>C</kbd> |
| `--seed N` | The seed of its random choices: a random order, jitter, generators. | |
| `--check` | Check the emulators and exit, without opening a port. | off |
| `--library PATH` | The library names are looked up in. | `emulators.json` in the app's data folder |
| `--server URL`, `--token-file PATH` | Start them on a server, follow them through its API, and stop them at the end. | |
| `--secrets`, `--secrets-dir` | As for [`run`](#cli-run). | |

```text
$ signallab emulate tests/orders-api.json --for 60
Orders API (http) answering on 127.0.0.1:18099
answering for 60 s
+   1.209 s Orders API  #1  GET /orders/42 → 200 OK · 12 B  1 ms  ← 127.0.0.1:55744
+   1.209 s Orders API  —  GET /nothing → 404 Not Found · 20 B  2 ms  ← 127.0.0.1:55745
Orders API: 2 requests, 1 without a rule, 0 failed · #1 1
```

Each line is the time since the start, the emulator, the rule that answered
(`#1`, or `—` for none), the request and what it got, the time it took and who
sent it. At the end each emulator's counts: requests, how many no rule took,
how many failed, how many met an outage or went undelivered when there were
any, and the hits per rule.

Exit codes: `0` when it stops at <kbd>Ctrl</kbd>+<kbd>C</kbd> or `--for`; `2`
when an emulator is not valid; `3` when its port is taken or cannot be opened,
or a socket fails while it answers.

In a pipeline, start it in the background, test the system against it, and read
the counts at the end:

```bash
signallab emulate tests/payments-mock.json --for 300 --json > mock.ndjson &
npm test          # the system under test, configured for the emulator's address
wait              # the last lines of mock.ndjson are the counts
```

On a server, an emulator started with `--server` is stopped when `signallab`
ends normally; one left behind by a killed process can be stopped from the
server's interface. See [Emulators](../tools/emulators.md).

## `emulators` {#cli-emulators}

```text
signallab emulators [--library PATH]
```

Lists the emulators of the library: id, name, protocol, address and how many
rules. `--library PATH` reads another library file instead of `emulators.json`
in the app's data folder. `signallab emulate <id>` starts one.

The library is only read. If there is no such file — the app makes the one in
its data folder the first time it starts — the command says so and exits `2`.

## `templates` {#cli-templates}

```text
signallab templates
```

Lists the bundled templates, which `run` and `validate` take by name. They are
the app's own:

| Name | In the app | Parameters |
| --- | --- | --- |
| `empty` | [[ui:exp.templateEmpty]] | |
| `http-check` | [[ui:exp.templateHttp]] | |
| `status-branch` | [[ui:exp.templateBranch]] | |
| `parallel-flows` | [[ui:exp.templateParallel]] | |
| `osc-ping-reply` | [[ui:exp.templatePingReply]] | `device` = `127.0.0.1:9000` |
| `poll-until-ready` | [[ui:exp.templatePoll]] | `device` = `127.0.0.1:9000` |
| `flaky-api` | [[ui:exp.templateFlaky]] | `api` = `http://127.0.0.1:18080` |
| `fault-phases` | [[ui:exp.templateFaults]] | |
| `dependency-outage` | [[ui:exp.templateOutage]] | `api` = `http://127.0.0.1:18090` |
| `websocket-echo` | [[ui:exp.templateWsEcho]] | `service` = `ws://127.0.0.1:9001/echo` |

Every template points at loopback. `flaky-api`, `fault-phases` and
`dependency-outage` bring their own emulators, so they run with nothing else
listening — a quick way to see `signallab` work.

## `nodes` {#cli-nodes}

```text
signallab nodes
```

Prints, as JSON, what an experiment is made of: the document's shape and rules,
every kind of node with its label, description, fields, outputs and an example
the engine accepts, the `{{template}}` language, load profiles and the emulator
document. It is what an assistant reads through [`signallab mcp`](mcp.md) to
write an experiment; for people, [Nodes](../experiments/nodes.md) says the same
with more words.

## `mcp` {#cli-mcp}

```text
signallab mcp [OPTIONS]
```

Serves Signal Lab to an AI assistant over the Model Context Protocol, on stdin
and stdout. All of it — setting it up in Claude Code, Claude Desktop, Cursor or
VS Code, its options and its tools — is on [Assistants (MCP)](mcp.md).

## `doctor` {#cli-doctor}

```text
signallab doctor [--server URL] [--token-file PATH]
```

Says what could stand between Signal Lab and the gear, and exits `1` when
something does. Its lines follow [`--lang`](#language) like the rest of the
command line:

- the network this machine is on: its name and address;
- the data folder: whether it can be written (one not made yet is fine — the
  app makes it on first use);
- the firewall: on Windows, for `signallab` and the desktop app each, whether a
  rule lets other machines in on the kind of network the machine is on now
  (private, domain or public), or a rule blocks them — what a *Cancel* at the
  system's prompt leaves behind; on Linux, whether ufw or firewalld is on, and
  the command that opens a port;
- with `--server` (or `SIGNALLAB_SERVER`): whether the server answers and
  accepts the token.

```text
signallab [[version]]
Network: LAB-PC · 192.0.2.15
Data folder: C:\Users\lab\Documents\SignalLab — writable
Firewall · signallab (C:\…\Signal Lab\signallab.exe) · private network: not allowed yet — signallab firewall allow
Firewall · app (C:\…\Signal Lab\signal-lab.exe) · private network: allowed
✖ 1 thing in the way
```

`--json` prints the same as one object. See
[Troubleshooting](../reference/troubleshooting.md).

## `firewall` {#cli-firewall}

```text
signallab firewall allow [--public]
```

On Windows, lets other machines reach Signal Lab — what a monitor or a wait
needs to hear a device. Windows first asks for administrator rights; then the
inbound rules of `signallab` and of the desktop app (found next to it, or where
the installers put it) are replaced by one allow rule each, blocking rules
included, on private and domain networks.

| Option | What it does |
| --- | --- |
| `--public` | Also on public networks — a venue's Wi-Fi often is one. |

Exit codes: `0` done; `3` when the administrator prompt is declined or the
change fails. On Linux nothing is changed: it prints the ufw or firewalld
command that opens the ports you listen on, and exits `0`.

The firewall changes only when you run this; nothing else in `signallab` touches
it.

## `version` {#cli-version}

```text
signallab version
```

Prints `signallab [[version]]` — with `--json`, `{"version": "[[version]]"}`. `signallab --version`
prints the same version.
