---
title: Events
description: The WebSocket of engine events on a Signal Lab server, and every channel with its payload and when it is sent.
---

# Events

Everything that happens while a job runs — a run's steps, a monitor's
messages, a burst's numbers, the Inspector's frames, a job's end — is sent as
an event. In a browser, the server's page gets them on one WebSocket,
`/api/events`; a script can listen on the same socket. The desktop app gets the
same events, with the same names and payloads, inside the app.

## Subscribing {#subscribe}

Open a WebSocket to `/api/events` on the server:

```bash
websocat -H "Authorization: Bearer $TOKEN" ws://127.0.0.1:1430/api/events
```

- **Authentication** is the rest of the API's: the token as
  `Authorization: Bearer`, or a browser's session cookie. Without it the
  upgrade is refused with `401` `auth.required`.
- **Origin**: a client that sends an `Origin` header must send the server's
  own (host and port equal to `Host`), or the upgrade is refused with `403`
  `auth.origin`. Most WebSocket libraries outside a browser send none.
- **Every event to every client.** There is nothing to subscribe to: each
  socket gets every event of every job, whoever started it. Pick what you need
  by `event`, and by `job_id` in the payload.
- **Listen only.** The server ignores what a client sends, except a close; a
  message larger than 64 KiB closes the socket.
- **Keep-alive.** The server pings every 20 s, so a quiet socket stays open
  through proxies. When the server stops, it closes every socket.
- **Nothing is replayed.** Events sent while a client was not connected are
  lost to it. A client that reconnects should read the current state with
  commands (`jobs_list`, `inspect_snapshot`, `emulator_exchanges`…).
- **Falling behind.** Up to 4096 events wait for one socket. A client that
  falls further behind gets [`server://lagged`](#event-server-lagged) with how
  many it missed. A client that stops reading altogether — nothing it takes for
  10 s — is disconnected; reconnect and read the state again.

## Message format {#format}

Each event is one text message holding one JSON object:

```json
{ "event": "scan://open", "payload": { "job_id": 9, "ts": 1759600000123, "port": 8080, "banner": null } }
```

| Field | What it is |
| --- | --- |
| `event` | The channel, below |
| `payload` | The event's values; its shape depends on the channel |

Times (`ts`, a peer's `first_ms` and `last_ms`) are milliseconds since 1970;
latencies and other durations (`*_latency_ms`, `p50_ms`…, `ms`) are
milliseconds. Errors in payloads are
[`EngineError`](index.md#errors) objects; their codes are listed in
[error messages](../reference/errors.md).

## Channels {#channels}

| Channel | Sent by | When |
| --- | --- | --- |
| [`experiment://step`](#event-experiment-step) | A run | A step starts, passes, fails, retries, repeats or reports load |
| [`experiment://ended`](#event-experiment-ended) | A run | Once, when the run ends on its own |
| [`job://ended`](#event-job-ended) | Every job | Once, when the job ends on its own or fails |
| [`osc://message`](#event-osc-message) | OSC monitor | Every packet, sampled to 200/s under a flood |
| [`osc://monitor-stat`](#event-osc-monitor-stat) | OSC monitor | Every 250 ms, and at the end |
| [`osc://gen-tick`](#event-osc-gen-tick) | OSC generator | Every message, or 30 to 45 times a second above 60 messages a second |
| [`http://burst-progress`](#event-http-burst-progress) | HTTP burst | Every 100 ms, and at the end |
| [`ws://state`](#event-ws-state) | WebSocket connection | Connected, closed |
| [`ws://messages`](#event-ws-messages) | WebSocket connection | Every 100 ms with something new |
| [`mqtt://state`](#event-mqtt-state) | MQTT connection | Connected, subscribed, closed |
| [`mqtt://messages`](#event-mqtt-messages) | MQTT connection | Every 100 ms with something new |
| [`mqtt://ack`](#event-mqtt-ack) | MQTT connection | A QoS 1/2 publish completed; an unsubscribe answered |
| [`broadcast://emit-stat`](#event-broadcast-emit-stat) | Beacon | Every 250 ms, and at the end |
| [`broadcast://peers`](#event-broadcast-peers) | Discovery listener | Every 400 ms |
| [`netsim://stat`](#event-netsim-stat) | Impairment relay | Every 250 ms |
| [`storm://stat`](#event-storm-stat) | Storm | Every 250 ms, and at the end |
| [`scan://open`](#event-scan-open) | Scanner | Every open port |
| [`scan://progress`](#event-scan-progress) | Scanner | About every 1 % of the range, and at the end |
| [`emulator://activity`](#event-emulator-activity) | Emulator job | Every 200 ms with something new |
| [`inspect://batch`](#event-inspect-batch) | Inspector | Every 120 ms with new frames, about once a second when quiet, while capture is armed |
| [`server://lagged`](#event-server-lagged) | The server | A client fell behind |

### `experiment://step` {#event-experiment-step}

One step of a run: a node starting, passing, failing, waiting to try again,
repeating, or reporting a load's progress. A run started with `/api/run` sends
the same steps on its response (see [runs](run.md)).

| Field | Type | Meaning |
| --- | --- | --- |
| `job_id` | number | The run's job |
| `ts` | number | When |
| `node_id` | string | The node |
| `state` | string | `running`, `passed`, `failed`, `retry` (an attempt failed and the step runs again after a pause), `repeating` (a repeating action's progress, at most once a second) or `load` (a load's progress, at most once a second) |
| `detail` | string | What happened, in English; empty for `running` and `failed` (see `error`) |
| `message_key` | string or null | The interface's text for it, as a key of its dictionary |
| `message_params` | object or null | The values `message_key` names |
| `vars` | object | Variables the step wrote; left out when none |
| `error` | `EngineError` | Why it failed, or why the attempt did (`retry`); left out otherwise |
| `frame` | number | The Inspector frame of the message a wait (or a send's expected reply) matched, when capture was armed; left out otherwise |
| `load` | object | What a load measured, its thresholds read — on a load step's last event, passed or failed; left out otherwise. See [load](../experiments/load.md) |
| `omitted` | number | Steps just before this one that were left out; left out when none. A run reports its first 5 000 steps one by one, then one a second (and every failure), and as it ends its last 500 — so a Loop of a thousand iterations stays readable |

A run's [[ui:exp.node.end]] node shows `running` when the first branch reaches it and
`passed` once every branch has finished without a failure. Secret values are
masked in every field.

### `experiment://ended` {#event-experiment-ended}

A run ended on its own: it passed, failed or ran out of time. Sent right after
the same payload on [`job://ended`](#event-job-ended). A run stopped with
`job_stop` or [[ui:app.stopAll]] sends neither and saves no report.

| Field | Type | Meaning |
| --- | --- | --- |
| `job_id` | number | The run's job |
| `kind` | string | `experiment` |
| `seed` | number | The seed it ran with |
| `profile` | string or null | Its profile |
| `overridden` | boolean | Some parameter values came from [[ui:exp.runWith]] or `overrides` |
| `error` | `EngineError` or null | The run's first failure; null when it passed |
| `report_path` | string or null | Its report, in `runs/` of the data folder |
| `report_error` | `EngineError` or null | Why the report could not be written |

### `job://ended` {#event-job-ended}

A job ended on its own or failed. A job stopped with `job_stop` or
`jobs_stop_all` does not send it.

| Field | Type | Meaning |
| --- | --- | --- |
| `job_id` | number | The job |
| `kind` | string | `osc-monitor`, `osc-gen`, `http-burst`, `netsim`, `storm`, `scan`, `beacon`, `discovery`, `mqtt`, `websocket`, `emulator` or `experiment` |
| `error` | `EngineError` or null | Why it ended, when something went wrong |

A run's `job://ended` carries the fields of
[`experiment://ended`](#event-experiment-ended) too. What ends each kind:

| `kind` | Ends when | `error` |
| --- | --- | --- |
| `osc-monitor` | The socket can no longer receive | `wait.receive_failed` |
| `osc-gen` | Its duration is over, or a send fails | null, or `transport.*` |
| `http-burst` | Its total or duration is reached | null |
| `storm` | Its duration is over | null |
| `scan` | Every port of the range was tried | null |
| `beacon` | Its rounds or duration are over, or more than 32 sends failed with none sent | null, or `transport.*` |
| `discovery` | The socket can no longer receive | `wait.receive_failed` |
| `mqtt` | The broker closed the connection or it was lost | `transport.*` (`transport.reset` when the broker closed it), or `mqtt.protocol` |
| `websocket` | The connection closed | null, or why it was lost |
| `netsim` | The relay can no longer work | why |
| `emulator` | Its socket fails | why |
| `experiment` | The run ends | the run's failure, or null |

### `osc://message` {#event-osc-message}

One UDP packet an OSC monitor received, decoded. Decoded packets are sampled to
at most 200 events a second so a flood cannot swamp the link to the interface
([`osc://monitor-stat`](#event-osc-monitor-stat) says how many were received and
how many were not forwarded); a malformed packet is always sent.

| Field | Type | Meaning |
| --- | --- | --- |
| `job_id` | number | The monitor's job |
| `ts` | number | When it arrived |
| `from` | string | The sender, `IP:port` |
| `bytes` | number | The packet's size |
| `messages` | object[] | Each message of the packet (a bundle has several): `address` and `args` ([`OscArg`](commands.md#type-oscarg)`[]`) |
| `error` | `EngineError` or null | `osc.packet_malformed` when the packet did not decode (then `messages` is empty) |

### `osc://monitor-stat` {#event-osc-monitor-stat}

An OSC monitor's running totals, every 250 ms and once more when it ends, so a
sampled list shows how much traffic it stands for.

| Field | Type | Meaning |
| --- | --- | --- |
| `job_id` | number | The monitor's job |
| `ts` | number | When |
| `packets` | number | Packets received, decoded or not |
| `bytes` | number | Bytes received |
| `not_shown` | number | Packets received but not forwarded as [`osc://message`](#event-osc-message) (the rate sample held them) |
| `malformed` | number | Packets that did not decode as OSC |

### `osc://gen-tick` {#event-osc-gen-tick}

An OSC generator's progress: for every message below 60 messages a second;
above that for every n-th, n being the rate divided by 30 and rounded down —
30 to 45 times a second.

| Field | Type | Meaning |
| --- | --- | --- |
| `job_id` | number | The generator's job |
| `ts` | number | When |
| `value` | number | The value just sent, before it is rounded to an integer or to a 32-bit float |
| `sent` | number | Messages sent so far |

### `http://burst-progress` {#event-http-burst-progress}

An HTTP burst's numbers, every 100 ms while it runs, and once more with
`done: true` when it ends on its own.

| Field | Type | Meaning |
| --- | --- | --- |
| `job_id` | number | The burst's job |
| `ts` | number | When |
| `sent` | number | Requests answered or failed so far |
| `ok` | number | Of those, answered with a 2xx status and the whole body |
| `failed` | number | Of those, any other status, a body cut short, or no answer — never one this machine could not send |
| `local` | number | Of those, requests this machine could not send: it ran out of sockets or local ports (`transport.exhausted`). Not in `failed`, nor in the latencies |
| `missed` | number | A paced burst's requests that waited too long for a free worker and were skipped |
| `rps` | number | Requests per second over the last 100 ms; in the last event, over the whole burst |
| `last_latency_ms`, `min_latency_ms`, `max_latency_ms`, `avg_latency_ms` | number | Latencies so far |
| `p50_ms`, `p90_ms`, `p95_ms`, `p99_ms` | number | Percentiles of every request so far, failures included, within 0.5 % |
| `done` | boolean | The last event of the burst |

### `ws://state` {#event-ws-state}

A WebSocket connection opened by `ws_connect` connected, or closed. A
connection whose job was stopped sends no `closed`.

| Field | Type | Meaning |
| --- | --- | --- |
| `job_id` | number | The connection's job |
| `ts` | number | When |
| `state` | string | `connected` or `closed` |
| `handshake` | object | `url`, `peer`, `local`, `protocol` (the subprotocol the server chose, or null) and `ms` (connecting and the upgrade) |
| `closed` | object or null | With `closed`: `code`, `reason`, `by` (`client`, `server` or `lost`) and `error` |

### `ws://messages` {#event-ws-messages}

What a WebSocket connection sent and received since the last event, every
100 ms when there is something.

| Field | Type | Meaning |
| --- | --- | --- |
| `job_id` | number | The connection's job |
| `ts` | number | When |
| `messages` | object[] | In order: `ts`, `dir` (`rx` received, `tx` sent), `kind` (`text` or `binary`), `text` (the first 64 KiB as UTF-8; of a binary message the first 4096 bytes; bytes that are not UTF-8 become `�`), `hex` (a binary message's first 4096 bytes as hex, else null), `bytes` (the full size) and `truncated` (more than was shown: past 64 KiB of text, past 4096 bytes of binary) |
| `dropped` | number | Messages left out of this event because there were more than 2000, or more than 1 MiB of `text` and `hex`; the oldest go first |
| `received`, `sent` | number | Messages received and sent on the connection so far, shown or not |

### `mqtt://state` {#event-mqtt-state}

An MQTT connection's state changed.

| Field | Type | Meaning |
| --- | --- | --- |
| `job_id` | number | The connection's job |
| `ts` | number | When |
| `state` | string | `connected`; `subscribed` after each answer to a subscribe; `closed` when the connection ended (not when its job was stopped) |
| `broker` | string | `host:port` |
| `error` | `EngineError` or null | Why a `closed` connection ended (`transport.reset` when the broker closed it); null otherwise |
| `grants` | object[] | With `subscribed`: each filter asked for, with `filter`, `qos` (granted) and `accepted`; empty otherwise |

### `mqtt://messages` {#event-mqtt-messages}

What an MQTT connection received since the last event, every 100 ms when there
is something. A redelivered QoS 2 message is shown once.

| Field | Type | Meaning |
| --- | --- | --- |
| `job_id` | number | The connection's job |
| `ts` | number | When |
| `messages` | object[] | One per topic, the latest that arrived on it: `ts`, `topic`, `payload` (its first 4096 bytes as UTF-8; bytes that are not become `�`), `bytes` (the whole payload's size), `truncated` (`payload` is only the start), `qos`, `retain`, `dup`, and `count` (messages on the topic since the last event, this one included) |
| `dropped` | number | Always 0: when an event would carry more than 4000 topics or 2 MiB of payload, the connection stops reading until it has gone out, and the broker waits |

### `mqtt://ack` {#event-mqtt-ack}

The broker completed something the connection asked for.

| Field | Type | Meaning |
| --- | --- | --- |
| `job_id` | number | The connection's job |
| `ts` | number | When |
| `kind` | string | `published` (a QoS 1 or 2 publish is complete) or `unsubscribed` |
| `packet_id` | number | The MQTT packet id |
| `topic` | string or null | The published topic; null for `unsubscribed` |

### `broadcast://emit-stat` {#event-broadcast-emit-stat}

A beacon's counters, every 250 ms, and once more when it ends on its own with
`pps` 0.

| Field | Type | Meaning |
| --- | --- | --- |
| `job_id` | number | The beacon's job |
| `ts` | number | When |
| `targets` | number | Destinations in each round |
| `rounds` | number | Rounds sent |
| `packets`, `bytes` | number | Datagrams and bytes sent |
| `errors` | number | Sends that failed |
| `local` | number | Sends this machine could not make (out of buffers or sockets), not among `errors`; left out while 0 |
| `pps` | number | Datagrams per second over the last 250 ms |

### `broadcast://peers` {#event-broadcast-peers}

What a discovery listener has heard, every 400 ms.

| Field | Type | Meaning |
| --- | --- | --- |
| `job_id` | number | The listener's job |
| `ts` | number | When |
| `peers` | object[] | Most recently heard first: `addr`, `proto`, `packets`, `bytes`, `first_ms`, `last_ms`, `last_summary`, `responded` (its packets that were answered, counted as they arrived); at most 512 |
| `packets`, `bytes` | number | Everything received |
| `responses` | number | Answers sent |
| `unlisted` | number | Distinct peers past the 512 the table holds: counted here, not in `peers` |
| `replies_skipped` | number | Probes not answered because too many auto-replies (1024) were already in flight — the cap that keeps a probe flood from being amplified without bound |

### `netsim://stat` {#event-netsim-stat}

An impairment relay's counters, every 250 ms. Relays of a run's
[[ui:exp.node.impairment]] nodes report in the run's report instead.

| Field | Type | Meaning |
| --- | --- | --- |
| `job_id` | number | The relay's job |
| `ts` | number | When |
| `received`, `forwarded` | number | Datagrams or chunks in and out |
| `dropped` | number | Lost to `loss`, bursts or `offline` (UDP; a TCP relay holds a stream while offline and drops nothing) |
| `throttled` | number | UDP: dropped by the bandwidth limit, or because too many were already on their way. TCP: chunks that held their stream back for the bandwidth limit |
| `duplicated`, `corrupted`, `reordered` | number | What the profile did to them |
| `bytes` | number | Bytes passed on |
| `connections`, `reset`, `stalled` | number | TCP: connections taken, reset, left half-open; left out while 0 |
| `refused` | number | TCP: connections the target refused or did not answer, passed on to the client as a reset; left out while 0 |
| `failed` | number | What this machine could not pass on: a datagram the system would not send, a connection it had no local port or socket for (tried again for about two seconds first), or one past the relay's 1 024 connections at once; left out while 0 |
| `profile` | string | The profile it impairs with now, as the timeline names it: its name, or what it does (`60 ms ±25 · loss 2%`) |

### `storm://stat` {#event-storm-stat}

A storm's counters, every 250 ms, and once more when it ends on its own with
`pps` and `mbps` 0.

| Field | Type | Meaning |
| --- | --- | --- |
| `job_id` | number | The storm's job |
| `ts` | number | When |
| `packets`, `bytes` | number | Datagrams (or TCP connections) and bytes sent |
| `errors` | number | Sends or connections the target refused, reset or did not answer |
| `local` | number | Units this machine could not send — it ran out of local ports, sockets or buffers, not the target refusing; never counted as `errors` |
| `pps` | number | Per second over the last 250 ms |
| `mbps` | number | Megabits per second over the last 250 ms |

### `scan://open` {#event-scan-open}

The scanner found an open port.

| Field | Type | Meaning |
| --- | --- | --- |
| `job_id` | number | The scan's job |
| `ts` | number | When |
| `port` | number | The port |
| `banner` | string or null | What the service sent first, when banners were asked for and it said something within 400 ms |

### `scan://progress` {#event-scan-progress}

How far a scan is: about every 1 % of the range, and when it ends on its own
with `done` equal to `total` (that one can arrive twice).

| Field | Type | Meaning |
| --- | --- | --- |
| `job_id` | number | The scan's job |
| `ts` | number | When |
| `done` | number | Ports tried |
| `total` | number | Ports in the range |
| `open` | number | Open ports found |
| `unchecked` | number | Ports this machine could not try (it ran out of local ports or sockets), so neither open nor closed; 0 normally |

### `emulator://activity` {#event-emulator-activity}

What an emulator started with `emulator_start` received and answered since the
last event, every 200 ms when something changed (an exchange, being taken
down or brought up, or a message the MQTT broker could not deliver). The [[ui:exp.node.emulator]] nodes of a run do not send it; their counters
are in the run's report.

| Field | Type | Meaning |
| --- | --- | --- |
| `job_id` | number | The emulator's job |
| `ts` | number | When |
| `counts` | object | `total`, `unmatched`, `failed`, `down`, `hits` (per rule), `missed` (MQTT; left out while 0) and `refused` (connections past the limit; left out while 0) — as [`emulator_exchanges`](commands.md#emulator_exchanges) |
| `forced` | string | `unavailable`, `reset` or `timeout` while it is taken down; left out otherwise |
| `exchanges` | object[] | The new exchanges, as `emulator_exchanges` lists them but without `data`; at most 200 |
| `dropped` | number | Exchanges past the first 200 of the interval, not sent here; `emulator_exchanges` still has the last 500 |

### `inspect://batch` {#event-inspect-batch}

New Inspector frames. Sent only while capture is armed: every 120 ms when
there are new frames, and about once a second when there are none, so the
counters stay current.

| Field | Type | Meaning |
| --- | --- | --- |
| `frames` | object[] | The new [frames](commands.md#type-frame), oldest first, at most 250; without their bytes (use `inspect_payload`) |
| `stats` | object | The capture's counters, [`CaptureStats`](commands.md#type-frame) |
| `skipped_now` | number | Frames captured since the last batch but not in this one — more than 250 arrived, or the buffer let them go. They are still in an export while the buffer holds them |

### `server://lagged` {#event-server-lagged}

Server only. This client fell more than 4096 events behind and missed some.
Read the state again with commands.

| Field | Type | Meaning |
| --- | --- | --- |
| `skipped` | number | How many events it missed |
