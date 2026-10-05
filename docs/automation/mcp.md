---
title: Assistants (MCP)
description: signallab mcp lets an AI assistant build, check and run experiments, send messages, listen and play emulators, through the Model Context Protocol.
---

# Signal Lab for an assistant: `signallab mcp`

`signallab mcp` is a [Model Context Protocol](https://modelcontextprotocol.io)
server. An assistant in Claude Code, Claude Desktop, Cursor, VS Code or any
other MCP client starts it and can then:

- learn what an experiment is made of, write one, check it, run it and read,
  step by step, why it failed;
- send one OSC message, datagram, HTTP request, WebSocket message or MQTT
  publish, and listen on a port for what a device sends;
- fire a signal from your library;
- play a dependency — an HTTP API, an OSC, UDP or TCP device, an MQTT broker —
  and read what your system sent it;
- read earlier runs back and compare two of them.

Every action goes through the same engine commands the app uses, so an
experiment the assistant runs is the run the app would make, with the same
report, and every failure is worded as the interface words it.

::: warning
Sends, runs and emulators put real traffic on the network. Tell the assistant
which devices are yours to talk to; the bundled templates point at loopback
(`127.0.0.1`).
:::

## Setting it up {#setup}

`signallab` comes with the desktop app and is on your `PATH` after installing it
(see [Installing](cli.md#install)). The client starts `signallab mcp` itself and
talks to it over stdin and stdout; you do not run it by hand.

### Print the configuration {#print-config}

`--print-config` prints what a client needs, with the full path of this
`signallab`:

| Command | What it prints |
| --- | --- |
| `signallab mcp --print-config claude-code` | The `claude mcp add` command line. |
| `signallab mcp --print-config claude-desktop` | The `mcpServers` entry for Claude Desktop's configuration file. |
| `signallab mcp --print-config cursor` | The same `mcpServers` entry, for Cursor's `mcp.json`. |
| `signallab mcp --print-config vscode` | The `servers` entry for VS Code's `.vscode/mcp.json`. |

For an assistant that works on a [lab server](#on-a-server), add
`--server URL`: the printed configuration then carries it, with a placeholder
for the token.

### Claude Code {#claude-code}

Run the line `--print-config claude-code` prints, for example:

```bash
claude mcp add signallab -- "C:\Program Files\Signal Lab\signallab.exe" mcp
```

### Claude Desktop and Cursor {#claude-desktop}

Put the entry into the client's configuration — for Claude Desktop,
`claude_desktop_config.json`; for Cursor, `mcp.json` — and restart the client:

```json
{
  "mcpServers": {
    "signallab": {
      "command": "C:\\Program Files\\Signal Lab\\signallab.exe",
      "args": ["mcp"],
      "env": {}
    }
  }
}
```

### VS Code {#vscode}

```json
{
  "servers": {
    "signallab": {
      "type": "stdio",
      "command": "/usr/bin/signallab",
      "args": ["mcp"],
      "env": {}
    }
  }
}
```

### Other clients {#other-clients}

Any client that starts a stdio server works the same way: the command is
`signallab` (or its full path), the arguments `mcp` and any of the
[options](#options). On Linux, the server image can serve as the command too:

```bash
docker run -i --rm --network host --entrypoint signallab ghcr.io/proanima/signallab:[[version]] mcp
```

## Options {#options}

| Option | What it does | Default |
| --- | --- | --- |
| `--server URL` | Run experiments, sends and emulators on this Signal Lab server (see [On a lab server](#on-a-server)). | `SIGNALLAB_SERVER` |
| `--token-file PATH` | A file holding the server's token. | `SIGNALLAB_TOKEN_FILE`, else `SIGNALLAB_TOKEN` |
| `--data-dir PATH` | Where runs and their reports are kept. Not with `--server`. | the app's data folder (`Documents/SignalLab`) |
| `--library PATH` | The signal library for `list_signals` and `fire_signal`. | the app's `signals.json` |
| `--emulators PATH` | The emulator library for `list_emulators` and `start_emulator`. | the app's `emulators.json` |
| `--secrets files\|system` | Where secret values come from for runs on this machine, as for [`run`](cli.md#secrets). Not with `--server`. | `files` |
| `--secrets-dir PATH` | A folder of secret files, one per name. Not with `--server`. | `/run/secrets/signallab` when it exists |
| `--lang <code>` | The language of results and failures. | `SIGNALLAB_LANG`, else the locale, else `en` |
| `--print-config CLIENT` | Print a client's configuration and exit: `claude-code`, `claude-desktop`, `cursor` or `vscode`. | |

Runs keep their reports in the app's data folder, where the app keeps its own,
so they stay after the session.

## Tools {#tools}

Tools that only read are marked read-only, so a client can let them run without
asking. Tools that reach the outside world — they send, listen or start
something — are marked so, and a client can ask you before each call. None is
marked destructive.

| Tool | What it does | Reaches the outside world |
| --- | --- | --- |
| `describe_nodes` | The experiment document, every kind of node with its fields, outputs and an example, the `{{template}}` language, load profiles and the emulator document. | no |
| `list_templates` | The bundled experiments, with their parameters. | no |
| `get_template` | One bundled experiment as a document. | no |
| `validate_experiment` | Checks an experiment as the editor does before a run; sends nothing. | no |
| `run_experiment` | Runs an experiment to its end and reports every step. | yes |
| `send_osc` | One OSC message. | yes |
| `send_udp` | One UDP datagram. | yes |
| `send_http` | One HTTP request. | yes |
| `send_mqtt` | One MQTT 3.1.1 publish. | yes |
| `send_ws` | One WebSocket exchange. | yes |
| `listen` | What arrives on a UDP port for a while. | yes |
| `list_signals` | The signals of your library. | no |
| `fire_signal` | Sends a library signal. | yes |
| `list_emulators` | The emulators of your library. | no |
| `start_emulator` | Starts an emulator. | yes |
| `emulator_exchanges` | What a running emulator received and answered. | no |
| `set_emulator_down` | Takes a running emulator down, or brings it back. | yes |
| `list_runs` | The reports of earlier runs. | no |
| `compare_runs` | Two runs side by side. | no |
| `list_jobs` | What is running. | no |
| `stop_job` | Stops a running job. | yes |

### Experiments {#tools-experiments}

`describe_nodes` is what the assistant reads before writing an experiment; it is
the same as [`signallab nodes`](cli.md#cli-nodes). `list_templates` and
`get_template` give working examples to run or adapt.

`validate_experiment` and `run_experiment` take the experiment in one of three
ways — exactly one of the first three arguments — and the others adjust the run:

| Argument | What it is |
| --- | --- |
| `document` | An experiment document, as the app saves it. |
| `file` | The path of an experiment file on the machine `signallab` runs on. |
| `template` | The name of a bundled template. |
| `params` | Parameter values for this run: `{"name": "value"}`; numbers and booleans are taken as text. |
| `profile` | Run with this profile of the document; `""` for the defaults. |
| `seed` | `run_experiment`: the seed of the random values. |
| `timeout` | `run_experiment`: seconds the run may take, 1 to 300 (default 300). |

`run_experiment` answers when the run has ended: passed, failed or stopped, its
duration and seed, every step with what it did or why it failed, what each
emulator was asked, what each impairment relay did, and the report's path. A
run that fails is a normal answer — the steps say why — not a failed call.

### Single messages {#tools-send}

| Tool | Arguments |
| --- | --- |
| `send_osc` | `target` (`host:port`), `address`, `args`: numbers (whole → int32, or int64 beyond its range; else float32), strings, booleans, `null`, or `{"type": "int"\|"float"\|"str"\|"long"\|"double"\|"bool"\|"blob"\|"nil", "value": …}`. |
| `send_udp` | `target`, and `text` or `hex` (`"de ad be ef"`). |
| `send_http` | `method`, `url`, `headers` (`{"Name": "value"}`), `body`, `timeout_ms` (default 10000), `auth`: `{"scheme": "basic"\|"digest", "username", "password"}` or `{"scheme": "bearer", "token"}`. Returns the status, the time, the headers and the body — its first 16 KiB. |
| `send_mqtt` | `broker` (`host:port`, port 1883 when none), `topic`, `payload`, `qos` (0, 1 or 2), `retain`. An empty payload with `retain` clears a retained value. |
| `send_ws` | `url` (`ws://` or `wss://`), `text` or `hex`, `headers`, `protocols`, and to wait for the answer `expect` (contains), `expect_regex` or `wait` (any message); `timeout_ms` 1 to 120000 (default 2000). Returns the handshake, what was sent and the answer, its JSON parsed when it is JSON. |

These are the commands the app's screens use; see [`signallab send`](cli.md#cli-send).

### Listening {#tools-listen}

`listen` opens a UDP port **on the machine `signallab mcp` runs on**, for a
while, and returns what arrived: OSC messages decoded, other datagrams as text
and hex.

| Argument | What it is | Default |
| --- | --- | --- |
| `bind` | `IP:port`, e.g. `0.0.0.0:9000`. | required |
| `protocol` | `osc` or `udp`. | `osc` |
| `seconds` | How long to listen, 0.1 to 60. | 5 |
| `max` | Stop after this many datagrams, 1 to 1000. | 100 |

When nothing arrives on `0.0.0.0`, the answer reminds the assistant to check
the firewall ([`signallab doctor`](cli.md#cli-doctor)). With `--server`,
`listen` is refused: on a server, an experiment with a wait node listens there.

### Signals and emulators {#tools-library}

`list_signals` and `fire_signal` use your signal library — the app's
`signals.json`, `--library`, or a `library` path given to the call. A signal is
fired by its id or name, exactly as the app fires it.

`list_emulators` names the emulators of your library. `start_emulator` starts
one — a document in `emulator`, or a library entry's id or name in `name` — and
returns its job id and address; it answers by its rules until `stop_job`.
`bind` moves it to another `IP:port`, `params` gives values its templates read,
`seed` fixes its random choices. `emulator_exchanges` (`job_id`, and `after` for
only the newer ones) lists what arrived and what each rule answered.
`set_emulator_down` (`job_id`, `down`, and `fault`: `unavailable`, `reset` or
`timeout`) pulls the plug on a running emulator until it is brought up again:
HTTP meets the fault (`unavailable` answers 503), a TCP device and an MQTT broker
drop connections, OSC and UDP answer nothing. See [Emulators](../tools/emulators.md).

### Runs and jobs {#tools-runs}

`list_runs` reads the reports of earlier runs, newest first — of one experiment
when `experiment` names it, at most `limit` (1 to 500, default 50) — with each
load step's numbers. `compare_runs` takes two of their names, `a` (before) and
`b` (after), and puts each load step's latencies, error rate, achieved rate and
missed requests side by side, marking a change of 5 % or more the wrong way as a
regression. See [Runs and reports](../experiments/runs.md).

`list_jobs` lists what is running — monitors, generators, emulators, runs —
and `stop_job` stops one by its id.

## Results and errors {#results}

Every answer is text for the model and the same as structured data. A failure
is marked as an error and carries the engine's error — a stable `code`, its
values, the node and field it is about — worded in the chosen language with
`--lang`. Arguments the assistant got wrong come back in words it can correct.

## Progress and cancelling {#progress}

When the client asks for progress on `run_experiment`, every step is reported
as it happens (the node and its state), so the assistant — and you — see the run
move. Cancelling a call stops it; cancelling `run_experiment` stops the run
itself, as [[ui:common.stop]] in the app does.

When the client closes the connection, calls still under way finish, then
`signallab mcp` exits.

## On a lab server {#on-a-server}

With `--server http://192.0.2.10:1430` the experiments, sends, signals and
emulators happen **on that server**, through its API — with its network, its
secrets and its data folder — so the assistant reaches gear only the lab can
reach. Give the token in the client's environment:

```json
{
  "mcpServers": {
    "signallab": {
      "command": "signallab",
      "args": ["mcp", "--server", "http://192.0.2.10:1430"],
      "env": { "SIGNALLAB_TOKEN": "<the server's token>" }
    }
  }
}
```

What stays on this machine: the signal and emulator libraries (the app's, or
`--library` and `--emulators`) and the files a call names (`file`, `library`)
are read here, and what they hold is sent to the server; `listen` is refused.
See [Running Signal Lab as a server](../server/index.md).

## Safety {#safety}

- The assistant can do only what the tools do, and every tool is one of the
  app's own commands: it cannot reach anything the app could not.
- Tools that send, listen or start something are marked as reaching the
  outside world; your client decides whether to ask you before each call.
- Secret values never reach the assistant: an experiment names them as
  `{{secret.NAME}}`, and every result shows `••••` in their place.
- An emulator or a listener opens a port on the machine it runs on;
  `list_jobs` and `stop_job` show and end what is still running.

## Protocol {#protocol}

For client authors: JSON-RPC 2.0 over stdio, one message per line; stdout
carries protocol messages only, and anything for a person goes to stderr.
Protocol versions `2025-06-18`, `2025-03-26` and `2024-11-05` (the newest when
the client asks for another), batches, `ping`, `tools/list` and `tools/call`;
progress as `notifications/progress` for a call that sent a `progressToken`,
cancellation by `notifications/cancelled`. The server's `instructions` tell the
model how the tools fit together.
