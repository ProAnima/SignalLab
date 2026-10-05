---
title: UDP and TCP
description: Where Signal Lab sends and receives raw UDP datagrams and TCP data — experiment steps, signals, the command line, the load and scan tools and emulated devices — and how payloads are written.
---

# UDP and TCP

Raw UDP and TCP have no screen of their own. They are what you use for a
device with its own text or binary protocol — a projector, a media server, a
sensor — and they turn up in several places:

| To… | Use |
| --- | --- |
| send a datagram or a TCP message as a step, and wait for the answer | the [experiment steps](#experiments) |
| keep a datagram to send again, or replay one you captured | a [UDP signal](#signals) |
| send one datagram from a script | [`signallab send udp`](#cli) |
| send to many hosts at once, to a broadcast address or a multicast group | the [Broadcast](broadcast.md) screen |
| load a server or a link with traffic | [Storm](#storm) |
| find which TCP ports a host has open | [Scanner](#scanner) |
| play the device's side | a [UDP or TCP device](#emulators) emulator |
| make the network between two ends worse | an [impairment relay](#impairment) |

OSC is a format carried in UDP datagrams; it has its own page:
[OSC](osc.md).

## Payloads {#payloads}

Wherever you write a raw payload, it is one of two kinds:

| Kind | What is sent | Example |
| --- | --- | --- |
| Text | The characters as UTF-8, exactly as typed: no terminator, no line ending added. A line protocol needs its line ending in the text. | `PING` |
| Hex bytes | Byte for byte, written as pairs of hex digits. Spaces, `:`, `-` and `,` between pairs and a `0x` before them are allowed. | `de ad be ef`, `DEADBEEF`, `0xde,0xad` |

A datagram carries at most 65 507 bytes. An odd number of hex digits, or none,
is an error before anything is sent.

::: info
UDP has no delivery receipt. "Sent" means the datagram left this machine, not
that anything received it. To know that a device heard you, wait for its answer.
:::

## Where it goes {#destinations}

A destination is an IP address and a port, or a host name and a port:
`192.0.2.20:9000`, `[2001:db8::20]:9000` (an IPv6 address goes in brackets),
`projector.local:9000`. This holds for a UDP step's target, an OSC target,
a UDP signal, `signallab send udp` and `signallab send osc`, the
[Broadcast](broadcast.md) list, [Storm](../tools/storm.md)'s target and the TCP
step's host.

A host name is looked up each time it is used. When it has an IPv4 address,
that one is used — so `localhost` reaches a service listening on `127.0.0.1`,
where the first address a system lists for it may be `::1` — and a name that
has only IPv6 addresses is reached from an IPv6 socket. A name that does not
resolve fails with `Cannot resolve …`; a destination with no port, or that is
neither form, with `… is not a valid address`. Addresses a service *listens*
on (a monitor's, a wait's, an emulator's) are always `IP:port`.

## In experiments {#experiments}

| Step | What it does |
| --- | --- |
| [[ui:exp.node.udp]] | Sends its [[ui:exp.payload]] as text to [[ui:common.target]]. The target is `IP:port` or `host:port` ([above](#destinations)), and several targets separated by commas each get the datagram. With [[ui:exp.expectReply]] it sends from [[ui:exp.replyOn]] and waits there for an answer in the same step. [Details](../experiments/nodes.md#node-udp) |
| [[ui:exp.node.wait_udp]] | Listens on [[ui:exp.listenOn]] (`IP:port`) and waits for a datagram whose payload matches. [Details](../experiments/nodes.md#node-wait_udp) |
| [[ui:exp.node.tcp]] | Connects to [[ui:exp.host]] and [[ui:exp.port]], writes its [[ui:exp.payload]] as text, listens 250 ms for an answer, and closes. The step says how many bytes came back, not what they were. [Details](../experiments/nodes.md#node-tcp) |

The UDP payload and target, the TCP host and payload, and a wait's pattern take
`{{templates}}`, so a datagram can carry the run's id or a value an earlier step
extracted. See [Data and templates](../experiments/data.md).

### Matching a datagram {#matching}

[[ui:exp.node.wait_udp]] and the reply of [[ui:exp.node.udp]] choose a datagram
by its payload:

| [[ui:exp.waitMode]] | Takes a datagram when |
| --- | --- |
| [[ui:exp.mode.any]] | always: the first one that arrives |
| [[ui:exp.mode.contains]] | its payload, read as text, contains the pattern |
| [[ui:exp.mode.regex]] | its payload, read as text, matches the regular expression |
| [[ui:exp.mode.hex]] | its bytes contain the pattern's bytes, written in hex |

What matched is kept in the step's variable ([[ui:exp.replyVariable]],
`reply` unless you rename it): `text`, `hex` (the first 1024 bytes), `bytes`
(the size), `from` (the sender's `IP:port`), `ms` (how long it took) and `match`
(the text or bytes found, or a regular expression's first group).

A wait starts listening when the run starts, not when the step is reached, so
an answer that comes very fast is not missed. It takes only what arrived after
the latest send on its path.

### Limits and defaults {#limits}

| Setting | Default | Range |
| --- | --- | --- |
| TCP step: [[ui:common.timeoutMs]] (connecting, writing and the answer together) | 4000 ms | 1–120 000 ms |
| [[ui:exp.waitTimeout]] of a wait or a reply | 2000 ms | 1–120 000 ms |
| UDP payload | — | at most 65 507 bytes |
| Listening address | — | `IP:port` with a port; a reply's [[ui:exp.replyOn]] may use port 0 (any free port) |

In the Inspector, a UDP step's datagram appears with the source `broadcast`
(or `experiment` when the step waits for a reply), and every datagram arriving
at a wait's port with the source `experiment-wait`. A TCP step appears as two
`tcp` frames with the source `experiment`: the payload it wrote and, when one
came, the answer it read. Its timeline entry says what it sent and how big the
answer was.

## Signals {#signals}

A [[ui:sig.tr.udp]] signal in the library is a target and a payload, as text or
hex bytes. Fire it from [[ui:nav.signals]], or with <kbd>Ctrl</kbd>+<kbd>K</kbd>
from any screen. Its target may be a host name.

Any datagram the Inspector kept whole can become one: [[ui:sig.fromFrame]] makes a hex
UDP signal, in the [[ui:sig.capturedFolder]] folder, that replays those exact
bytes — to the frame's destination for a frame that was sent, to the address that
received it for one that came in (on this computer, when that was every address).
A TCP chunk cannot: it is a piece of a stream. See [Signals](../tools/signals.md) and
[Inspector](../tools/inspector.md#save-as-signal).

A text UDP signal can be added to an experiment as a [[ui:exp.node.udp]] step;
a hex one cannot, because the step sends text.

## From the command line {#cli}

`signallab send udp` sends one datagram:

```bash
signallab send udp 127.0.0.1:9000 --text "PING"
signallab send udp 127.0.0.1:9000 --hex "de ad be ef"
```

```text
✔ sent 4 bytes → 127.0.0.1:9000
```

Give exactly one of `--text` and `--hex`. The target is `IP:port` or
`host:port` ([above](#destinations)). It exits with 0 when the datagram went out, 1 when sending failed,
and 2 when the target or the hex is invalid. There is no `send tcp`. See [Command line](../automation/cli.md#cli-send-udp).

## Storm {#storm}

[[ui:nav.storm]] is a load source for your own servers and links: a
[[ui:st.udp]] sends datagrams of a set size at a set rate, a [[ui:st.tcp]]
opens a connection, writes the payload and closes, again and again. Throughput
is measured live. See [Storm](../tools/storm.md).

## Scanner {#scanner}

[[ui:nav.scan]] tries a TCP connection to each port in a range and lists the
ones that accept, with what the service says first if you ask for banners. See
[Scanner](../tools/scanner.md).

::: danger
Storm and Scanner send real traffic to real hosts. Point them only at systems
you own or are allowed to test: a storm can saturate a link, and both can trip
intrusion detection.
:::

## Emulated devices {#emulators}

On the [[ui:nav.emulators]] screen, Signal Lab can be the device:

- a [[ui:emu.new.udp]] answers datagrams by rules on their payload — any,
  containing a text, matching a regular expression, containing bytes — with a
  text or hex reply built from what arrived, to the sender or to another
  `IP:port`, after a delay if you set one;
- a [[ui:emu.new.tcp]] accepts connections, splits what arrives into messages
  at a line ending you choose (LF, CR LF, CR, or every chunk as it comes),
  answers each by the same kind of rules, can send a greeting when a client
  connects, and can close the connection after a reply.

Both can also run as an [[ui:exp.node.emulator]] step for the length of a run.
See [Emulators](../tools/emulators.md).

## Impairment {#impairment}

The [[ui:nav.netsim]] relay sits between a client and its target and degrades
what passes: per datagram over UDP (delay, loss, duplicates, reordering, a
bandwidth limit) or per stream over TCP (delay, a bandwidth limit, connections
reset or left half-open). See [Impairment](../tools/impairment.md) and, inside
an experiment, [Faults](../experiments/faults.md).

## "Port unreachable" on Windows {#port-unreachable}

When a datagram reaches a port where nothing listens, the receiving machine
usually answers with an ICMP "port unreachable" message. Windows reports that
answer on the sending socket's next receive, as if the connection had been
reset — even though UDP has no connection.

Signal Lab expects this. Its listeners — the OSC monitor, the discovery
listener, experiment waits and replies, UDP and OSC emulators, the impairment
relay — note it and carry on listening. A device that has gone away does not
stop them. A listener that really cannot receive any more ends its job, and the
console says why.

## Problems {#troubleshooting}

| What you see | Usual cause |
| --- | --- |
| `… is not a valid address` | The target has no port, or is neither `IP:port` nor `host:port`. |
| `Cannot resolve …` | The host name does not resolve on this machine. |
| `… refused the connection — nothing is listening on that port` (TCP) | Nothing listens on that port, or a firewall rejects it. |
| `No answer from … in time` (TCP) | The host does not answer at all — wrong address, or a firewall that drops instead of rejecting. |
| A wait times out though the device answers | The device answers to the port the datagram came from, not to the wait's port. Let the send wait for the reply itself with [[ui:exp.expectReply]]: it then goes out from the port the answer comes back to. |
| Datagrams from other machines never arrive | On Windows, the firewall can keep them out: allow Signal Lab when the app offers it. See [Troubleshooting](../reference/troubleshooting.md). |

Every error message is listed in [Error messages](../reference/errors.md#transport).
