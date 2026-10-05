---
title: Inspector
description: Capture every frame Signal Lab's tools send and receive, filter and read them decoded and as bytes, export them, and keep one as a signal.
---

# Inspector

The Inspector is one timeline for every tool: each OSC message, datagram,
HTTP exchange, MQTT publish, WebSocket message, relayed packet and emulator
exchange lands there, decoded, with the bytes it was made of. Use it to see
what actually went over the wire, in what order, and what happened to it.

It lives in the bottom panel, as the [[ui:dock.inspector]] tab next to the
console, so it is there on every screen. Click the tab to open it; the panel
opens tall enough for a few rows and a frame's detail. The ⤢ button
([[ui:dock.maximise]]) makes the panel as tall as the window. The Inspector
keeps its list and selection while you close the panel or change screens.

## Capturing {#capture}

Capture is off when Signal Lab starts, and while it is off it costs nothing:
the tools do not even build frames.

1. Open the [[ui:dock.inspector]] tab.
2. Press [[ui:ins.arm]]. The dot on the tab turns red and pulses.
3. Use any tool. Frames appear at the top of the list, newest first.
4. Press [[ui:ins.disarm]] when you have what you need.

The tab shows how many frames have been captured, from any screen.

Frames are captured from the moment you arm capture, never before: arm it
first, then send.

On a [server](../server/index.md), capture belongs to the server: every page
signed in to it sees the same frames, and arming or clearing it on one page
does it for all.

### What is captured {#sources}

| Tool | Frames | How many |
| --- | --- | --- |
| [OSC](../protocols/osc.md): send | Each message sent | Every one |
| [OSC](../protocols/osc.md): monitor | Each packet received; one that does not decode is marked with the decode error | Every one |
| [OSC](../protocols/osc.md): signal generator | Messages sent | At most one per 100 ms, marked `sampled` |
| [Broadcast](../protocols/broadcast.md): send once | Each datagram, one per target; a failed send with its error | Every one |
| [Broadcast](../protocols/broadcast.md): beacon | Datagrams sent | At most one per 50 ms |
| [Broadcast](../protocols/broadcast.md): discovery listener | Probes received | At most one per 40 ms |
| [Broadcast](../protocols/broadcast.md): discovery listener | Its answers (`auto-reply`) | Every one |
| [HTTP](../protocols/http.md): send, signals, experiment requests | Each exchange: the request line, the status and time, the response headers and the start of the body | Every one |
| [HTTP](../protocols/http.md): load burst, and requests under load | Exchanges | At most one per 100 ms |
| [MQTT](../protocols/mqtt.md): connection | Publishes sent | Every one |
| [MQTT](../protocols/mqtt.md): connection | Messages received | At most one per 200 ms |
| [MQTT](../protocols/mqtt.md): an MQTT signal sent while the screen is not connected to its broker | The publish | Every one |
| [WebSocket](../protocols/websocket.md) | Messages sent and received | Every one while traffic is light; at most 200 a second |
| [Emulators](emulators.md) | What arrives and the reply, together | At most one exchange per 10 ms |
| [Impairment](impairment.md) | Every relayed datagram or chunk, both ways, with its fate | At most one per 25 ms for both directions together |
| [Storm](storm.md) (UDP) | Flood packets, all identical | One a second, marked `sampled 1/s`; a TCP storm captures none |
| [Scanner](scanner.md) | Each open port, with its banner | Every one |
| [Experiments](../experiments/index.md) | What a run's steps send (a TCP message: the payload written and the answer read), and what its waits receive | As the tool it uses |

A tool that samples leaves the rest out on purpose, and counts them: the next
frame it does draw says how many it held back, in its verdict as `+n not
shown` (`sampled · +5 not shown`). The count is of what the tool would have
drawn, not of the capture's own shedding (see [Counts and gaps](#counts)).

## The frame list {#list}

| Column | What |
| --- | --- |
| [[ui:common.time]] | When it was captured, to the millisecond. |
| [[ui:ins.dir]] | → sent (`tx`), ← received (`rx`). For a relay, → is client to target and ← target to client. |
| [[ui:bc.proto]] | `osc`, `udp`, `tcp`, `http`, `mqtt` or `ws`. |
| [[ui:bc.peer]] | The other side: an `IP:port`, a URL, a broker. |
| [[ui:common.bytes]] | The frame's size. |
| [[ui:ins.summary]] | One line in the protocol's own notation, such as `/fader/1 0.75` or `GET http://127.0.0.1:8080/ → 200 in 3ms`. |
| [[ui:ins.verdict]] | What happened to it, when there is something to say. |

The verdict is green, amber or red. Red is a loss or a failure (`dropped
(loss)`, `failed`, `error: …`); amber is a frame altered or only a sample of
many (`corrupted`, `copy 2/2`, `sampled`, `+n not shown`); green is the rest. Some verdicts you
will meet:

| Verdict | From | Means |
| --- | --- | --- |
| `forwarded +42ms` | Impairment | Passed on after that delay; `· corrupted`, `· reordered` or `· copy 1/2` may follow. |
| `dropped (loss)`, `dropped (burst)`, `dropped (offline)` | Impairment | Lost on purpose, and why. |
| `throttled` | Impairment | Dropped by the bandwidth limit. |
| `· client→target`, `· target→client` | Impairment | Ends the verdict of every relayed frame, before any `+n not shown`: which way it was going. |
| `#2 → 200 OK · 37 B`, `— → 404 …` | Emulators | Which rule answered (`—`: none) and the answer. |
| `down`, `down → 503` | Emulators | It arrived while the emulator was down. |
| `200 OK`, `failed` | HTTP | The response status, or no response at all. |
| `open` | Scanner | An open port. |
| `auto-reply` | Discovery | An answer the listener sent to a probe. |
| `clears retained` | MQTT | An empty retained publish. |
| `+n not shown` | Any tool that samples | That many frames since the previous one were left out; it follows the frame's other verdict after a `·`. |

The list keeps the newest 4000 frames and draws the newest 300 that match the
filters; under the filters it says how many it shows of how many match.

### Filtering {#filter}

- Type in the text field ([[ui:ins.filterPlaceholder]]) to keep frames whose
  summary, peer, source, protocol or verdict contains the text.
- Click protocol chips (`osc`, `udp`, `tcp`, `http`, `mqtt`, `ws`) to show only
  those protocols. With no chip on, every protocol shows.
- Click `tx` or `rx` to show only sent or only received frames.
- [[ui:common.reset]] clears all three.

Filters change only what the list shows. Capture, the counts and an export
always cover everything.

### Pausing and clearing {#pause}

[[ui:ins.pause]] freezes the list so you can read it while traffic goes on;
capture continues. [[ui:ins.resume]] lets new frames in again. Frames that
arrived while the view was paused are not added to the list, but they are in
the capture and in an export.

[[ui:common.clear]] empties the list and the capture, and resets its counts.

### Counts and gaps {#counts}

The bar at the top counts the frames captured and their bytes, and how full the
capture is (frames held of 8192).

When frames arrive faster than the list can take them — more than 250 in about
an eighth of a second — the list skips the oldest of them. An amber chip then
counts the frames not shown, and a row in the list marks where they are
missing. Those frames are still in the capture, unless newer ones have since
pushed them out: export it to see them.

## A frame's detail {#detail}

Click a row to see the frame on the right.

| Field | What |
| --- | --- |
| [[ui:ins.seq]] | The frame's number. Numbers rise in capture order and are never reused. |
| [[ui:common.time]] | When it was captured. |
| [[ui:ins.direction]] | Sent or received. |
| [[ui:common.protocol]] | As in the list. |
| [[ui:ins.source]] | The tool that captured it (`osc-send`, `osc-monitor`, `netsim`, `emulator`, `experiment-wait`, …), and its job number when it belongs to one. |
| [[ui:ins.local]] | The address on this side, when there is one. For a relayed frame, the address the relay listens on. |
| [[ui:bc.peer]] | The other side. For a relayed frame, where it was going: the target, or the client the answer went back to. |
| [[ui:ins.size]] | Its size in bytes. |
| [[ui:ins.verdict]] | As in the list. |

Under [[ui:ins.decoded]] is the frame read in its protocol: every message of
an OSC bundle with its arguments, an HTTP response's headers and the start of
its body, an emulator's request and its reply.

Under [[ui:ins.rawBytes]] is a hex dump: offset, 16 bytes in hex, and the same
bytes as text. The list carries the first KiB of each frame; when a frame is
longer, a button under the dump loads all of it.

### What a frame keeps {#limits}

| Limit | Value | At the limit |
| --- | --- | --- |
| Bytes one frame keeps | 256 KiB | A longer frame keeps its first 256 KiB and says how much of the total it kept. |
| Frames in the capture | 8192 | The oldest frame makes room. |
| Bytes the capture keeps in all | 64 MiB | The oldest frames make room. |

Some frames keep no bytes: HTTP exchanges (their size is recorded, and the
response headers and the start of the body are in the decoded text instead) and
open ports from the Scanner.

An MQTT frame keeps the message's payload, not the protocol's packet around it;
the topic, QoS and retain flag are in its summary.

### Secrets {#secrets}

While an experiment run or [[ui:exp.sendNow]] uses [secrets](../experiments/data.md#secrets),
their values are masked in every frame before it is captured: `••••` in the
summary, the decoded text, the addresses and the verdict, and `*` for each byte
in the payload, so offsets in the dump stay true. The HTTP screen's credentials
never appear either: an HTTP frame holds the response, not the
`Authorization` header that was sent.

## Saving a frame as a signal {#save-as-signal}

To keep a packet you caught and send it again later — when the device that
sent it is no longer there:

1. Select the frame.
2. Press [[ui:sig.fromFrame]].

The signal goes into the [[ui:sig.capturedFolder]] folder of the
[signal library](signals.md) with every byte of the frame, taken from what the
capture kept, not from the decoded text. It is named after the frame's summary,
and its note says which frame it came from.

What you get depends on the frame:

| Frame | Signal |
| --- | --- |
| An OSC or UDP datagram | A raw UDP signal with the frame's bytes, hex. |
| An MQTT publish — sent, received, or an emulator's | An MQTT signal with the frame's broker, topic, QoS and retain flag, and its payload as text, exactly as it was. An empty payload is kept, so clearing a retained value can be saved. |
| Anything else: a TCP stream (including one a relay carried, or the TCP node's), an HTTP exchange, a WebSocket message, an MQTT packet that is not a publish (a client's subscribe at an emulator) | Nothing: the button is off, and its tip says why. A signal sends one datagram or one publish; these cannot be sent again as they were. |

A datagram is sent to:

- a received frame — the address that received it (the [[ui:ins.local]]
  side), so the signal stands in for the sender;
- a sent frame — the peer it was sent to;
- a relayed frame, either way — the address it was going to: the target for a
  frame going from the client, the client for an answer.

When that address is every address of this computer — a monitor listening on
`0.0.0.0:9000` or `[::]:9000` — the signal is aimed at this computer instead:
`127.0.0.1:9000` or `[::1]:9000`. Open the signal and change its target if you
mean another address.

An MQTT signal goes to the broker the frame says. A broker listening on every
address is reached on `127.0.0.1` the same way.

The button is also off for:

- a frame not kept whole: one larger than 256 KiB, or one that kept no bytes;
- an MQTT message whose payload is not text — a signal's payload is text, so
  its bytes could not be sent again as they were;
- a received frame that names no socket on this side, so there is no address to send to.

A frame the capture has already let go cannot be saved either; the console says
so. While the library file cannot be read, [[ui:sig.fromFrame]] is off too, and
the tip shows the file's error (see [Signals](signals.md#file)).

## Exporting {#export}

[[ui:ins.exportJsonl]] and [[ui:ins.exportTxt]] write the whole capture — up to
8192 frames, every byte each one kept, whatever the filters show — to a file
`capture-<time>.jsonl` or `capture-<time>.txt` in the data folder (see
[Files](../reference/files.md)). The console says where. In a browser
connected to a server, the file is written on the server and your browser
downloads it.

An empty capture is not written; the console says there is nothing to save.

- **`.jsonl`** — one JSON object per line, one line per frame: `seq`, `ts`
  (milliseconds since 1970), `proto`, `dir`, `source`, `job_id`, `local`,
  `remote`, `bytes`, `kept`, `summary`, `detail`, `hex` (the dump of the first
  KiB), `verdict`, and `data`, the bytes it kept as base64.
- **`.txt`** — for reading: a line per frame with its number, time, direction,
  protocol, peer, size and verdict, then its summary, its decoded text and a
  hex dump of every byte it kept.

```json
{"seq":12,"ts":1767225600123,"proto":"osc","dir":"rx","source":"osc-monitor","job_id":3,"local":"0.0.0.0:9000","remote":"127.0.0.1:53211","bytes":20,"summary":"/fader/1 0.75","detail":"/fader/1 0.75","hex":"0000  2f 66 61 64 65 72 2f 31  00 00 00 00 2c 66 00 00  |/fader/1....,f..|\n0010  3f 40 00 00                                       |?@..|\n","verdict":null,"kept":20,"data":"L2ZhZGVyLzEAAAAALGYAAD9AAAA="}
```

## Coming from elsewhere {#reveal}

Other screens point at frames: a wait in an experiment's timeline links the
frame it matched, and an emulator's received list has a ⌕ button
([[ui:emu.inspectFrame]]) on each exchange. Following one opens the
Inspector with that frame selected, the filters cleared and the view resumed.

## Related {#related}

- [Signals](signals.md) — what a saved frame becomes.
- [Impairment](impairment.md) — the relay's verdicts.
- [Emulators](emulators.md) — what an emulator received.
