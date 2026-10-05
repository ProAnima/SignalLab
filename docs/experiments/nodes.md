---
title: Nodes
description: Every kind of experiment node — what it does, its fields with defaults and limits, its outputs, the settings it takes, and how it looks in an experiment file.
---

# Node reference

Every kind of node an experiment can hold, in the groups of the add menu:
[actions](#actions), [waits](#waits), [emulation](#emulation),
[faults](#faults), [data](#data), [checks](#checks) and [flow](#flow). How to
add and wire them is in [The editor](index.md); `signallab nodes` prints the
same catalogue as JSON, for scripts and assistants ([The command
line](../automation/cli.md)).

## Reading this page {#reading}

Each node has a table of its fields:

- **Field** is the name in the properties pane; **In the file** is the key in
  the experiment's JSON.
- **Default** is what a node gets when you add it in the editor. Where a file
  may leave a key out, the value it then takes is given as *if absent*;
  other keys are required in a file.
- **Templates**: *yes* — the field takes `{{templates}}`: parameters,
  variables set earlier, secrets and generators, resolved as the step runs
  ([Data and templates](data.md)). *Parameters only* — it is opened before the
  first step, when only parameters are known. *No* — the value is taken as
  written.

Times are in milliseconds. Limits are checked before a run starts; a field out
of range keeps the experiment from running and is shown on the node.

## A node in a file {#file-shape}

In an experiment file, a node is an object with an `id` (unique in the
experiment), its `type`, its place on the canvas (`x`, `y`, zero or more), its
fields, and the settings it uses (`retry`, `repeat`, `load`, left out when
off). A wire is an edge from one node's output (`port`, `next` if absent) to
another node:

```json
{
  "nodes": [
    { "id": "start", "type": "start", "x": 40, "y": 80 },
    { "id": "ping", "type": "udp", "x": 270, "y": 80, "target": "127.0.0.1:9000", "text": "PING",
      "retry": { "attempts": 3, "delay_ms": 500, "backoff": "fixed" } },
    { "id": "end", "type": "end", "x": 500, "y": 80 }
  ],
  "edges": [
    { "from": "start", "to": "ping", "port": "next" },
    { "from": "ping", "to": "end", "port": "next" }
  ]
}
```

The examples below show one node each, as a file holds it.

## Settings shared by many nodes {#settings}

These are switched on in the lower part of a node's properties. Which node
takes which is listed under each node.

| Setting | Takes it | What it does |
| --- | --- | --- |
| [Retry](#retry) | Nodes that send or listen: [[ui:exp.node.http]], [[ui:exp.node.tcp]], [[ui:exp.node.osc]], [[ui:exp.node.udp]], [[ui:exp.node.mqtt]], [[ui:exp.node.ws_connect]], [[ui:exp.node.ws_send]], and every wait | Tries again when the step fails |
| [Repeat](#repeat) | Nodes that send: [[ui:exp.node.http]], [[ui:exp.node.tcp]], [[ui:exp.node.osc]], [[ui:exp.node.udp]], [[ui:exp.node.mqtt]], [[ui:exp.node.ws_send]] | Sends again and again, a number of times or for a time |
| [Load](#load) | [[ui:exp.node.http]] | Sends the request on a load profile, measured and judged by thresholds |
| [Waiting for a reply](#reply) | [[ui:exp.node.osc]], [[ui:exp.node.udp]] | Sends and waits for the answer in the same step |

### Retry {#retry}

[[ui:exp.retryOn]]: when the step fails — no connection, a timeout, a wait
with nothing matching — it pauses and runs again. Each failed attempt is a row
in the timeline; the step fails when the last attempt does. A template that
cannot be resolved is not retried. [[ui:common.stop]] also ends a pause.

| Field | In the file | What | Default and limits |
| --- | --- | --- | --- |
| [[ui:exp.attempts]] | `retry.attempts` | Attempts in all, the first included | 3; 2–10 in the editor (a file may also say 1) |
| [[ui:exp.retryDelay]] | `retry.delay_ms` | The pause before the second attempt | 500; 0–60 000 |
| [[ui:exp.backoff]] | `retry.backoff` | [[ui:exp.backoff.fixed]] (`fixed`): the same pause each time; [[ui:exp.backoff.exponential]] (`exponential`): twice as long after each failure | `fixed` (also if absent) |

No pause is longer than 60 seconds, however it doubles. A wait whose
[[ui:exp.portTimeout]] output has a wire does not fail on a timeout — it
leaves through that output — so it is not retried then.

### Repeat {#repeat}

[[ui:exp.repeatOn]]: the node sends again and again — a heartbeat, a poll, a
steady stream — without a loop in the graph. Each send reads its templates
afresh (`{{counter}}` is its number, `{{now}}` its time), and Retry, when on,
applies to each send. The step passes when every send did; a send that fails
for good fails the step. The timeline reports progress at most once a second.

| Field | In the file | What | Default and limits |
| --- | --- | --- | --- |
| [[ui:exp.repeatBy]] | `repeat.until` | [[ui:exp.repeatBy.count]] (`count`) or [[ui:exp.repeatBy.duration]] (`duration`) | `count` (also if absent) |
| [[ui:exp.repeatCount]] | `repeat.count` | Sends in all, the first included | 10 (also if absent); 2–10 000 |
| [[ui:exp.repeatDuration]] | `repeat.duration_ms` | How long to keep sending, from the first send | 10 000 (also if absent); 1–300 000 |
| [[ui:exp.repeatInterval]] | `repeat.interval_ms` | The pause between two sends | 1 000; 10–60 000; required in a file |
| [[ui:exp.repeatJitter]] | `repeat.jitter_ms` | Each pause up to this much longer, drawn from the run's seed | 0 (also if absent); 0–60 000 |

The repeats must fit in a run's 300 seconds, and *for a time* must need fewer
than 10 000 sends (its time divided by the interval).

### Load {#load}

[[ui:exp.loadOn]], on an [[ui:exp.node.http]] only: the request is sent on a profile —
a constant rate, a ramp, steps, a spike or random arrivals — with up to 512 in
flight at once (32 by default), and measured: latencies, errors, the rate
achieved. Thresholds decide whether the step passes. Load replaces Repeat and
Retry (a failed request is counted, not tried again), and leaves no response for
the checks after it. Its fields and results are in [Load testing](load.md).

### Waiting for a reply {#reply}

[[ui:exp.expectReply]], on an [[ui:exp.node.osc]] or a [[ui:exp.node.udp]]: the message is sent
from the port the reply is awaited on, so a device that answers the sender is
heard, and the step passes only when a matching reply arrives in time. No reply
fails the step — Retry sends again. The reply is stored in a variable, as a
wait's is.

| Field | In the file | What | Default and limits | Templates |
| --- | --- | --- | --- | --- |
| [[ui:exp.replyOn]] | `reply.bind` | `IP:port` to send from and listen on; port 0 takes any free port | `0.0.0.0:0` | No |
| [[ui:exp.replyAddress]] (OSC) | `reply.address` | The reply's address pattern, as in [[[ui:exp.node.wait_osc]]](#node-wait_osc) | `/*` | Yes |
| [[ui:exp.argRules]] (OSC) | `reply.args` | Argument rules, as in [[ui:exp.node.wait_osc]] | none; at most 16 | Values: yes |
| [[ui:exp.replyMode]] (UDP) | `reply.mode` | `any`, `contains`, `regex` or `hex` — see [Payload matching](#payload-matching) | `any` (also if absent) | No |
| [[ui:field.pattern]] (UDP) | `reply.pattern` | What the reply must contain or match | empty; required unless `any` | Yes |
| [[ui:exp.waitTimeout]] | `reply.timeout_ms` | How long to wait | 2 000 (also if absent); 1–120 000 | No |
| [[ui:exp.replyVariable]] | `reply.variable` | The variable the reply is stored in | `reply` (also if absent) | No |

The reply's port is opened before the first step, like a wait's.

## Actions {#actions}

Nodes that send. A wait after an action counts messages from the moment the
action started.

### HTTP request {#node-http}

Sends one HTTP request and keeps the response for the checks, branches and
[[ui:exp.node.extract]] nodes after it.

| Field | In the file | What | Default and limits | Templates |
| --- | --- | --- | --- | --- |
| [[ui:exp.method]] | `request.method` | GET, HEAD, POST, PUT, PATCH, DELETE or OPTIONS (a file may name any method) | `GET` | No |
| URL | `request.url` | An `http://` or `https://` URL | `http://127.0.0.1:8080/` | Yes |
| [[ui:common.timeoutMs]] | `request.timeout_ms` | For the whole exchange | 4 000 (10 000 if absent); 1–120 000 | No |
| [[ui:exp.headers]] | `request.headers` | `[[name, value], …]`; a row with an empty name is skipped | none | Yes, names and values |
| [[ui:exp.body]] | `request.body` | Text, or `null` for none | `null` | Yes |
| [[ui:field.auth]] | `request.auth` | [[ui:http.auth.none]], [[ui:http.auth.basic]], [[ui:http.auth.bearer]] or [[ui:http.auth.digest]], with [[ui:field.username]] and [[ui:field.password]], or [[ui:field.token]] | none | Yes |

- Any answer passes the step, 404 and 500 included: check the status with
  [[[ui:exp.node.assert_status]]](#node-assert_status) or branch on it with
  [[[ui:exp.node.branch_status]]](#node-branch_status). A request that gets no answer — refused, a
  timeout, a name that does not resolve, a certificate that is not trusted —
  fails the step.
- Redirects are followed, ten at most. `https://` certificates are verified.
- The response body is kept up to 256 KiB for the checks; a larger body is cut
  there (the checks say so when what they look for may be past the cut).
- Digest answers the server's 401 challenge and sends the request again. The
  credentials go only into the request: steps, reports and the Inspector never
  show the `Authorization` header. Write a password as `{{secret.NAME}}`.
- While the experiment keeps cookies (on by default, under
  [[ui:exp.params]]), what servers set is sent back with the run's later
  requests to them.

Outputs: [[ui:exp.outputPort]]. Settings: Retry, Repeat, Load.

```json
{ "id": "cue", "type": "http", "x": 270, "y": 80,
  "request": { "method": "POST", "url": "{{api}}/cue", "headers": [["Content-Type", "application/json"]],
               "body": "{\"cue\": 1}", "timeout_ms": 5000,
               "auth": { "scheme": "bearer", "token": "{{secret.API_TOKEN}}" } } }
```

See also [HTTP](../protocols/http.md).

### TCP message {#node-tcp}

Connects to a host over TCP, writes the payload, waits up to 250 ms for the
first bytes of an answer (it reads at most 1 024 bytes, once) and closes the
connection. The answer's size is reported, not checked.

In the [Inspector](../tools/inspector.md) the step is two `tcp` frames with the
source `experiment`: the payload written and, when one came, the answer read.
Secrets in use are masked in both, as in any frame.

| Field | In the file | What | Default and limits | Templates |
| --- | --- | --- | --- | --- |
| [[ui:exp.host]] | `host` | A host name or an IP address | `127.0.0.1` | Yes |
| [[ui:exp.port]] | `port` | | 9000; 1–65 535 | No |
| [[ui:common.timeoutMs]] | `timeout_ms` | For connecting, writing and the answer together | 4 000 (also if absent); 1–120 000 | No |
| [[ui:exp.payload]] | `payload` | The text written once connected, as UTF-8 | `hello` | Yes |

The step fails when the connection is refused, the name does not resolve or the
time runs out. Outputs: [[ui:exp.outputPort]]. Settings: Retry, Repeat.
[[ui:exp.sendNow]] connects and writes the payload once, and the node's result
says how many bytes were sent and came back.

```json
{ "id": "go", "type": "tcp", "x": 270, "y": 80, "host": "127.0.0.1", "port": 5000, "payload": "GO\r\n", "timeout_ms": 2000 }
```

### OSC message {#node-osc}

Sends one OSC 1.0 message over UDP.

| Field | In the file | What | Default and limits | Templates |
| --- | --- | --- | --- | --- |
| [[ui:common.target]] | `target` | `IP:port` or `host:port`; a host name is looked up when the step sends, its IPv4 address taken when it has one | `127.0.0.1:9000` | Yes |
| [[ui:common.address]] | `address` | Starts with `/` | `/test` | Yes |
| [[ui:exp.arguments]] | `args` | `[{ "type", "value" }, …]` — `int`, `float`, `str`, `long`, `double`, `bool`, `blob` (bytes), `nil` (no value) | none | Text (`str`) values: yes |
| [[ui:exp.expectReply]] | `reply` | Optional: send and wait for the answer — see [Waiting for a reply](#reply) | off | |

Outputs: [[ui:exp.outputPort]]; with a reply expected, it is followed only when
the reply came. Settings: Retry, Repeat, a reply. ⚡ [[ui:exp.routeThrough]] in
its properties puts an [[[ui:exp.node.impairment]]](#node-impairment) in front of it.

```json
{ "id": "fader", "type": "osc", "x": 270, "y": 80, "target": "{{device}}", "address": "/fader/1",
  "args": [{ "type": "float", "value": 0.75 }] }
```

See also [OSC](../protocols/osc.md).

### UDP datagram {#node-udp}

Sends a text payload as one UDP datagram to one or more targets.

| Field | In the file | What | Default and limits | Templates |
| --- | --- | --- | --- | --- |
| [[ui:common.target]] | `target` | `IP:port` or `host:port`; several separated by commas, semicolons or new lines each get the datagram. A host name is looked up when the step sends, its IPv4 address taken when it has one | `127.0.0.1:9000` | Yes |
| [[ui:exp.payload]] | `text` | The payload, as UTF-8 | `hello`; at most 65 507 bytes | Yes |
| [[ui:exp.expectReply]] | `reply` | Optional: send and wait for the answer — see [Waiting for a reply](#reply) | off | |

The step fails if any target cannot be reached. Outputs: [[ui:exp.outputPort]].
Settings: Retry, Repeat, a reply.

```json
{ "id": "ping", "type": "udp", "x": 270, "y": 80, "target": "{{device}}", "text": "PING {{run.id}}",
  "reply": { "bind": "0.0.0.0:0", "mode": "contains", "pattern": "PONG", "timeout_ms": 1000, "variable": "pong" } }
```

### MQTT publish {#node-mqtt}

Connects to an MQTT broker, publishes one message and disconnects. The
connection is MQTT 3.1.1 over plain TCP, with a clean session and no user name
or password. Connecting, publishing and the broker's acknowledgement must all
happen within 15 seconds.

| Field | In the file | What | Default and limits | Templates |
| --- | --- | --- | --- | --- |
| [[ui:exp.broker]] | `host` | The broker's host name or address | `127.0.0.1` | Yes |
| [[ui:exp.port]] | `port` | | 1883; 1–65 535 | No |
| [[ui:exp.topic]] | `topic` | No wildcards (`+`, `#`) | `lab/test` | Yes |
| [[ui:exp.payload]] | `payload` | The message, as text | `hello` | Yes |
| QoS | `qos` | 0, 1 or 2 | 0 | No |
| [[ui:exp.retain]] | `retain` | `true`: the broker keeps it as the topic's value | `false` | No |

All six keys are required in a file. The step fails when the broker cannot be
reached or refuses the connection or the message. Outputs:
[[ui:exp.outputPort]]. Settings: Retry, Repeat.

```json
{ "id": "light", "type": "mqtt", "x": 270, "y": 80, "host": "{{broker}}", "port": 1883,
  "topic": "lab/light/1/set", "payload": "on", "qos": 1, "retain": false }
```

See also [MQTT](../protocols/mqtt.md).

### WebSocket connect {#node-ws_connect}

Opens a WebSocket for the rest of the run, or until a
[[[ui:exp.node.ws_close]]](#node-ws_close). What arrives from then on is kept for the
[[[ui:exp.node.wait_ws]]](#node-wait_ws) steps on it. The URL and headers are resolved when
the step runs, so a token extracted earlier can be in them. Run again — in a
[[ui:exp.node.loop]] — it closes its previous connection first and opens a new one. When the run
ends, in any way, its connections are closed with a close frame.

| Field | In the file | What | Default and limits | Templates |
| --- | --- | --- | --- | --- |
| URL | `url` | A `ws://` or `wss://` URL | `ws://127.0.0.1:9001/` | Yes |
| [[ui:exp.headers]] | `headers` | `[[name, value], …]` sent with the upgrade request | none | Yes, names and values |
| [[ui:exp.wsProtocols]] | `protocols` | Subprotocols to offer, in order of preference; the server picks one | none | No |
| [[ui:common.timeoutMs]] | `timeout_ms` | For connecting and the upgrade | 5 000 (10 000 if absent); 1–120 000 | No |

`wss://` trusts the same certificates as `https://`. The step fails when the
connection or the upgrade fails; the server's status is in the reason.
Outputs: [[ui:exp.outputPort]]. Settings: Retry (not Repeat).

```json
{ "id": "socket", "type": "ws_connect", "x": 270, "y": 80, "url": "ws://127.0.0.1:9001/chat",
  "headers": [["Authorization", "Bearer {{token}}"]], "protocols": ["chat.v1"], "timeout_ms": 5000 }
```

See also [WebSocket](../protocols/websocket.md).

### WebSocket send {#node-ws_send}

Sends one message on the connection a [[ui:exp.node.ws_connect]] opened.

| Field | In the file | What | Default and limits | Templates |
| --- | --- | --- | --- | --- |
| [[ui:exp.wsConnection]] | `connection` | The id of a [[ui:exp.node.ws_connect]] node of this experiment | the first one | No |
| [[ui:exp.wsFormat]] | `binary` | [[ui:exp.wsText]] (`false`), or [[ui:exp.wsBinary]] (`true`): the payload is bytes written as hex, `de ad be ef` | `false` (also if absent) | No |
| [[ui:exp.payload]] | `text` | The message | `hello`; at most 16 MiB | Yes |

The connect must come before the send on its path; a send whose connection is
not open fails. Answers count from the moment the message is written.
Outputs: [[ui:exp.outputPort]]. Settings: Retry, Repeat.

```json
{ "id": "hello", "type": "ws_send", "x": 500, "y": 80, "connection": "socket",
  "text": "{\"type\":\"ping\",\"id\":\"{{uuid}}\"}", "binary": false }
```

### WebSocket close {#node-ws_close}

Closes a connection with a close handshake. The timeline says who closed it:
this step, the server earlier (with its code), or a connection that had broken.

| Field | In the file | What | Default and limits | Templates |
| --- | --- | --- | --- | --- |
| [[ui:exp.wsConnection]] | `connection` | The id of a [[ui:exp.node.ws_connect]] node | the first one | No |
| [[ui:field.code]] | `code` | 1000 (normal), or 3000–4999 for an application's own | 1000 (also if absent) | No |
| [[ui:field.reason]] | `reason` | Sent with the code | empty; at most 123 bytes, after templates | Yes |

Outputs: [[ui:exp.outputPort]]. No settings.

```json
{ "id": "bye", "type": "ws_close", "x": 960, "y": 80, "connection": "socket", "code": 1000, "reason": "done" }
```

### Log marker {#node-log}

Writes a line into the timeline and the report — a checkpoint, or the values a
run reached.

| Field | In the file | What | Default and limits | Templates |
| --- | --- | --- | --- | --- |
| [[ui:exp.logMessage]] | `message` | The text | `Check point`; at most 10 000 characters | Yes |

Outputs: [[ui:exp.outputPort]]. No settings.

```json
{ "id": "ready", "type": "log", "x": 500, "y": 80, "message": "device {{device}} ready" }
```

## Waits {#waits}

The [[ui:exp.group.observe]] group: nodes that wait for something to arrive.
They share these rules:

- **They listen from the start of the run.** A wait's port or broker
  subscription is opened before the first step, so a device that answers
  faster than the next step begins is not missed. Two waits on the same address
  share one socket.
- **They count from the latest action on their branch.** A message that
  arrived before the branch's last request is not an answer to it; before any
  action, everything since the run started counts.
- **The first matching message is taken.** A message one wait took is not seen
  by another.
- **[[ui:exp.portMatched]] or [[ui:exp.portTimeout]].** On a match the
  message is stored in the wait's variable and the flow follows
  [[ui:exp.portMatched]]. When the time runs out, it follows
  [[ui:exp.portTimeout]] if that output has a wire; otherwise the step fails,
  saying how many other messages arrived.
- Each socket keeps the latest 1 024 messages (and 64 MiB); older ones are
  dropped, and a timeout says how many were.
- [[ui:exp.listenNow]] listens with that one step, from now on.

Outputs: [[ui:exp.portMatched]] (required), [[ui:exp.portTimeout]] (optional).
Settings: Retry.

### Payload matching {#payload-matching}

[[ui:exp.node.wait_udp]], [[ui:exp.node.wait_mqtt]], [[ui:exp.node.wait_ws]]
and a UDP reply choose how the payload must look:

| Option | In the file | Matches when the payload |
| --- | --- | --- |
| [[ui:exp.mode.any]] | `any` | is anything |
| [[ui:exp.mode.contains]] | `contains` | read as UTF-8 text, contains the pattern (case-sensitive) |
| [[ui:exp.mode.regex]] | `regex` | read as UTF-8 text, matches the regular expression |
| [[ui:exp.mode.hex]] | `hex` | contains the bytes, written as hex pairs: `de ad be ef`, `deadbeef`, `0xde,0xad`, `DE:AD` |

The matched message is stored as an object. Later steps read its fields as
`{{reply.text}}` (with the variable's name in place of `reply`):

| Field | What |
| --- | --- |
| `text` | The payload as text |
| `hex`, `bytes` | The payload in hex (its first 1 024 bytes), and its size in bytes |
| `match` | What matched: the text, the regular expression's first group (or the whole match), or the bytes |
| `from` | The sender's `IP:port` |
| `ms` | Milliseconds from the branch's latest action (or the start of the run) to the message |
| `topic` | [[ui:exp.node.wait_mqtt]]: the topic it was published to |
| `json`, `kind` | [[ui:exp.node.wait_ws]]: the message parsed as JSON (`null` when it is not), and `text` or `binary` |

### Wait for OSC {#node-wait_osc}

Waits for an OSC message whose address matches a pattern and whose arguments
meet every rule. In a bundle, the first message that matches is the one taken.

| Field | In the file | What | Default and limits | Templates |
| --- | --- | --- | --- | --- |
| [[ui:exp.listenOn]] | `bind` | `IP:port` to listen on; `0.0.0.0` for every network card | `127.0.0.1:9001` | No |
| [[ui:exp.addressPattern]] | `address` | `*` any characters, `?` one, `[0-9]` a set (`[!0-9]` outside it), `{ping,pong}` either; wildcards stay within one `/` segment | `/pong`; at most 512 characters | Yes |
| [[ui:exp.argRules]] | `args` | `[{ "index", "op", "value" }, …]`: argument `index` compared with `value` by `op` ([Comparisons](#comparisons)); all must hold | none; at most 16, index 0–63 | Values: yes |
| [[ui:exp.waitTimeout]] | `timeout_ms` | | 2 000 (also if absent); 1–120 000 | No |
| [[ui:exp.replyVariable]] | `variable` | Where the message is stored | `reply` (also if absent) | No |

An argument compares as text: numbers as written, strings without quotes,
`true`/`false`, a blob in hex. A rule on an argument the message does not have
does not hold. The stored message has `address`, `args` (`{{reply.args[0]}}`),
`from` and `ms`.

```json
{ "id": "status", "type": "wait_osc", "x": 500, "y": 80, "bind": "0.0.0.0:9001", "address": "/status",
  "args": [{ "index": 0, "op": "eq", "value": "ready" }], "timeout_ms": 5000, "variable": "reply" }
```

### Wait for UDP {#node-wait_udp}

Waits for a UDP datagram whose payload matches.

| Field | In the file | What | Default and limits | Templates |
| --- | --- | --- | --- | --- |
| [[ui:exp.listenOn]] | `bind` | `IP:port` to listen on | `127.0.0.1:9001` | No |
| [[ui:exp.waitMode]] | `mode` | See [Payload matching](#payload-matching) | `contains` (`any` if absent) | No |
| [[ui:field.pattern]] | `pattern` | What the payload must contain or match | `pong`; required unless `any` | Yes |
| [[ui:exp.waitTimeout]] | `timeout_ms` | | 2 000 (also if absent); 1–120 000 | No |
| [[ui:exp.replyVariable]] | `variable` | | `reply` (also if absent) | No |

```json
{ "id": "ready", "type": "wait_udp", "x": 500, "y": 80, "bind": "0.0.0.0:9002", "mode": "contains",
  "pattern": "READY", "timeout_ms": 5000, "variable": "reply" }
```

### Wait for MQTT {#node-wait_mqtt}

Waits for a message published to a topic at a broker, whose payload matches.
The run connects and subscribes before its first step. Retained messages the
broker replays on subscribing are ignored: only what is published after the
run started counts.

| Field | In the file | What | Default and limits | Templates |
| --- | --- | --- | --- | --- |
| [[ui:exp.broker]] | `host` | The broker | `127.0.0.1` | Parameters only |
| [[ui:exp.port]] | `port` | | 1883; 1–65 535 | No |
| [[ui:exp.topicFilter]] | `topic` | A filter: `+` is any one level, `#` everything below (last only) | `lab/#` | Parameters only |
| [[ui:exp.waitMode]] | `mode` | See [Payload matching](#payload-matching) | `any` (also if absent) | No |
| [[ui:field.pattern]] | `pattern` | | empty; required unless `any` | Yes |
| [[ui:exp.waitTimeout]] | `timeout_ms` | | 2 000 (also if absent); 1–120 000 | No |
| [[ui:exp.replyVariable]] | `variable` | | `reply` (also if absent) | No |

```json
{ "id": "state", "type": "wait_mqtt", "x": 500, "y": 80, "host": "{{broker}}", "port": 1883,
  "topic": "lab/+/state", "mode": "contains", "pattern": "on", "timeout_ms": 5000, "variable": "reply" }
```

### Wait for HTTP request {#node-wait_http}

Waits for an HTTP request — a webhook, a callback — to the run's
[[[ui:exp.node.emulator]]](#node-emulator) on that address, or, when the run has no HTTP
emulator there, to a listener of the run's own that answers every request with
204. The request must match the method, the path and every condition.

| Field | In the file | What | Default and limits | Templates |
| --- | --- | --- | --- | --- |
| [[ui:exp.listenOn]] | `bind` | `IP:port` | `127.0.0.1:18080` — where a new [[ui:exp.node.emulator]] listens | No |
| [[ui:exp.method]] | `method` | A method, or [[ui:emu.methodAny]] (`ANY`); GET also takes HEAD | `ANY` (also if absent) | No |
| [[ui:exp.path]] | `path` | `/hooks/:name` names a segment (`{{request.params.name}}`); a final `/*` takes the rest | `/*` (also if absent); at most 512 characters | Yes |
| [[ui:emu.conditions]] | `when` | `[{ "on", "name", "op", "value" }, …]` on a `header`, a `query` parameter, the `body` or a `json` path; every one must hold | none; at most 16 | Yes, names and values |
| [[ui:exp.waitTimeout]] | `timeout_ms` | | 5 000 (2 000 if absent); 1–120 000 | No |
| [[ui:exp.replyVariable]] | `variable` | | `request` (also if absent) | No |

The stored request has `method`, `path`, `query`, `headers`, `body`, `json`,
`params`, `from` and `ms`: `{{request.json.event}}`, `{{request.headers.x-key}}`.

```json
{ "id": "hook", "type": "wait_http", "x": 500, "y": 80, "bind": "127.0.0.1:18081", "method": "POST",
  "path": "/hooks/:name", "when": [{ "on": "json", "name": "$.event", "op": "eq", "value": "deploy" }],
  "timeout_ms": 5000, "variable": "request" }
```

### Wait for WebSocket {#node-wait_ws}

Waits for a message on the connection a [[ui:exp.node.ws_connect]] opened, whose payload
matches. Messages since the latest action on the branch count — the connect
itself, a send, or any other request.

| Field | In the file | What | Default and limits | Templates |
| --- | --- | --- | --- | --- |
| [[ui:exp.wsConnection]] | `connection` | The id of a [[ui:exp.node.ws_connect]] node | the first one | No |
| [[ui:exp.waitMode]] | `mode` | See [Payload matching](#payload-matching) | `any` (also if absent) | No |
| [[ui:field.pattern]] | `pattern` | | empty; required unless `any` | Yes |
| [[ui:exp.waitTimeout]] | `timeout_ms` | | 2 000 (also if absent); 1–120 000 | No |
| [[ui:exp.replyVariable]] | `variable` | | `reply` (also if absent) | No |

A JSON message is readable field by field: `{{reply.json.type}}`. The connect
must come before the wait on its path.

```json
{ "id": "pong", "type": "wait_ws", "x": 730, "y": 80, "connection": "socket", "mode": "contains",
  "pattern": "pong", "timeout_ms": 3000, "variable": "reply" }
```

## Emulation {#emulation}

### Emulator {#node-emulator}

Plays a dependency — an HTTP API, an OSC, UDP or TCP device, an MQTT broker —
for the whole run. It opens before the first step and answers until the run
ends; in the flow the step passes at once. What it received is counted, rule by
rule, in the run's report.

| Field | In the file | What | Default |
| --- | --- | --- | --- |
| [[ui:emu.edit]] | `emulator` | The emulator: `name`, `bind` (`IP:port`), `protocol` (`http`, `osc`, `udp`, `tcp`, `mqtt`), its routes or rules, and an optional `outage` | An HTTP API named *API* on `127.0.0.1:18080` answering `/health` |

The properties show what it plays in one line. [[ui:emu.edit]] opens its rules,
the same editor as the [[[ui:nav.emulators]]](../tools/emulators.md) screen;
[[ui:emu.toLibrary]] keeps a copy in the emulator library, and
[[ui:emu.fromLibrary]] replaces this one with a copy from it. The rules — routes,
responses, faults, outages — are described there.

- An HTTP emulator is also what a [[[ui:exp.node.wait_http]]](#node-wait_http) on its
  address reads; an OSC or UDP emulator shares its port with the run's waits there.
- Two emulators of one transport cannot share a port in a run.
- [[[ui:exp.node.emulator_state]]](#node-emulator_state) takes it down and brings it back.

Outputs: [[ui:exp.outputPort]]. No settings.

```json
{ "id": "api", "type": "emulator", "x": 270, "y": 80,
  "emulator": { "name": "Orders API", "bind": "127.0.0.1:18080", "protocol": "http",
    "routes": [{ "method": "GET", "path": "/orders/:id", "order": "sequence",
                 "responses": [{ "status": 503 }, { "status": 200, "body": "{\"id\":\"{{request.params.id}}\"}" }] }] } }
```

## Faults {#faults}

Nodes that break things on cue. A branch of [[ui:exp.node.delay]] nodes and these beside the
traffic reads as a schedule; [Faults on a schedule](faults.md) shows how.

### Impairment {#node-impairment}

An impairment relay for the whole run: the system under test sends to (or
connects to) [[ui:exp.relayListen]] instead of the real target; the relay
forwards to [[ui:exp.relayTarget]], and the replies come back the same way,
impaired by the profile. It opens before the first step and closes when the run
ends, in any way, so nothing stays impaired; in the flow the step passes at once.
Every decision draws from the run's seed: the same seed and the same traffic
meet the same fate.

| Field | In the file | What | Default and limits | Templates |
| --- | --- | --- | --- | --- |
| [[ui:exp.relayListen]] | `listen` | `IP:port` the system under test sends to | `127.0.0.1:9010` | Parameters only |
| [[ui:exp.relayTarget]] | `target` | `IP:port` of the real destination, or `host:port` — a host name is looked up when the run starts, and a name that cannot be found stops the run at this node | `127.0.0.1:9000` | Parameters only |
| [[ui:ns.protocol]] | `protocol` | UDP (`udp`): each datagram meets its own fate; TCP (`tcp`): each connection is joined to one of its own to the target, and both streams are impaired | UDP (`udp` if absent) | No |
| [[ui:ns.preset]] and the values under it | `profile` | What the relay does to the traffic — see [The profile](#impair-profile) | [[ui:ns.preset.lan]] (no impairment if absent) | No |

A relay's listening address cannot be another socket of the run, and relays may
not forward to each other in a circle. A target given by name is followed once
it is looked up, so a circle through a name stops the run as it starts. The report counts each phase of a relay
apart.

Outputs: [[ui:exp.outputPort]]. No settings.

```json
{ "id": "relay", "type": "impairment", "x": 270, "y": 80, "listen": "127.0.0.1:9010", "target": "{{device}}",
  "profile": { "name": "lan", "latency_ms": 1, "jitter_ms": 1 } }
```

#### The profile {#impair-profile}

A preset chip — [[ui:ns.preset.lan]], [[ui:ns.preset.wifi]],
[[ui:ns.preset.4g]], [[ui:ns.preset.satellite]],
[[ui:ns.preset.intermittent]], [[ui:ns.preset.offline]] — fills in every value;
change any of them after. A relay reads only the values of its protocol; in a
file every key may be left out (zero, off).

| Field | In the file | What | Limits | Protocol |
| --- | --- | --- | --- | --- |
| — | `name` | A label for the timeline and the report: a preset's key (`lan`, `wifi`, `4g`, `satellite`, `intermittent`, `offline`) or your own | at most 60 characters | both |
| [[ui:ns.offline]] | `offline` | Nothing gets through | `true` / `false` | both |
| [[ui:ns.latency]] | `latency_ms` | Delay added to every packet, or chunk of a stream | 0–60 000 (the slider goes to 1 000) | both |
| [[ui:ns.jitter]] | `jitter_ms` | A random extra delay up to this much; a TCP stream stays in order | 0–60 000 (the slider goes to 500) | both |
| [[ui:ns.rate]] | `rate_kbps` | A bandwidth limit, 0 for none. UDP: past a second of queue, datagrams are dropped as throttled; TCP: the sender is slowed down, nothing is dropped | 0, or 8–10 000 000 | both |
| [[ui:ns.loss]] | `loss` | The chance a datagram is dropped | 0–1 (the slider shows %) | UDP |
| [[ui:ns.burst]], [[ui:ns.burstLength]] | `burst_start`, `burst_length` | The chance a burst of loss starts, and how many datagrams it lasts on average | 0–1; 1–1 000 when bursts are on | UDP |
| [[ui:ns.duplicate]] | `duplicate` | The chance a datagram is sent twice | 0–1 | UDP |
| [[ui:ns.corrupt]] | `corrupt` | The chance one bit of a datagram is flipped | 0–1 | UDP |
| [[ui:ns.reorder]] | `reorder` | The chance a datagram is held back, so later ones overtake it | 0–1 | UDP |
| [[ui:ns.reset]] | `reset` | The chance a chunk of a stream resets its connection instead — both sides get a reset | 0–1 | TCP |
| [[ui:ns.stall]] | `stall` | The chance a chunk leaves its connection half-open: nothing more goes through either way, and neither side is told | 0–1 | TCP |

More on relays, presets and what they model on the
[[[ui:nav.netsim]]](../tools/impairment.md) page.

### Change impairment {#node-impairment_change}

Switches one of the run's [[ui:exp.node.impairment]] nodes to another profile from this step on,
without dropping its port. The phase so far is closed and counted in the report.

| Field | In the file | What | Default | Templates |
| --- | --- | --- | --- | --- |
| [[ui:exp.relay]] | `relay` | The id of an [[ui:exp.node.impairment]] node of this experiment | the first one | No |
| [[ui:ns.preset]] and the values under it | `profile` | What it impairs with from now on — see [The profile](#impair-profile); the relay reads the values of its own protocol | [[ui:ns.preset.offline]] (no impairment if absent) | No |

The step fails if the relay is not running — it failed to relay, say.
Outputs: [[ui:exp.outputPort]]. No settings.

```json
{ "id": "cut", "type": "impairment_change", "x": 730, "y": 200, "relay": "relay",
  "profile": { "name": "offline", "offline": true } }
```

### Emulator down/up {#node-emulator_state}

Takes one of the run's emulators down, or brings it back up. While it is down an
HTTP emulator answers as [[ui:exp.downFault]] says; a TCP device and an MQTT
broker drop their connections and refuse new ones; OSC and UDP devices answer
nothing. Up again, the emulator follows its own outage schedule, if it has one.

| Field | In the file | What | Default |
| --- | --- | --- | --- |
| [[ui:exp.emulatorNode]] | `emulator` | The id of an [[ui:exp.node.emulator]] node of this experiment | the first one |
| [[ui:exp.emulatorDownState]] | `down` | [[ui:exp.emulatorGoesDown]] (`true`) or [[ui:exp.emulatorComesUp]] (`false`) | down (`false` if absent) |
| [[ui:exp.downFault]] | `fault` | HTTP only: [[ui:emu.outageFault.unavailable]] (`unavailable`), [[ui:emu.outageFault.reset]] (`reset`: the connection is closed without an answer) or [[ui:emu.outageFault.timeout]] (`timeout`: the request is held until the client gives up, 120 s at most) | `unavailable` (also if absent) |

Outputs: [[ui:exp.outputPort]]. No settings. Nothing is templated.

```json
{ "id": "down", "type": "emulator_state", "x": 500, "y": 200, "emulator": "api", "down": true, "fault": "unavailable" }
```

## Data {#data}

### Extract value {#node-extract}

Saves a part of the latest HTTP response on its path as a variable, for later
fields (`{{token}}`), checks and branches. An HTTP request must come before it on
every path. Clicking a value in a [[ui:exp.sendNow]] response adds one for you.

| Field | In the file | What | Default and limits | Templates |
| --- | --- | --- | --- | --- |
| [[ui:exp.variable]] | `variable` | The name: letters, digits and `_`, not starting with a digit, not a reserved word, not a parameter's name | `token` | No |
| [[ui:exp.extractFrom]] | `from` | [[ui:exp.from.json]] (`json`), [[ui:exp.from.header]] (`header`), [[ui:exp.from.status]] (`status`), [[ui:exp.from.body]] (`body`) or [[ui:exp.from.regex]] (`regex`) | `json` | No |
| [[ui:exp.jsonPath]], [[ui:exp.headerName]] or [[ui:exp.pattern]] | `expr` | A JSON path (`$.data.token`, `$.items[0]`, `$["first name"]`), a header name (any case), or a regular expression — its first group, or the whole match | `$.token`; not used for status and body | No |

The step fails when there is nothing to take: the body is not JSON, the path or
header is missing, the expression does not match, or — for a JSON field or the
whole body — the body was longer than the 256 KiB kept. A status is stored as a
number; the rest as text, or as the JSON value found. Outputs:
[[ui:exp.outputPort]]. No settings.

```json
{ "id": "token", "type": "extract", "x": 500, "y": 80, "variable": "token", "from": "json", "expr": "$.data.token" }
```

More on variables in [Data and templates](data.md).

## Checks {#checks}

A check passes, or fails the run. The four response checks read the latest HTTP
response on their path, so an HTTP request — not one under load — must come
before them on every path.

### HTTP status {#node-assert_status}

Passes when the latest response's status is exactly the one given.

| Field | In the file | What | Default and limits | Templates |
| --- | --- | --- | --- | --- |
| [[ui:exp.expectedStatus]] | `status` | | 200; 100–599 | No |

Outputs: [[ui:exp.outputPort]]. No settings.

```json
{ "id": "ok", "type": "assert_status", "x": 500, "y": 80, "status": 200 }
```

### Response text {#node-assert_body}

Passes when the latest response's body contains the text, exactly (case
included). Only the first 256 KiB of a body are kept: text not found in a body
that was cut fails with that reason.

| Field | In the file | What | Default and limits | Templates |
| --- | --- | --- | --- | --- |
| [[ui:exp.contains]] | `contains` | | `ok`; required | Yes |

Outputs: [[ui:exp.outputPort]]. No settings.

```json
{ "id": "ready", "type": "assert_body", "x": 500, "y": 80, "contains": "ready" }
```

### Response header {#node-assert_header}

Passes when the latest response has the header and its value contains the text.
The header's name is matched in any case; the value exactly.

| Field | In the file | What | Default and limits | Templates |
| --- | --- | --- | --- | --- |
| [[ui:exp.headerName]] | `name` | | `content-type`; required | Yes |
| [[ui:exp.contains]] | `contains` | What its value must contain; empty: the header only has to be there | `application/json` | Yes |

Outputs: [[ui:exp.outputPort]]. No settings.

```json
{ "id": "json", "type": "assert_header", "x": 500, "y": 80, "name": "Content-Type", "contains": "json" }
```

### Response time {#node-assert_latency}

Passes when the latest response took at most this long, from sending the request
to the end of its body.

| Field | In the file | What | Default and limits | Templates |
| --- | --- | --- | --- | --- |
| [[ui:exp.maxLatency]] | `max_ms` | | 1 000; 1–120 000 | No |

Outputs: [[ui:exp.outputPort]]. No settings.

```json
{ "id": "fast", "type": "assert_latency", "x": 500, "y": 80, "max_ms": 250 }
```

### Check value {#node-assert_value}

Compares a value — usually a variable, written as a template — with an expected
one, and passes when the comparison holds.

| Field | In the file | What | Default | Templates |
| --- | --- | --- | --- | --- |
| [[ui:exp.value]] | `value` | What is compared: `{{token}}`, `{{reply.args[0]}}` | `{{token}}` | Yes |
| [[ui:exp.operator]] | `op` | See [Comparisons](#comparisons) | [[ui:exp.op.not_empty]] | No |
| [[ui:exp.expected]] | `expected` | Not used by [[ui:exp.op.empty]] and [[ui:exp.op.not_empty]] | empty (also if absent) | Yes |

Outputs: [[ui:exp.outputPort]]. No settings.

```json
{ "id": "state", "type": "assert_value", "x": 730, "y": 80, "value": "{{state}}", "op": "eq", "expected": "ready" }
```

### Comparisons {#comparisons}

[[ui:exp.node.assert_value]], [[ui:exp.node.branch_value]], the exit condition
of a [[ui:exp.node.loop]], OSC argument rules and HTTP conditions compare the
same way:

| Option | In the file | Holds when the value |
| --- | --- | --- |
| [[ui:exp.op.eq]] | `eq` | equals the expected one — as numbers when both are numbers (`200` = `200.0`), else as exact text |
| [[ui:exp.op.ne]] | `ne` | does not equal it, by the same rule |
| [[ui:exp.op.lt]], [[ui:exp.op.le]], [[ui:exp.op.gt]], [[ui:exp.op.ge]] | `lt`, `le`, `gt`, `ge` | is less, at most, greater, at least — both must be numbers: otherwise a check, a branch or a [[ui:exp.node.loop]] fails the step, and an argument rule or an HTTP condition does not hold |
| [[ui:exp.op.contains]] | `contains` | contains the expected text |
| [[ui:exp.op.matches]] | `matches` | matches the expected regular expression |
| [[ui:exp.op.empty]], [[ui:exp.op.not_empty]] | `empty`, `not_empty` | is empty (spaces count as empty) / is not |

## Flow {#flow}

Nodes that decide where the run goes. More on branches, joins and loops in
[Flow](flow.md).

### Start {#node-start}

Where the run begins; every experiment has exactly one. It has no input and no
fields. The timeline's first row gives the run's seed.

Outputs: [[ui:exp.outputPort]], required. Several wires from it start parallel
branches at once.

```json
{ "id": "start", "type": "start", "x": 40, "y": 80 }
```

### End {#node-end}

Where the run completes; every experiment has exactly one, and it has no outputs.
Several branches may lead to it: the run passes once, after the last branch
finished, and only if none failed. A run that never reaches [[ui:exp.node.end]] fails.

```json
{ "id": "end", "type": "end", "x": 960, "y": 80 }
```

### Delay {#node-delay}

Waits a fixed time before the next step.

| Field | In the file | What | Default and limits | Templates |
| --- | --- | --- | --- | --- |
| [[ui:exp.delayMs]] | `ms` | | 300; 0–60 000 | No |

Outputs: [[ui:exp.outputPort]]. No settings. For longer waits, put several in a row
or in a [[ui:exp.node.loop]].

```json
{ "id": "pause", "type": "delay", "x": 500, "y": 80, "ms": 500 }
```

### Status branch {#node-branch_status}

Chooses [[ui:exp.yes]] when the latest HTTP response has this status, else
[[ui:exp.no]]. An HTTP request must come before it on every path.

| Field | In the file | What | Default and limits | Templates |
| --- | --- | --- | --- | --- |
| [[ui:exp.expectedStatus]] | `status` | | 200; 100–599 | No |

Outputs: [[ui:exp.yes]] and [[ui:exp.no]], both required. No settings.

```json
{ "id": "branch", "type": "branch_status", "x": 500, "y": 80, "status": 200 }
```

### Branch on value {#node-branch_value}

Chooses [[ui:exp.yes]] when a comparison holds, else [[ui:exp.no]]. Its fields
and [comparisons](#comparisons) are those of
[[[ui:exp.node.assert_value]]](#node-assert_value); a comparison that cannot be
made (`lt` on text) fails the step.

| Field | In the file | What | Default | Templates |
| --- | --- | --- | --- | --- |
| [[ui:exp.value]] | `value` | What is compared | `{{token}}` | Yes |
| [[ui:exp.operator]] | `op` | | [[ui:exp.op.eq]] | No |
| [[ui:exp.expected]] | `expected` | | empty (also if absent) | Yes |

Outputs: [[ui:exp.yes]] and [[ui:exp.no]], both required. No settings.

```json
{ "id": "ok", "type": "branch_value", "x": 730, "y": 80, "value": "{{reply.args[0]}}", "op": "eq", "expected": "ok" }
```

### Parallel branch {#node-fork}

Runs what follows [[ui:exp.branch1]] and [[ui:exp.branch2]] at the same time,
each branch with its own copy of the variables. Each output may have more wires
for more branches. No fields.

Outputs: [[ui:exp.branch1]] and [[ui:exp.branch2]], both required.

```json
{ "id": "split", "type": "fork", "x": 270, "y": 80 }
```

### Join branches {#node-join}

Waits until every wire into it has been reached, then continues once, with the
branches' variables merged — where two branches set the same variable, the one
whose wire comes later in the file wins — and the latest HTTP response of the
last of them that had one. No fields.

Only branches that all run meet here: a [[ui:exp.node.join]] behind a
[[ui:exp.node.branch_status]], whose [[ui:exp.yes]] and [[ui:exp.no]] never both
happen, never continues; when no other path reaches [[ui:exp.node.end]], the run
fails at this node, saying how many branches it still waited for.

Outputs: [[ui:exp.outputPort]], required.

Any other node with several wires into it runs once for each arrival.

```json
{ "id": "joined", "type": "join", "x": 730, "y": 80 }
```

### Loop {#node-loop}

Runs the steps on [[ui:exp.portBody]] — which lead back to it — again and
again: at most a number of times, and, when it has an exit condition, until that
holds.

| Field | In the file | What | Default and limits | Templates |
| --- | --- | --- | --- | --- |
| [[ui:exp.loopMax]] | `max` | Iterations at most | 5; 1–1 000 | No |
| [[ui:exp.loopUntilOn]] | `until` | Optional exit condition `{ "value", "op", "expected" }`, as [[[ui:exp.node.assert_value]]](#node-assert_value) | off | Value and expected: yes |

- The body always runs at least once. The exit condition is read after each
  iteration, so the body can set what it tests.
- [[ui:exp.portDone]] follows when the condition holds — or, without a
  condition, after the last iteration.
- [[ui:exp.portLimit]] follows when the iterations ran out before the condition
  held. Without a wire on it, that fails the step.
- Inside the body, `{{counter}}` is the iteration's number.
- A body runs as one branch: each output inside it has one wire; it holds no
  [[ui:exp.node.start]], [[ui:exp.node.end]], [[ui:exp.node.fork]],
  [[ui:exp.node.join]] or other [[ui:exp.node.loop]]; it is entered only
  through [[ui:exp.portBody]]; and every wire in it leads on in the body or
  back to the loop.

Outputs: [[ui:exp.portBody]] and [[ui:exp.portDone]] (required),
[[ui:exp.portLimit]] (optional). No settings.

```json
{ "id": "poll", "type": "loop", "x": 270, "y": 80, "max": 10,
  "until": { "value": "{{status.args[0]}}", "op": "eq", "expected": "ready" } }
```
